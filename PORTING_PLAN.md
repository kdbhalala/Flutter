# Flutter for Zed Porting Plan

## Implementation Status

### Completed

| Feature | Notes |
|---------|-------|
| Dart LSP | `dart language-server --protocol=lsp`; binary resolved from LSP settings → PATH → FLUTTER_ROOT → ~/.fvm/default |
| dart.env passthrough | `dart.env` key from LSP settings injected into both the language server process and the debug adapter process |
| Tree-sitter grammar | Syntax highlighting, bracket matching, indentation, outline |
| DAP launch/attach | `flutter debug_adapter` / `dart debug_adapter`; full launch and attach flows |
| FVM support | `useFvm: true` switches to `fvm flutter debug_adapter`; FVM task variants provided |
| SDK fallbacks | `which` → `dart.sdkPath`/`dart.flutterSdkPath` LSP setting → `FLUTTER_ROOT` env → existence-checked loop: FVM default → asdf shims (`~/.asdf/shims`) → mise shims (`~/.local/share/mise/shims`) |
| Device resolution | `resolve_device_id()` mirrors VS Code Dart-Code fuzzy matching against `flutter devices --machine` |
| `/flutter` slash command | run, test, devices, doctor, attach, pub get/upgrade/outdated, emulators, clean, screenshot, gen-l10n, create, build apk/apk--release/appbundle/ios/ios--release/ipa/web |
| `/dart` slash command | run, test, analyze, format, fix, pub get/upgrade/outdated, create, compile, doc |
| `/fvm` slash command | delegates to flutter completions |
| Rich completion labels | Class/Function/Constructor/Method/Property/Variable with icons and types |
| Workspace config passthrough | Full LSP settings forwarded to the Dart language server |
| `dap_config_to_scenario` | Converts Zed new-session UI `DebugConfig` → `DebugScenario`; reads `pubspec.yaml` in `cwd` to detect flutter vs dart project (falls back to `"flutter"` when not readable) |
| Extra DAP config fields | `debugSdkLibraries`, `debugExternalPackageLibraries`, `flutterMode` (debug/profile/release), `webRenderer` (injected as `--web-renderer=X`), `additionalArgs` |
| Tasks | flutter: run, run (release), pub get/upgrade/outdated, build apk/apk--release/appbundle/ios/ios--release/ipa/web, test, test file, clean, devices, doctor, emulators, gen-l10n, attach, screenshot; dart: run file, test file, format, analyze, pub get/upgrade/outdated; fvm: flutter variants |

### Blocked by Zed Extension API Limitations

| VS Code Feature | Blocker |
|-----------------|---------|
| Hot reload / hot restart | No Zed extension API for sending custom messages to a running debug session |
| Flutter Outline panel | Zed has no secondary panel / custom tree-view API |
| Dart DevTools integration | No browser-launch or webview API in Zed extensions |
| Coverage gutters | No gutter-decoration API |
| Widget inspector | Requires custom UI panel not yet exposed |
| Profile / memory views | Same — no custom panel API |
| Snippet completions | Zed snippet API not yet available to extensions |
| Flutter-specific code actions | LSP code actions work but flutter-specific ones (wrap with Widget, extract method) require a Flutter-aware code action provider not exposed by the Zed API |

### Not Yet Ported (API available, not yet implemented)

None — all feasible features are implemented.

