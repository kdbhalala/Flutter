# Flutter for Zed

Flutter and Dart support for the [Zed](https://zed.dev) editor — LSP, syntax highlighting, debugging, tasks, and slash commands.

---

## Features

| Feature | Details |
|---------|---------|
| **Dart LSP** | Autocomplete, diagnostics, go-to-definition, hover, rename, find references, and code actions (Wrap with Widget/Padding/Center, Extract Widget, etc.) via Dart Analysis Server |
| **Inlay Hints** | Live type and parameter name hints for variables, parameters, function return types, and chained calls |
| **Syntax highlighting** | Full Tree-sitter Dart grammar including Dart 3 patterns, records, and sealed classes |
| **Auto-indent** | Bracket/brace/block-aware indentation for classes, functions, if/for/while, try/catch, and literals |
| **Code outline** | Classes, mixins, enums, extensions, functions, getters, setters, constructors, operator overloads |
| **Snippets** | 130+ snippets — Dart, Flutter widgets, lifecycle, BLoC, Riverpod, Provider, Freezed, go_router, json_serializable, CustomPainter, InheritedWidget |
| **Debug locator** | Auto-converts `flutter run` / `dart run` tasks to debug sessions — no manual `.zed/debug.json` needed |
| **Debugging** | Flutter and Dart debug adapter (DAP) — launch and attach, FVM, Puro, and Proto aware |
| **Tasks** | 50+ built-in tasks for Flutter, Dart, and FVM workflows including build_runner, coverage, web |
| **Runnables** | `main()`, `test()`, `testWidgets()`, `testGoldens()`, `patrolTest()`, `blocTest()`, and `group()` detected as runnable in the editor gutter |
| **Slash commands** | `/flutter`, `/dart`, `/fvm` in the Zed assistant with devtools and build runner support |
| **Version Manager support** | Project-local `.fvm` symlink, FVM, Puro, Proto, asdf, and mise auto-detected |
| **SDK auto-detection** | Finds SDK via LSP settings → local `.fvm` → Puro → Proto → PATH → `FLUTTER_ROOT` → asdf → mise |
| **Rich completions** | Typed labels for Class, Function, Constructor, Method, Property, Variable, Enum, Field, Module |

---

## Requirements

- [Flutter SDK](https://flutter.dev/docs/get-started/install) — includes `dart` and `flutter`
- **Optional:** [fvm](https://fvm.app), [puro](https://puro.dev), [proto](https://moonrepo.dev/proto), [asdf](https://asdf-vm.com), or [mise](https://mise.jdx.dev) — auto-detected

The extension automatically searches for the Dart/Flutter SDK in this priority order:

1. `dart.sdkPath` / `dart.flutterSdkPath` in Zed LSP settings
2. Project-local `.fvm/flutter_sdk/bin/` symlink
3. Global FVM (`~/.fvm/default/bin/`)
4. Puro (`~/.puro/envs/default/bin/`, `~/.puro/bin/`)
5. Proto (`~/.proto/shims/`, `~/.proto/bin/`)
6. `dart` / `flutter` on `PATH`
7. `FLUTTER_ROOT` environment variable
8. `~/.asdf/shims/` (asdf shims)
9. `~/.local/share/mise/shims/` (mise shims)

---

## Installation

Search **"Flutter"** in Zed's extension panel (`cmd+shift+x`) or add to `~/.config/zed/settings.json`:

```json
{
  "auto_install_extensions": {
    "flutter": true
  }
}
```

---

## LSP Settings

Configure the Dart language server in your Zed `settings.json`:

```json
{
  "lsp": {
    "dart": {
      "settings": {
        "dart": {
          "lineLength": 120,
          "enableSdkFormatter": true,
          "completeFunctionCalls": true,
          "closingLabels": true,
          "inlayHints": {
            "showForParameters": true,
            "showForVariableTypes": true,
            "showForFunctionReturnTypes": true,
            "showForChainedCalls": true
          }
        }
      }
    }
  }
}
```

### Notable LSP settings

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `dart.sdkPath` | string | — | Path to Dart SDK root — used for both LSP binary and DAP resolution |
| `dart.flutterSdkPath` | string | — | Path to Flutter SDK root |
| `dart.env` | object | `{}` | Extra environment variables injected into the LSP process and debug adapter |
| `dart.lineLength` | number | `80` | Formatter line length |
| `dart.enableSdkFormatter` | bool | `true` | Use the SDK formatter |
| `dart.completeFunctionCalls` | bool | `true` | Insert argument placeholders on completion |
| `dart.closingLabels` | bool | `true` | Render closing comments for nested widget trees |
| `dart.inlayHints.showForParameters` | bool | `true` | Show parameter name inlay hints |
| `dart.inlayHints.showForVariableTypes` | bool | `true` | Show variable type inlay hints |
| `dart.inlayHints.showForFunctionReturnTypes` | bool | `true` | Show return type inlay hints |
| `dart.inlayHints.showForChainedCalls` | bool | `true` | Show chained method call type hints |

All `dart.*` settings are forwarded verbatim to the Dart Analysis Server.

---

## Code Actions & Refactoring

Press **`cmd+.`** (macOS) or **`ctrl+.`** (Linux/Windows) on any Flutter widget or Dart identifier to access native Dart Analysis Server code actions:
- **Wrap with Widget / Padding / Center / Container / Builder / StreamBuilder**
- **Extract Widget / Extract Method**
- **Remove Widget**
- **Move Widget Up / Move Widget Down**
- **Apply `dart fix` recommendations**

## Debugging

Add debug configurations to `.zed/debug.json` in your project root or launch directly from Zed's task runner.

### Flutter — Launch

```json
[
  {
    "label": "Flutter (debug)",
    "adapter": "Dart",
    "type": "flutter",
    "request": "launch",
    "program": "lib/main.dart",
    "device_id": "chrome"
  }
]
```

### Flutter — Attach to running app

```json
[
  {
    "label": "Flutter (attach)",
    "adapter": "Dart",
    "type": "flutter",
    "request": "attach",
    "vmServiceUri": "http://127.0.0.1:8181/"
  }
]
```

### Dart CLI

```json
[
  {
    "label": "Dart CLI",
    "adapter": "Dart",
    "type": "dart",
    "request": "launch",
    "program": "bin/main.dart"
  }
]
```

### Supported debug config fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `adapter` | string | — | Must be `"Dart"` |
| `type` | string | — | `"flutter"` or `"dart"` |
| `request` | string | `"launch"` | `"launch"` or `"attach"` |
| `program` | string | `"lib/main.dart"` | Entrypoint file |
| `cwd` | string | workspace root | Working directory |
| `args` | string[] | `[]` | Arguments passed to the program |
| `additionalArgs` | string[] | `[]` | Extra args appended after `args` |
| `device_id` | string | — | Device ID or platform alias (e.g. `"chrome"`, `"ios"`, `"android"`) |
| `useFvm` | bool | `false` | Use FVM-managed Flutter |
| `flutterMode` | string | `"debug"` | `"debug"`, `"profile"`, or `"release"` |
| `debugSdkLibraries` | bool | `false` | Step into SDK source |
| `debugExternalPackageLibraries` | bool | `false` | Step into pub package source |
| `webRenderer` | string | — | `"html"`, `"canvaskit"`, `"skwasm"` |
| `vmServiceUri` | string | — | VM service URI for attach |
| `stopOnEntry` | bool | `false` | Break on first line |

---

## Tasks

Run via `cmd+shift+p` → **"task: spawn"**.

### Flutter Tasks

| Task | Command |
|------|---------|
| flutter: run | `flutter run` |
| flutter: run (release) | `flutter run --release` |
| flutter: run -d chrome | `flutter run -d chrome` |
| flutter: test | `flutter test` |
| flutter: test --coverage | `flutter test --coverage` |
| flutter: clean | `flutter clean` |
| flutter: pub get | `flutter pub get` |
| flutter: pub upgrade | `flutter pub upgrade` |
| flutter: pub run build_runner build | `flutter pub run build_runner build --delete-conflicting-outputs` |
| flutter: pub run build_runner watch | `flutter pub run build_runner watch --delete-conflicting-outputs` |
| flutter: build apk | `flutter build apk` |
| flutter: build appbundle | `flutter build appbundle` |
| flutter: build ios | `flutter build ios` |
| flutter: build web | `flutter build web` |

### Dart Tasks

| Task | Command |
|------|---------|
| dart: run $ZED_FILE | `dart run <current file>` |
| dart: test $ZED_STEM | `dart test <current file>` |
| dart: format | `dart format .` |
| dart: analyze | `dart analyze` |
| dart: fix | `dart fix --apply` |
| dart: pub get | `dart pub get` |
| dart: pub upgrade | `dart pub upgrade` |
| dart: build_runner build | `dart run build_runner build --delete-conflicting-outputs` |
| dart: build_runner watch | `dart run build_runner watch --delete-conflicting-outputs` |

---

## Slash Commands in Assistant

Use in the Zed AI assistant panel:
- **`/flutter <subcommand>`**: Token completions for `run`, `test`, `devices`, `doctor`, `devtools`, `pub`, `clean`, `build`, etc.
- **`/dart <subcommand>`**: Token completions for `run`, `test`, `analyze`, `format`, `fix`, `pub`, `devtools`, `compile`, etc.
- **`/fvm flutter <subcommand>`**: Runs FVM-managed Flutter commands with full completion support.

---

## License

MIT — see [LICENSE](LICENSE).
