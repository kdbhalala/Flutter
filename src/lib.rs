use zed_extension_api::{
    self as zed, DebugAdapterBinary, DebugTaskDefinition, LanguageServerId,
    StartDebuggingRequestArguments, StartDebuggingRequestArgumentsRequest, Worktree,
};

struct FlutterExtension;

impl FlutterExtension {
    fn dart_binary(&self, worktree: &Worktree) -> Result<String, String> {
        worktree
            .which("dart")
            .ok_or_else(|| "dart not found in PATH. Install from flutter.dev".to_string())
    }

    fn flutter_binary(&self, worktree: &Worktree) -> Result<String, String> {
        worktree
            .which("flutter")
            .ok_or_else(|| "flutter not found in PATH. Install from flutter.dev".to_string())
    }
}

impl zed::Extension for FlutterExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<zed::Command, String> {
        let dart = self.dart_binary(worktree)?;
        Ok(zed::Command {
            command: dart,
            args: vec![
                "language-server".to_string(),
                "--protocol=lsp".to_string(),
                "--client-id=zed".to_string(),
                "--client-version=1".to_string(),
            ],
            env: Default::default(),
        })
    }

    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &LanguageServerId,
        _worktree: &Worktree,
    ) -> Result<Option<zed::serde_json::Value>, String> {
        Ok(Some(zed::serde_json::json!({
            "dart": {
                "lineLength": 80,
                "enableSdkFormatter": true,
                "completeFunctionCalls": true,
                "showTodos": true,
                "analysisExcludedFolders": [".dart_tool", ".pub-cache", "build"]
            }
        })))
    }

    fn get_dap_binary(
        &mut self,
        _adapter_name: String,
        config: DebugTaskDefinition,
        _user_provided_debug_adapter_path: Option<String>,
        worktree: &Worktree,
    ) -> Result<DebugAdapterBinary, String> {
        let raw: zed::serde_json::Value =
            zed::serde_json::from_str(&config.config).unwrap_or_default();

        let use_fvm = raw
            .get("useFvm")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let (command, args) = if use_fvm {
            let fvm = worktree
                .which("fvm")
                .ok_or_else(|| "fvm not found in PATH".to_string())?;
            (
                fvm,
                vec![
                    "flutter".to_string(),
                    "debug-adapter".to_string(),
                ],
            )
        } else {
            let flutter = self.flutter_binary(worktree)?;
            (flutter, vec!["debug-adapter".to_string()])
        };

        let request = match raw.get("request").and_then(|v| v.as_str()) {
            Some("attach") => StartDebuggingRequestArgumentsRequest::Attach,
            _ => StartDebuggingRequestArgumentsRequest::Launch,
        };

        Ok(DebugAdapterBinary {
            command: Some(command),
            arguments: args,
            envs: Default::default(),
            cwd: raw
                .get("cwd")
                .and_then(|v| v.as_str())
                .map(String::from),
            connection: None,
            request_args: StartDebuggingRequestArguments {
                configuration: config.config,
                request,
            },
        })
    }
}

zed::register_extension!(FlutterExtension);
