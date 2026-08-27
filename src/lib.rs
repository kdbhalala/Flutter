use std::collections::BTreeSet;

use zed::lsp::CompletionKind;
use zed::settings::LspSettings;
use zed::{CodeLabel, CodeLabelSpan};
use zed_extension_api::process::Command;
use zed_extension_api::serde_json::json;
use zed_extension_api::{
    self as zed, current_platform, serde_json, DebugAdapterBinary, DebugConfig, DebugRequest,
    DebugScenario, DebugTaskDefinition, Os, Result, SlashCommand, SlashCommandArgumentCompletion,
    SlashCommandOutput, StartDebuggingRequestArguments, StartDebuggingRequestArgumentsRequest,
    TaskTemplate, Worktree,
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

fn is_top_level_flutter_pubspec(content: &str) -> bool {
    content.lines().any(|line| {
        let trimmed = line.trim_end();
        !matches!(line.chars().next(), Some(' ' | '\t')) && trimmed == "flutter:"
    })
}

/// Detect whether the project at `cwd` is a Flutter or plain Dart project by
/// reading `pubspec.yaml`. Falls back to the program path shape when `cwd`
/// is unavailable.
fn detect_debug_type(cwd: &str, program: Option<&str>) -> &'static str {
    if !cwd.is_empty() {
        let pubspec_path = format!("{}/pubspec.yaml", cwd);
        if let Ok(content) = std::fs::read_to_string(&pubspec_path) {
            return if is_top_level_flutter_pubspec(&content) {
                "flutter"
            } else {
                "dart"
            };
        }
    }

    if let Some(program) = program.map(str::trim).filter(|program| !program.is_empty()) {
        if program == "lib/main.dart" || program.starts_with("lib/") || program.contains("/lib/") {
            return "flutter";
        }
        if program.starts_with("bin/") || program.contains("/bin/") {
            return "dart";
        }
    }

    "dart"
}

/// Helper to find SDK binaries in known version manager and workspace locations
fn find_version_manager_tool(worktree: &Worktree, tool: &str) -> Option<String> {
    // 1. Workspace-local FVM symlink: .fvm/flutter_sdk/bin/<tool>
    let local_fvm = format!("{}/.fvm/flutter_sdk/bin/{}", worktree.root_path(), tool);
    if std::path::Path::new(&local_fvm).exists() {
        return Some(local_fvm);
    }

    let env = worktree.shell_env();
    let (_, home) = env.iter().find(|(k, _)| k == "HOME")?;

    // 2. Global FVM, Puro, Proto, asdf, mise
    for candidate in [
        format!("{}/.fvm/default/bin/{}", home, tool),
        format!("{}/.puro/envs/default/bin/{}", home, tool),
        format!("{}/.puro/bin/{}", home, tool),
        format!("{}/.proto/shims/{}", home, tool),
        format!("{}/.proto/bin/{}", home, tool),
        format!("{}/.asdf/shims/{}", home, tool),
        format!("{}/.local/share/mise/shims/{}", home, tool),
    ] {
        if std::path::Path::new(&candidate).exists() {
            return Some(candidate);
        }
    }
    None
}

/// Resolve flutter/dart tool path with priority:
/// 1. FVM binary (if use_fvm)
/// 2. dart.sdkPath / dart.flutterSdkPath from LSP settings
/// 3. Workspace-local or global version manager (FVM, Puro, Proto, asdf, mise)
/// 4. PATH via worktree.which
/// 5. FLUTTER_ROOT env var → <root>/bin/<tool>
fn flutter_tool_path(worktree: &Worktree, tool: &str, use_fvm: bool) -> String {
    if use_fvm {
        return worktree.which("fvm").unwrap_or_else(|| "fvm".to_string());
    }
    if let Some(sdk_bin) = sdk_path_from_settings(worktree, tool) {
        return sdk_bin;
    }
    if let Some(vm_tool) = find_version_manager_tool(worktree, tool) {
        return vm_tool;
    }
    if let Some(path) = worktree.which(tool) {
        return path;
    }
    let env = worktree.shell_env();
    if let Some((_, root)) = env.iter().find(|(k, _)| k == "FLUTTER_ROOT") {
        return format!("{}/bin/{}", root, tool);
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

fn parse_flutter_devices(stdout: &str) -> std::result::Result<Vec<serde_json::Value>, String> {
    match serde_json::from_str(stdout.trim()) {
        Ok(serde_json::Value::Array(arr)) => Ok(arr),
        Ok(_) => Err("flutter devices returned unexpected JSON".into()),
        Err(_) => stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(serde_json::from_str::<serde_json::Value>)
            .collect::<std::result::Result<_, _>>()
            .map_err(|e| format!("flutter devices output is not valid JSON: {e}")),
    }
}

fn slash_completion(
    label: impl Into<String>,
    new_text: impl Into<String>,
) -> SlashCommandArgumentCompletion {
    SlashCommandArgumentCompletion {
        label: label.into(),
        new_text: new_text.into(),
        run_command: false,
    }
}

fn push_token_completion(completions: &mut Vec<SlashCommandArgumentCompletion>, token: &str) {
    completions.push(slash_completion(token, token));
}

fn flutter_device_slash_command_completions(use_fvm: bool) -> Vec<SlashCommandArgumentCompletion> {
    let mut seen = BTreeSet::new();
    let mut completions = Vec::new();

    for alias in ["chrome", "ios", "android", "macos", "linux", "windows", "web"] {
        if seen.insert(alias.to_string()) {
            completions.push(slash_completion(alias, alias));
        }
    }

    let (os, _) = current_platform();
    let flutter_tool = match os {
        Os::Windows => "flutter.bat",
        _ => "flutter",
    };

    let cmd = if use_fvm {
        let command = Command::new("fvm");
        command.arg("flutter")
    } else {
        Command::new(flutter_tool)
    };

    if let Ok(output) = cmd.args(["devices", "--machine"]).output() {
        if output.status.unwrap_or(-1) == 0 {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Ok(devices) = parse_flutter_devices(stdout.trim()) {
                for device in devices {
                    if let Some(id) = device.get("id").and_then(|value| value.as_str()) {
                        if seen.insert(id.to_string()) {
                            let label = device
                                .get("name")
                                .and_then(|value| value.as_str())
                                .map(|name| format!("{id} ({name})"))
                                .unwrap_or_else(|| id.to_string());
                            completions.push(slash_completion(label, id));
                        }
                    }
                }
            }
        }
    }

    completions
}

fn flutter_slash_command_completions(
    args: &[String],
    use_fvm: bool,
) -> Vec<SlashCommandArgumentCompletion> {
    let mut completions = Vec::new();

    if args.is_empty() {
        for token in [
            "run",
            "test",
            "devices",
            "doctor",
            "devtools",
            "attach",
            "pub",
            "emulators",
            "clean",
            "screenshot",
            "gen-l10n",
            "create",
            "build",
            "upgrade",
        ] {
            push_token_completion(&mut completions, token);
        }
        return completions;
    }

    if let Some(position) = args.iter().position(|arg| arg == "-d") {
        if args.len() <= position + 2 {
            return flutter_device_slash_command_completions(use_fvm);
        }
    }

    if let Some(position) = args.iter().position(|arg| arg == "--web-renderer") {
        if args.len() <= position + 2 {
            for token in ["auto", "html", "canvaskit", "skwasm"] {
                push_token_completion(&mut completions, token);
            }
            return completions;
        }
    }

    match args[0].as_str() {
        "run" if args.len() == 1 => {
            for token in ["-d", "--release", "--web-renderer"] {
                push_token_completion(&mut completions, token);
            }
        }
        "test" if args.len() == 1 => {
            push_token_completion(&mut completions, "--coverage");
        }
        "pub" => match args.len() {
            1 => {
                for token in ["get", "upgrade", "outdated", "run"] {
                    push_token_completion(&mut completions, token);
                }
            }
            2 if args[1] == "run" => {
                push_token_completion(&mut completions, "build_runner");
            }
            3 if args[1] == "run" && args[2] == "build_runner" => {
                for token in ["build", "watch"] {
                    push_token_completion(&mut completions, token);
                }
            }
            4 if args[1] == "run"
                && args[2] == "build_runner"
                && matches!(args[3].as_str(), "build" | "watch") =>
            {
                push_token_completion(&mut completions, "--delete-conflicting-outputs");
            }
            _ => {}
        },
        "build" => match args.len() {
            1 => {
                for token in ["apk", "appbundle", "ios", "ipa", "web"] {
                    push_token_completion(&mut completions, token);
                }
            }
            2 if matches!(args[1].as_str(), "apk" | "ios") => {
                push_token_completion(&mut completions, "--release");
            }
            _ => {}
        },
        _ => {}
    }

    completions
}

fn dart_slash_command_completions(args: &[String]) -> Vec<SlashCommandArgumentCompletion> {
    let mut completions = Vec::new();

    if args.is_empty() {
        for token in [
            "run",
            "test",
            "analyze",
            "format",
            "fix",
            "pub",
            "devtools",
            "create",
            "compile",
            "doc",
        ] {
            push_token_completion(&mut completions, token);
        }
        return completions;
    }

    match args[0].as_str() {
        "run" => match args.len() {
            1 => push_token_completion(&mut completions, "build_runner"),
            2 if args[1] == "build_runner" => {
                for token in ["build", "watch"] {
                    push_token_completion(&mut completions, token);
                }
            }
            3 if args[1] == "build_runner" && matches!(args[2].as_str(), "build" | "watch") => {
                push_token_completion(&mut completions, "--delete-conflicting-outputs");
            }
            _ => {}
        },
        "test" if args.len() == 1 => {
            push_token_completion(&mut completions, "--coverage=coverage");
        }
        "fix" if args.len() == 1 => {
            push_token_completion(&mut completions, "--apply");
        }
        "pub" => match args.len() {
            1 => {
                for token in ["get", "upgrade", "outdated", "cache"] {
                    push_token_completion(&mut completions, token);
                }
            }
            2 if args[1] == "cache" => push_token_completion(&mut completions, "repair"),
            _ => {}
        },
        "compile" if args.len() == 1 => {
            push_token_completion(&mut completions, "exe");
        }
        _ => {}
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
    let mut cmd = Command::new(flutter_tool_path(worktree, tool, use_fvm));
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
    let arr = parse_flutter_devices(stdout.trim())?;

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

        // Workspace or global version managers: FVM -> Puro -> Proto -> asdf -> mise
        if let Some(vm_dart) = find_version_manager_tool(worktree, "dart") {
            return Ok(DartBinary {
                path: vm_dart,
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

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let user_settings = LspSettings::for_worktree("dart", worktree)
            .ok()
            .and_then(|s| s.settings)
            .and_then(|s| {
                s.as_object()
                    .and_then(|o| o.get("dart"))
                    .and_then(|d| d.as_object())
                    .cloned()
            });

        let get_bool = |key: &str, default: bool| -> bool {
            user_settings
                .as_ref()
                .and_then(|s| s.get(key))
                .and_then(|v| v.as_bool())
                .unwrap_or(default)
        };

        let inlay_hints_settings = user_settings
            .as_ref()
            .and_then(|s| s.get("inlayHints"))
            .and_then(|h| h.as_object());

        let get_inlay_bool = |key: &str, default: bool| -> bool {
            inlay_hints_settings
                .and_then(|h| h.get(key))
                .and_then(|v| v.as_bool())
                .unwrap_or(default)
        };

        Ok(Some(serde_json::json!({
            // Only analyze Dart projects that have files open in the editor.
            "onlyAnalyzeProjectsWithOpenFiles": get_bool("onlyAnalyzeProjectsWithOpenFiles", true),
            // Surface completion items from libraries not yet imported.
            "suggestFromUnimportedLibraries": get_bool("suggestFromUnimportedLibraries", true),
            // Show closing labels for long Flutter widget trees.
            "closingLabels": get_bool("closingLabels", true),
            // Complete function calls with parentheses / arguments.
            "completeFunctionCalls": get_bool("completeFunctionCalls", true),
            // Enable document outline support.
            "outline": true,
            // Enable Flutter widget outline support.
            "flutterOutline": true,
            // Allow the language server to open URIs (for dart fix, etc.).
            "allowOpenUri": true,
            // Inlay hints for parameter names, variable types, and return types.
            "inlayHints": {
                "showForParameters": get_inlay_bool("showForParameters", true),
                "showForVariableTypes": get_inlay_bool("showForVariableTypes", true),
                "showForFunctionReturnTypes": get_inlay_bool("showForFunctionReturnTypes", true),
                "showForChainedCalls": get_inlay_bool("showForChainedCalls", true),
            }
        })))
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
            "type": debug_mode,
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

        let resolved_program = if program.is_empty() {
            "lib/main.dart".to_string()
        } else {
            program
        };

        // Detect flutter vs dart from pubspec.yaml; falls back to program path heuristics.
        let debug_type = detect_debug_type(&cwd, Some(resolved_program.as_str()));

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
            // Enum, Interface, Struct, TypeParameter — render like types
            CompletionKind::Enum
            | CompletionKind::Interface
            | CompletionKind::Struct
            | CompletionKind::TypeParameter => Some(CodeLabel {
                filter_range: (0..completion.label.len()).into(),
                spans: vec![CodeLabelSpan::literal(
                    completion.label,
                    Some("type".into()),
                )],
                code: String::new(),
            }),
            // Enum members and constants
            CompletionKind::EnumMember | CompletionKind::Constant => {
                let name = completion.label;
                Some(CodeLabel {
                    filter_range: (0..name.len()).into(),
                    spans: vec![CodeLabelSpan::literal(name, Some("constant".into()))],
                    code: String::new(),
                })
            }
            // Fields render like properties
            CompletionKind::Field => {
                let arrow = " → ";
                let name = completion.label.clone();
                if let Some(ty) = completion.detail {
                    let class_start = "class A {";
                    let get = " get ";
                    let property_end = " => a; }";
                    let code = format!("{class_start}{ty}{get}{name}{property_end}");
                    let name_start = class_start.len() + ty.len() + get.len();
                    Some(CodeLabel {
                        spans: vec![
                            CodeLabelSpan::code_range(name_start..name_start + name.len()),
                            CodeLabelSpan::literal(arrow, None),
                            CodeLabelSpan::code_range(
                                class_start.len()..class_start.len() + ty.len(),
                            ),
                        ],
                        filter_range: (0..name.len()).into(),
                        code,
                    })
                } else {
                    Some(CodeLabel {
                        filter_range: (0..name.len()).into(),
                        spans: vec![CodeLabelSpan::literal(name, Some("property".into()))],
                        code: String::new(),
                    })
                }
            }
            // Modules (libraries / packages)
            CompletionKind::Module => {
                let name = completion.label;
                Some(CodeLabel {
                    filter_range: (0..name.len()).into(),
                    spans: vec![CodeLabelSpan::literal(name, Some("keyword.import".into()))],
                    code: String::new(),
                })
            }
            // Keywords
            CompletionKind::Keyword => {
                let name = completion.label;
                Some(CodeLabel {
                    filter_range: (0..name.len()).into(),
                    spans: vec![CodeLabelSpan::literal(name, Some("keyword".into()))],
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
            "flutter" => Ok(flutter_slash_command_completions(&args, false)),
            "dart" => Ok(dart_slash_command_completions(&args)),
            "fvm" => {
                if args.is_empty() {
                    Ok(vec![slash_completion("flutter", "flutter")])
                } else if args.len() == 1 && args[0] == "flutter" {
                    Ok(flutter_slash_command_completions(&[], true))
                } else if !args.is_empty() && args[0] == "flutter" {
                    Ok(flutter_slash_command_completions(&args[1..], true))
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
            return Err(text.to_string());
        }

        Ok(SlashCommandOutput {
            text,
            sections: Vec::new(),
        })
    }

    fn dap_locator_create_scenario(
        &mut self,
        _locator_name: String,
        build_task: TaskTemplate,
        _resolved_label: String,
        debug_adapter_name: String,
    ) -> Option<DebugScenario> {
        if debug_adapter_name != "Dart" {
            return None;
        }

        // Normalize the command to its basename so absolute paths work too.
        let cmd_base = std::path::Path::new(&build_task.command)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(build_task.command.as_str())
            .trim_end_matches(".bat")
            .to_lowercase();

        // Resolve effective tool and args when FVM is the launcher.
        let (tool, effective_args): (&str, &[String]) = if cmd_base == "fvm" {
            match build_task.args.first().map(|s| s.as_str()) {
                Some("flutter") => ("flutter", &build_task.args[1..]),
                Some("dart") => ("dart", &build_task.args[1..]),
                _ => return None,
            }
        } else if cmd_base == "flutter" || cmd_base == "flutter.bat" {
            ("flutter", build_task.args.as_slice())
        } else if cmd_base == "dart" || cmd_base == "dart.bat" {
            ("dart", build_task.args.as_slice())
        } else {
            return None;
        };

        // Only convert tasks that actually launch a debuggable process.
        let subcommand = effective_args.first().map(|s| s.as_str()).unwrap_or("");
        let debug_type = match (tool, subcommand) {
            ("flutter", "run") | ("flutter", "test") => "flutter",
            ("dart", "run") | ("dart", "test") => "dart",
            // `dart path/to/file.dart` direct invocation
            ("dart", s) if s.ends_with(".dart") => "dart",
            _ => return None,
        };

        let mut explicit_program = None;
        let mut args = Vec::new();
        let mut remaining_args = effective_args.iter().skip(1).peekable();

        while let Some(arg) = remaining_args.next() {
            if (arg.as_str() == "-t" || arg.as_str() == "--target")
                && remaining_args
                    .peek()
                    .is_some_and(|target| target.ends_with(".dart"))
            {
                explicit_program = remaining_args.next().cloned();
                continue;
            }

            if arg.ends_with(".dart") {
                explicit_program = Some(arg.clone());
                continue;
            }

            args.push(arg.clone());
        }

        if subcommand == "test" && explicit_program.is_none() {
            return None;
        }

        if tool == "dart"
            && subcommand == "run"
            && explicit_program.is_none()
            && effective_args
                .iter()
                .skip(1)
                .any(|arg| !arg.starts_with('-'))
        {
            return None;
        }

        let program = explicit_program.unwrap_or_else(|| {
            if debug_type == "flutter" {
                "lib/main.dart".to_string()
            } else {
                "bin/main.dart".to_string()
            }
        });

        let use_fvm = cmd_base == "fvm";

        let config_json = serde_json::json!({
            "type": debug_type,
            "request": "launch",
            "program": program,
            "cwd": build_task.cwd.clone().unwrap_or_default(),
            "args": args,
            "useFvm": use_fvm,
            "flutterMode": "debug",
            "debugSdkLibraries": false,
            "debugExternalPackageLibraries": false,
            "stopOnEntry": false,
        })
        .to_string();

        Some(DebugScenario {
            label: build_task.label.clone(),
            adapter: "Dart".to_string(),
            build: None,
            config: config_json,
            tcp_connection: None,
        })
    }
}

zed::register_extension!(FlutterExtension);
