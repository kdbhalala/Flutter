use zed::lsp::CompletionKind;
use zed::settings::LspSettings;
use zed::{CodeLabel, CodeLabelSpan};
use zed_extension_api::process::Command;
use zed_extension_api::serde_json::json;
use zed_extension_api::{
    self as zed, current_platform, serde_json, DebugAdapterBinary, DebugConfig, DebugRequest,
    DebugScenario, DebugTaskDefinition, Os, Result, SlashCommand, SlashCommandArgumentCompletion,
    SlashCommandOutput, StartDebuggingRequestArguments, StartDebuggingRequestArgumentsRequest,
    Worktree,
};

struct DartBinary {
    pub path: String,
    pub args: Option<Vec<String>>,
}

struct FlutterExtension;

/// Read `dart.sdkPath` (for dart) or `dart.flutterSdkPath` (for flutter) from
/// LSP settings and return `<sdkPath>/bin/<tool>`, or `None` when not set.
fn sdk_path_from_settings(worktree: &Worktree, tool: &str) -> Option<String> {
    let settings = LspSettings::for_worktree("dart", worktree)
        .ok()
        .and_then(|s| s.settings)?;
    let dart_settings = settings.as_object()?.get("dart")?;
    let sdk_key = if tool == "flutter" || tool == "flutter.bat" {
        "flutterSdkPath"
    } else {
        "sdkPath"
    };
    let sdk_path = dart_settings.get(sdk_key).and_then(|v| v.as_str())?;
    Some(format!("{}/bin/{}", sdk_path, tool))
}

/// Detect whether the project at `cwd` is a Flutter or plain Dart project by
/// reading `pubspec.yaml`. Presence of a `flutter:` key marks Flutter.
/// Falls back to `"flutter"` when the file is not readable.
fn detect_debug_type(cwd: &str) -> &'static str {
    if !cwd.is_empty() {
        let pubspec_path = format!("{}/pubspec.yaml", cwd);
        if let Ok(content) = std::fs::read_to_string(&pubspec_path) {
            return if content.contains("flutter:") {
                "flutter"
            } else {
                "dart"
            };
        }
    }
    "flutter"
}

/// Resolve flutter/dart tool path with priority:
/// 1. FVM binary (if use_fvm)
/// 2. dart.sdkPath / dart.flutterSdkPath from LSP settings
/// 3. PATH via worktree.which
/// 4. FLUTTER_ROOT env var → <root>/bin/<tool>
/// 5. Version manager paths checked for existence: FVM default → asdf → mise
fn flutter_tool_path(worktree: &Worktree, tool: &str, use_fvm: bool) -> String {
    if use_fvm {
        return worktree.which("fvm").unwrap_or_else(|| "fvm".to_string());
    }
    if let Some(sdk_bin) = sdk_path_from_settings(worktree, tool) {
        return sdk_bin;
    }
    if let Some(path) = worktree.which(tool) {
        return path;
    }
    let env = worktree.shell_env();
    if let Some((_, root)) = env.iter().find(|(k, _)| k == "FLUTTER_ROOT") {
        return format!("{}/bin/{}", root, tool);
    }
    if let Some((_, home)) = env.iter().find(|(k, _)| k == "HOME") {
        let home = home.clone();
        for candidate in [
            format!("{}/.fvm/default/bin/{}", home, tool),
            format!("{}/.asdf/shims/{}", home, tool),
            format!("{}/.local/share/mise/shims/{}", home, tool),
        ] {
            if std::path::Path::new(&candidate).exists() {
                return candidate;
            }
        }
    }
    tool.to_string()
}

fn slash_command_tool_path(worktree: Option<&Worktree>, tool: &str) -> String {
    worktree
        .and_then(|wt| wt.which(tool))
        .unwrap_or_else(|| tool.to_string())
}

/// Read dart.env from LSP settings and return as env-var pairs injected into
/// the language server and debug adapter processes.
fn dart_extra_env(worktree: &Worktree) -> Vec<(String, String)> {
    LspSettings::for_worktree("dart", worktree)
        .ok()
        .and_then(|s| s.settings)
        .and_then(|s| {
            s.as_object()
                .and_then(|o| o.get("dart"))
                .and_then(|d| d.get("env"))
                .and_then(|e| {
                    serde_json::from_value::<std::collections::HashMap<String, String>>(e.clone())
                        .ok()
                })
        })
        .map(|m| m.into_iter().collect())
        .unwrap_or_default()
}

fn flutter_slash_command_completions(args: &[String]) -> Vec<SlashCommandArgumentCompletion> {
    let top_level = [
        "run",
        "test",
        "devices",
        "doctor",
        "attach",
        "pub get",
        "pub upgrade",
        "pub outdated",
        "emulators",
        "clean",
        "screenshot",
        "gen-l10n",
        "create",
        "build apk",
        "build apk --release",
        "build appbundle",
        "build ios",
        "build ios --release",
        "build ipa",
        "build web",
    ];

    let mut completions: Vec<SlashCommandArgumentCompletion> = Vec::new();
    if args.is_empty() {
        for label in top_level.iter() {
            completions.push(SlashCommandArgumentCompletion {
                label: label.to_string(),
                new_text: label.to_string(),
                run_command: false,
            });
        }
        return completions;
    }

    match args[0].as_str() {
        "pub" => {
            for label in ["pub get", "pub upgrade", "pub outdated"] {
                completions.push(SlashCommandArgumentCompletion {
                    label: label.to_string(),
                    new_text: label.to_string(),
                    run_command: false,
                });
            }
        }
        "build" => {
            for label in [
                "build apk",
                "build apk --release",
                "build appbundle",
                "build ios",
                "build ios --release",
                "build ipa",
                "build web",
            ] {
                completions.push(SlashCommandArgumentCompletion {
                    label: label.to_string(),
                    new_text: label.to_string(),
                    run_command: false,
                });
            }
        }
        _ => {}
    }

    completions
}

fn dart_slash_command_completions(args: &[String]) -> Vec<SlashCommandArgumentCompletion> {
    let top_level = [
        "run",
        "test",
        "analyze",
        "format .",
        "fix",
        "pub get",
        "pub upgrade",
        "pub outdated",
        "create",
        "compile",
        "doc",
    ];

    let mut completions: Vec<SlashCommandArgumentCompletion> = Vec::new();
    if args.is_empty() {
        for label in top_level.iter() {
            completions.push(SlashCommandArgumentCompletion {
                label: label.to_string(),
                new_text: label.to_string(),
                run_command: false,
            });
        }
        return completions;
    }

    if args[0].as_str() == "pub" {
        for label in ["pub get", "pub upgrade", "pub outdated", "pub cache repair"] {
            completions.push(SlashCommandArgumentCompletion {
                label: label.to_string(),
                new_text: label.to_string(),
                run_command: false,
            });
        }
    }

    completions
}

/// Mirrors the fuzzy matching of VSCode Dart-Code's `findBestDevice()`.
///
/// Runs `flutter devices --machine`, parses the JSON device list, and
/// finds the best match for `search` using the same priority order:
///   1. Exact ID match
///   2. Exact name match
///   3. ID starts with search
///   4. Name starts with search
///   5. ID contains search
///   6. Name contains search
///
/// Returns:
///   Ok(Some(id))  – resolved device ID
///   Ok(None)      – no match found; caller should surface available devices
///   Err(msg)      – flutter devices failed; caller should fall through
fn resolve_device_id(
    worktree: &Worktree,
    tool: &str,
    use_fvm: bool,
    search: &str,
) -> std::result::Result<Option<String>, String> {
    let mut cmd = Command::new(&flutter_tool_path(worktree, tool, use_fvm));
    if use_fvm {
        cmd = cmd.arg(tool);
    }
    let output = cmd
        .args(["devices", "--machine"])
        .output()
        .map_err(|e| format!("flutter devices failed: {e}"))?;

    // status is Option<i32>; treat non-zero or signal-killed as failure.
    if output.status.unwrap_or(-1) != 0 {
        return Err(format!(
            "flutter devices exited with code {:?}",
            output.status
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let devices: Vec<serde_json::Value> = match serde_json::from_str(stdout.trim()) {
        Ok(serde_json::Value::Array(arr)) => arr,
        Ok(_) => return Err("flutter devices returned unexpected JSON".into()),
        Err(_) => stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str::<serde_json::Value>(line))
            .collect::<std::result::Result<_, _>>()
            .map_err(|e| format!("flutter devices output is not valid JSON: {e}"))?,
    };
    let arr = devices;

    let s = search.to_lowercase();

    let id_of = |d: &serde_json::Value| {
        d.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    let name_of = |d: &serde_json::Value| {
        d.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_lowercase()
    };
    let id_lower = |d: &serde_json::Value| id_of(d).to_lowercase();

    // Priority 1 – exact ID
    if let Some(d) = arr.iter().find(|d| id_lower(d) == s) {
        return Ok(Some(id_of(d)));
    }
    // Priority 2 – exact name
    if let Some(d) = arr.iter().find(|d| name_of(d) == s) {
        return Ok(Some(id_of(d)));
    }
    // Priority 3 – ID starts with search
    if let Some(d) = arr.iter().find(|d| id_lower(d).starts_with(&s)) {
        return Ok(Some(id_of(d)));
    }
    // Priority 4 – name starts with search
    if let Some(d) = arr.iter().find(|d| name_of(d).starts_with(&s)) {
        return Ok(Some(id_of(d)));
    }
    // Priority 5 – ID contains search
    if let Some(d) = arr.iter().find(|d| id_lower(d).contains(&s)) {
        return Ok(Some(id_of(d)));
    }
    // Priority 6 – name contains search
    if let Some(d) = arr.iter().find(|d| name_of(d).contains(&s)) {
        return Ok(Some(id_of(d)));
    }

    // No match – build a human-readable list of available device IDs.
    let available: Vec<String> = arr
        .iter()
        .filter_map(|d| {
            let id = d.get("id")?.as_str()?;
            let name = d.get("name")?.as_str().unwrap_or("?");
            Some(format!("  {id} ({name})"))
        })
        .collect();

    if available.is_empty() {
        Ok(None)
    } else {
        Err(format!(
            "No device matching \"{search}\".\nAvailable devices:\n{}",
            available.join("\n")
        ))
    }
}

impl FlutterExtension {
    fn language_server_binary(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<DartBinary> {
        let binary_settings = LspSettings::for_worktree("dart", worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.binary);
        let binary_args = binary_settings.as_ref().and_then(|s| s.arguments.clone());

        if let Some(path) = binary_settings.and_then(|s| s.path) {
            return Ok(DartBinary {
                path,
                args: binary_args,
            });
        }

        // dart.sdkPath setting takes priority over PATH search.
        if let Some(sdk_bin) = sdk_path_from_settings(worktree, "dart") {
            return Ok(DartBinary {
                path: sdk_bin,
                args: binary_args,
            });
        }

        if let Some(path) = worktree.which("dart") {
            return Ok(DartBinary {
                path,
                args: binary_args,
            });
        }

        // FLUTTER_ROOT fallback — dart lives in <FLUTTER_ROOT>/bin/dart
        let env = worktree.shell_env();
        if let Some((_, root)) = env.iter().find(|(k, _)| k == "FLUTTER_ROOT") {
            return Ok(DartBinary {
                path: format!("{}/bin/dart", root),
                args: binary_args,
            });
        }

        // Version-manager fallbacks: FVM default → asdf shims → mise shims
        if let Some((_, home)) = env.iter().find(|(k, _)| k == "HOME") {
            let home = home.clone();
            for candidate in [
                format!("{}/.fvm/default/bin/dart", home),
                format!("{}/.asdf/shims/dart", home),
                format!("{}/.local/share/mise/shims/dart", home),
            ] {
                if std::path::Path::new(&candidate).exists() {
                    return Ok(DartBinary {
                        path: candidate,
                        args: binary_args.clone(),
                    });
                }
            }
        }

        Err("dart must be installed from dart.dev/get-dart or pointed to by the LSP binary settings".to_string())
    }
}

impl zed::Extension for FlutterExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let dart_binary = self.language_server_binary(language_server_id, worktree)?;
        let extra_env = dart_extra_env(worktree);
        Ok(zed::Command {
            command: dart_binary.path,
            args: dart_binary.args.unwrap_or_else(|| {
                vec!["language-server".to_string(), "--protocol=lsp".to_string()]
            }),
            env: extra_env,
        })
    }

    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let settings = LspSettings::for_worktree("dart", worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings.clone())
            .unwrap_or_default();

        Ok(Some(serde_json::json!({ "dart": settings })))
    }

    fn get_dap_binary(
        &mut self,
        _adapter_name: String,
        config: DebugTaskDefinition,
        _user_provided_debug_adapter_path: Option<String>,
        worktree: &Worktree,
    ) -> Result<DebugAdapterBinary, String> {
        let user_config: serde_json::Value = serde_json::from_str(&config.config)
            .map_err(|e| format!("Failed to parse debug config: {e}"))?;

        let program = user_config
            .get("program")
            .and_then(|v| v.as_str())
            .unwrap_or("lib/main.dart");

        let args = user_config
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let use_fvm = user_config
            .get("useFvm")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let debug_mode = user_config
            .get("type")
            .and_then(|v| v.as_str())
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| "type is required and cannot be empty or null".to_string())?;

        let flutter_mode = user_config
            .get("flutterMode")
            .and_then(|v| v.as_str())
            .unwrap_or("debug")
            .to_string();

        let debug_sdk_libraries = user_config
            .get("debugSdkLibraries")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let debug_external_libraries = user_config
            .get("debugExternalPackageLibraries")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let web_renderer = user_config
            .get("webRenderer")
            .and_then(|v| v.as_str())
            .map(String::from);

        let additional_args: Vec<String> = user_config
            .get("additionalArgs")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();

        let (os, _) = current_platform();
        let tool = if debug_mode == "flutter" {
            match os {
                Os::Windows => "flutter.bat",
                _ => "flutter",
            }
        } else {
            match os {
                Os::Windows => "dart.bat",
                _ => "dart",
            }
        };

        let command_path = flutter_tool_path(worktree, tool, use_fvm);
        let (command, arguments) = if use_fvm {
            (
                command_path,
                vec![tool.to_string(), "debug_adapter".to_string()],
            )
        } else {
            (command_path, vec!["debug_adapter".to_string()])
        };

        let device_id = user_config.get("device_id").and_then(|v| v.as_str());
        let platform = user_config.get("platform").and_then(|v| v.as_str());

        // Resolve device_id against the live device list, mirroring VSCode's
        // FlutterDeviceManager.findBestDevice() fuzzy matching.  We only run
        // `flutter devices --machine` when the user explicitly set a device_id,
        // because the Flutter DAP handles device selection itself otherwise.
        let resolved_device_id: Option<String> = if let Some(search) = device_id {
            match resolve_device_id(worktree, tool, use_fvm, search) {
                Ok(Some(id)) => Some(id),
                Ok(None) => {
                    // flutter devices ran but returned nothing at all (no devices).
                    // Fall through with the raw value and let the DAP report.
                    Some(search.to_string())
                }
                Err(msg) if msg.starts_with("No device matching") => {
                    // Device list was retrieved but search had no match.
                    return Err(msg);
                }
                Err(_) => {
                    // flutter devices failed (not in PATH, SDK issue, etc.).
                    // Fall through with the raw value so the DAP gives its own error.
                    Some(search.to_string())
                }
            }
        } else {
            None
        };
        let cwd = user_config
            .get("cwd")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| Some(worktree.root_path()));
        let request = user_config
            .get("request")
            .and_then(|v| v.as_str())
            .unwrap_or("launch");
        let vm_service_uri = user_config.get("vmServiceUri").and_then(|v| v.as_str());

        // Merge user args + additionalArgs + web renderer flag
        let all_args: Vec<String> = {
            let mut combined = args.clone();
            combined.extend(additional_args);
            if let Some(ref renderer) = web_renderer {
                combined.push(format!("--web-renderer={}", renderer));
            }
            combined
        };

        let mut config_map = json!({
            "type": tool,
            "request": request,
            "program": program,
            "cwd": cwd.clone().unwrap_or_default(),
            "args": all_args,
            "flutterMode": flutter_mode,
            "debugSdkLibraries": debug_sdk_libraries,
            "debugExternalPackageLibraries": debug_external_libraries,
            "stopOnEntry": false
        });

        // Only inject optional fields when explicitly set by the user.
        // Omitting them lets Flutter's debug adapter use its own device
        // selection logic, avoiding stale UDIDs or wrong platform defaults.
        if let Some(did) = resolved_device_id {
            config_map["deviceId"] = json!(did);
        }
        if let Some(plat) = platform {
            config_map["platform"] = json!(plat);
        }
        if let Some(uri) = vm_service_uri {
            config_map["vmServiceUri"] = json!(uri);
        }

        let config_json = config_map.to_string();

        // Pass dart.env settings into the debug adapter process environment.
        let extra_env = dart_extra_env(worktree);

        Ok(DebugAdapterBinary {
            command: Some(command),
            arguments,
            envs: extra_env,
            cwd,
            connection: None,
            request_args: StartDebuggingRequestArguments {
                configuration: config_json,
                request: match request {
                    "attach" => StartDebuggingRequestArgumentsRequest::Attach,
                    _ => StartDebuggingRequestArgumentsRequest::Launch,
                },
            },
        })
    }

    fn dap_request_kind(
        &mut self,
        _adapter_name: String,
        config: serde_json::Value,
    ) -> Result<StartDebuggingRequestArgumentsRequest, String> {
        match config.get("request") {
            Some(v) if v == "launch" => Ok(StartDebuggingRequestArgumentsRequest::Launch),
            Some(v) if v == "attach" => Ok(StartDebuggingRequestArgumentsRequest::Attach),
            Some(value) => Err(format!("Unexpected value for `request`: {value:?}")),
            None => Err("Missing required `request` field in debug config".into()),
        }
    }

    /// Convert the generic new-session UI config into a Dart/Flutter debug scenario.
    /// Detects dart vs flutter by inspecting the program path.
    fn dap_config_to_scenario(&mut self, config: DebugConfig) -> Result<DebugScenario, String> {
        let (request_str, program, cwd, args) = match config.request {
            DebugRequest::Launch(req) => {
                ("launch", req.program, req.cwd.unwrap_or_default(), req.args)
            }
            DebugRequest::Attach(_) => ("attach", String::new(), String::new(), vec![]),
        };

        // Detect flutter vs dart from pubspec.yaml; falls back to "flutter".
        let debug_type = detect_debug_type(&cwd);

        let resolved_program = if program.is_empty() {
            "lib/main.dart".to_string()
        } else {
            program
        };

        let config_json = serde_json::json!({
            "type": debug_type,
            "request": request_str,
            "program": resolved_program,
            "cwd": cwd,
            "args": args,
            "flutterMode": "debug",
            "debugSdkLibraries": false,
            "debugExternalPackageLibraries": false,
            "stopOnEntry": config.stop_on_entry.unwrap_or(false),
        })
        .to_string();

        Ok(DebugScenario {
            label: config.label,
            adapter: "Dart".to_string(),
            build: None,
            config: config_json,
            tcp_connection: None,
        })
    }

    fn label_for_completion(
        &self,
        _language_server_id: &zed::LanguageServerId,
        completion: zed::lsp::Completion,
    ) -> Option<CodeLabel> {
        let arrow = " → ";
        match completion.kind? {
            CompletionKind::Class => Some(CodeLabel {
                filter_range: (0..completion.label.len()).into(),
                spans: vec![CodeLabelSpan::literal(
                    completion.label,
                    Some("type".into()),
                )],
                code: String::new(),
            }),
            CompletionKind::Function | CompletionKind::Constructor | CompletionKind::Method => {
                let mut parts = completion.detail.as_ref()?.split(arrow);
                let (name, _) = completion.label.split_once('(')?;
                let parameter_list = parts.next()?;
                let return_type = parts.next()?;
                let fn_name = " a";
                let fat_arrow = " => ";
                let call_expr = "();";
                let code =
                    format!("{return_type}{fn_name}{parameter_list}{fat_arrow}{name}{call_expr}");
                let parameter_list_start = return_type.len() + fn_name.len();
                Some(CodeLabel {
                    spans: vec![
                        CodeLabelSpan::code_range(
                            code.len() - call_expr.len() - name.len()..code.len() - call_expr.len(),
                        ),
                        CodeLabelSpan::code_range(
                            parameter_list_start..parameter_list_start + parameter_list.len(),
                        ),
                        CodeLabelSpan::literal(arrow, None),
                        CodeLabelSpan::code_range(0..return_type.len()),
                    ],
                    filter_range: (0..name.len()).into(),
                    code,
                })
            }
            CompletionKind::Property => {
                let class_start = "class A {";
                let get = " get ";
                let property_end = " => a; }";
                let ty = completion.detail?;
                let name = completion.label;
                let code = format!("{class_start}{ty}{get}{name}{property_end}");
                let name_start = class_start.len() + ty.len() + get.len();
                Some(CodeLabel {
                    spans: vec![
                        CodeLabelSpan::code_range(name_start..name_start + name.len()),
                        CodeLabelSpan::literal(arrow, None),
                        CodeLabelSpan::code_range(class_start.len()..class_start.len() + ty.len()),
                    ],
                    filter_range: (0..name.len()).into(),
                    code,
                })
            }
            CompletionKind::Variable => {
                let name = completion.label;
                Some(CodeLabel {
                    filter_range: (0..name.len()).into(),
                    spans: vec![CodeLabelSpan::literal(name, Some("variable".into()))],
                    code: String::new(),
                })
            }
            _ => None,
        }
    }

    fn complete_slash_command_argument(
        &self,
        command: SlashCommand,
        args: Vec<String>,
    ) -> Result<Vec<SlashCommandArgumentCompletion>, String> {
        match command.name.as_str() {
            "flutter" => Ok(flutter_slash_command_completions(&args)),
            "dart" => Ok(dart_slash_command_completions(&args)),
            "fvm" => {
                if args.is_empty() {
                    Ok(vec![SlashCommandArgumentCompletion {
                        label: "flutter".to_string(),
                        new_text: "flutter".to_string(),
                        run_command: false,
                    }])
                } else if args.len() == 1 && args[0] == "flutter" {
                    Ok(flutter_slash_command_completions(&[]))
                } else {
                    Ok(Vec::new())
                }
            }
            _ => Ok(Vec::new()),
        }
    }

    fn run_slash_command(
        &self,
        command: SlashCommand,
        args: Vec<String>,
        worktree: Option<&Worktree>,
    ) -> Result<SlashCommandOutput, String> {
        let tool = match command.name.as_str() {
            "flutter" => "flutter",
            "dart" => "dart",
            "fvm" => "fvm",
            other => return Err(format!("Unsupported slash command: {other}")),
        };

        let path = slash_command_tool_path(worktree, tool);
        let output = Command::new(&path)
            .args(args.clone())
            .output()
            .map_err(|e| format!("Failed to spawn {tool}: {e}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let text = if !stdout.trim().is_empty() {
            stdout
        } else if !stderr.trim().is_empty() {
            stderr
        } else {
            String::from("Command completed successfully.")
        };

        if output.status.unwrap_or(-1) != 0 {
            return Err(format!("{text}"));
        }

        Ok(SlashCommandOutput {
            text,
            sections: Vec::new(),
        })
    }
}

zed::register_extension!(FlutterExtension);
