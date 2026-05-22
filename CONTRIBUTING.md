# Contributing to Flutter for Zed

This is a Zed editor extension written in Rust that compiles to WebAssembly. It provides Flutter and Dart language support via the Dart Analysis Server (LSP), the Flutter/Dart debug adapter (DAP), tasks, snippets, and slash commands.

---

## Publishing Prerequisites

Before submitting a PR for publication to the Zed extensions registry, ensure you have completed the [Extension Publishing Prerequisites](https://zed.dev/docs/extensions/developing-extensions#extension-publishing-prerequisites):

- ✅ **License**: MIT license included (see [LICENSE](LICENSE))
- ✅ **Manifest**: Valid `extension.toml` with all required fields
- ✅ **Documentation**: Comprehensive README and CONTRIBUTING guide
- ⚠️ **Testing**: Manual verification in Zed is required (see [Testing](#testing) below)
- ⚠️ **No duplication**: Verify this extension doesn't duplicate functionality already provided by another extension

### Duplicate Functionality Check

Before opening a PR for publication, search the [Zed extensions registry](https://github.com/zed-industries/extensions) to ensure:
- No other extension provides Flutter or Dart language support
- If functionality overlaps with existing extensions, consider contributing fixes to those instead

---

## Repository layout

```
flutter/
├── Cargo.toml                     # Rust crate — compiles to WASM cdylib
├── extension.toml                 # Zed extension manifest
├── src/
│   └── lib.rs                     # All extension logic (single file)
├── languages/
│   └── dart/
│       ├── config.toml            # Language config (tab size, comment tokens)
│       ├── highlights.scm         # Tree-sitter highlight queries
│       ├── brackets.scm           # Bracket pair definitions
│       ├── indents.scm            # Indentation rules
│       ├── injections.scm         # Language injection rules
│       ├── outline.scm            # Outline panel symbol queries
│       └── tasks.json             # Built-in task definitions
├── grammars/
│   └── dart/                      # Tree-sitter Dart grammar source
│       ├── grammar.js             # Grammar definition
│       └── src/
│           ├── parser.c           # Generated C parser
│           └── scanner.c          # External scanner (strings, interpolation)
├── debug_adapter_schemas/
│   └── Dart.json                  # Debug configuration JSON schema
├── CHANGELOG.md                   # Version history and release notes
└── PORTING_PLAN.md                # What is done, what is blocked
```

---

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | stable (≥ 1.77) | [rustup.rs](https://rustup.rs) |
| WASM target | `wasm32-wasip1` | `rustup target add wasm32-wasip1` |
| Zed | latest | [zed.dev](https://zed.dev) |
| Flutter SDK | any | [flutter.dev](https://flutter.dev/docs/get-started/install) |

Install the WASM target once:

```sh
rustup target add wasm32-wasip1
```

---

## Building

```sh
cargo build --target wasm32-wasip1
```

The compiled WASM artifact lands in `target/wasm32-wasip1/debug/zed_flutter.wasm`.

For a release build:

```sh
cargo build --target wasm32-wasip1 --release
```

---

## Running locally in Zed

1. Open Zed.
2. Open the command palette (`cmd+shift+p`) → **"zed: install dev extension"**.
3. Select this repository's root folder.
4. Zed rebuilds the WASM on each file save — no manual `cargo build` needed during development.

Alternatively, add to `~/.config/zed/settings.json`:

```json
{
  "dev_extensions": ["/path/to/this/repo"]
}
```

---

## Testing

Before submitting a PR, you **must** test the extension locally as a dev extension in Zed. PRs for extensions that clearly don't work will be closed.

### Manual Testing Checklist

- [ ] **Build succeeds**: `cargo build --target wasm32-wasip1` completes without errors
- [ ] **Dev extension loads**: "zed: install dev extension" successfully loads the extension in Zed
- [ ] **Dart files recognized**: Dart syntax highlighting works in `.dart` files
- [ ] **LSP works**: Autocomplete, go-to-definition, and diagnostics function correctly
- [ ] **Tasks execute**: Run at least one Flutter and one Dart task via `cmd+shift+p` → **"task: spawn"**
- [ ] **Slash commands respond**: Test `/flutter`, `/dart`, and `/fvm` slash commands in the assistant
- [ ] **Debug session launches**: Create a simple Flutter project and verify debug configuration works
- [ ] **No console errors**: Check Zed's extension logs (View → Toggle Extension Logs) for errors

### Testing with Sample Projects

For comprehensive testing, use:
- **Flutter project**: `flutter create test_app && cd test_app`
- **Dart CLI project**: `dart create -t console test_cli`

---

## Architecture

### Entry point — `src/lib.rs`

All logic lives in a single `lib.rs`. It implements the `zed::Extension` trait:

```
FlutterExtension
├── language_server_command()       → start dart language-server
├── language_server_workspace_configuration() → forward LSP settings
├── get_dap_binary()                → resolve debug adapter binary + config
├── dap_request_kind()              → classify launch vs attach
├── dap_config_to_scenario()        → new-session UI → DebugScenario
├── label_for_completion()          → rich completion labels
├── complete_slash_command_argument() → tab completions for slash commands
└── run_slash_command()             → execute flutter/dart/fvm CLI
```

Key free functions:

| Function | Purpose |
|----------|---------|
| `flutter_tool_path(worktree, tool, use_fvm)` | Resolve `flutter`/`dart` binary with full SDK fallback chain |
| `sdk_path_from_settings(worktree, tool)` | Read `dart.sdkPath` / `dart.flutterSdkPath` from LSP settings |
| `dart_extra_env(worktree)` | Read `dart.env` from LSP settings → env-var pairs |
| `detect_debug_type(cwd)` | Read `pubspec.yaml` → `"flutter"` or `"dart"` |
| `resolve_device_id(worktree, tool, use_fvm, search)` | Fuzzy-match device against `flutter devices --machine` |
| `flutter_slash_command_completions(args)` | Completions for `/flutter` |
| `dart_slash_command_completions(args)` | Completions for `/dart` |
| `slash_command_tool_path(worktree, tool)` | Resolve binary for slash command execution |

### SDK resolution order

`flutter_tool_path` and `language_server_binary` search in this order:

1. `dart.sdkPath` / `dart.flutterSdkPath` from LSP settings
2. `worktree.which(tool)` (PATH)
3. `FLUTTER_ROOT` environment variable → `$FLUTTER_ROOT/bin/<tool>`
4. `~/.fvm/default/bin/<tool>` (existence checked)
5. `~/.asdf/shims/<tool>` (existence checked)
6. `~/.local/share/mise/shims/<tool>` (existence checked)

### Debug adapter flow

```
get_dap_binary(config)
  ├── parse JSON config
  ├── resolve tool (flutter vs dart, FVM flag)
  ├── resolve device_id via resolve_device_id() fuzzy match
  ├── merge args + additionalArgs + --web-renderer=<webRenderer>
  ├── inject dart.env into process envs
  └── return DebugAdapterBinary { command, arguments, envs, cwd, request_args }
```

The debug adapter binary is `flutter debug_adapter` (for Flutter) or `dart debug_adapter` (for Dart). FVM prepends `fvm` and passes the tool as the first argument.

### `dap_config_to_scenario`

Called when the user creates a new debug session from the Zed UI (not from `.zed/debug.json`). Reads `pubspec.yaml` in the `cwd` to determine `"flutter"` vs `"dart"` type. Falls back to `"flutter"` if detection fails.

### Slash commands

`/flutter`, `/dart`, `/fvm` run the corresponding CLI tool with the given subcommand arguments in the workspace root. `run_slash_command` spawns the process and returns stdout/stderr as text.

### tasks.json

Plain JSON array of Zed task definitions. Each entry has `label`, `command`, `args`, and `tags`. `$ZED_FILE` and `$ZED_STEM` are Zed task variables expanded at runtime.

---

## Adding a new task

Edit `languages/dart/tasks.json`. Follow the existing pattern:

```json
{
  "label": "flutter: <description>",
  "command": "flutter",
  "args": ["<subcommand>", "<flags>"],
  "tags": ["flutter-<tag>"]
}
```

No Rust changes required for tasks.

---

## Adding a new slash command subcommand

1. Find `flutter_slash_command_completions` or `dart_slash_command_completions` in `src/lib.rs`.
2. Add the new label string to the `top_level` array or the appropriate `match` arm.
3. No changes to `extension.toml` needed (commands are already registered).

---

## Adding a new slash command

1. Add a `[[slash_commands]]` block to `extension.toml`:

```toml
[[slash_commands]]
name = "mycommand"
description = "Short description"
tooltip_text = "Shown in UI"
requires_argument = true
```

2. Add a new completions function in `src/lib.rs`:

```rust
fn mycommand_slash_command_completions(args: &[String]) -> Vec<SlashCommandArgumentCompletion> {
    // ...
}
```

3. Add an arm to `complete_slash_command_argument`:

```rust
"mycommand" => Ok(mycommand_slash_command_completions(&args)),
```

4. Add an arm to `run_slash_command`:

```rust
"mycommand" => "mycommand",
```

---

## Adding a new debug config field

1. Parse the new field in `get_dap_binary` from `user_config`:

```rust
let my_field = user_config
    .get("myField")
    .and_then(|v| v.as_str())
    .unwrap_or("default");
```

2. Inject it into `config_map`:

```rust
config_map["myField"] = json!(my_field);
```

3. Document it in `debug_adapter_schemas/Dart.json` and in the README field table.

---

## Zed extension API reference

The extension uses `zed_extension_api = "0.7.0"`. Key types:

| Type | Use |
|------|-----|
| `Worktree` | Access workspace files, env, PATH resolution |
| `DebugAdapterBinary` | Return value of `get_dap_binary` |
| `DebugConfig` | Input to `dap_config_to_scenario` |
| `DebugScenario` | Return value of `dap_config_to_scenario` |
| `DebugRequest` | `Launch(LaunchRequest)` or `Attach(AttachRequest)` |
| `LaunchRequest` | `program`, `cwd`, `args` |
| `SlashCommand` | Command name passed to slash command handlers |
| `SlashCommandOutput` | `text` + `sections` returned by `run_slash_command` |
| `LspSettings` | Read LSP binary/settings from Zed config |

Full API docs: [docs.rs/zed_extension_api/0.7.0](https://docs.rs/zed_extension_api/0.7.0)

---

## What cannot be implemented (API gaps)

These features require Zed APIs that do not exist in v0.7.0:

| Feature | Missing API |
|---------|-------------|
| Hot reload / hot restart | Send custom message to running debug session |
| Flutter Outline panel | Custom tree-view / secondary panel |
| Dart DevTools | Browser launch / webview |
| Coverage gutters | Gutter decoration |
| Widget inspector | Custom panel |
| Snippet completions | Extension snippet API |
| Wrap with Widget / extract method | Custom code action provider |

See `PORTING_PLAN.md` for full status.

---

## Version and Release Process

This project follows [semantic versioning](https://semver.org/).

### Before releasing a new version

1. Update `version` in `extension.toml`
2. Add release notes to `CHANGELOG.md` with:
   - Version number and date
   - List of features, fixes, and breaking changes
   - Link to commit range: `https://github.com/kdbhalala/Flutter/compare/v<old>...v<new>`
3. Test the extension thoroughly (see [Testing](#testing) section)
4. Push changes and create a Git tag: `git tag v<version>`
5. Push the tag: `git push origin v<version>`

Example `CHANGELOG.md` entry:

```markdown
## [0.3.0] — 2026-05-22

### Added
- New `/fvm` slash command for FVM-managed Flutter workflows
- Support for Dart 3.2 patterns and records

### Fixed
- Device ID resolution for Android emulators

### Changed
- Improved SDK detection order for asdf/mise users

[0.3.0]: https://github.com/kdbhalala/Flutter/compare/v0.2.0...v0.3.0
```

---

## Submitting a pull request

1. Fork the repo and create a feature branch.
2. Make changes — run `cargo build --target wasm32-wasip1` and fix all errors.
3. Test in Zed via **"zed: install dev extension"** (see [Testing](#testing) section).
4. Update `CHANGELOG.md` with your changes if adding features or fixes.
5. Open a PR with a description of what changed and why.

**Testing is mandatory**: PRs for extensions that clearly don't work will be closed per Zed guidelines.

---

## License

MIT — see [LICENSE](LICENSE).
