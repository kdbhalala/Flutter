# Flutter for Zed

Flutter and Dart support for the [Zed](https://zed.dev) editor.

## Features

- **Dart LSP** — autocomplete, diagnostics, go-to-definition, hover, rename, find references, code actions via the Dart Analysis Server
- **Syntax highlighting** — full Tree-sitter grammar for Dart
- **Auto-indent** — bracket and brace aware indentation
- **Code outline** — classes, functions, getters, setters, enums, extensions in the outline panel
- **Debugging** — Flutter/Dart debug adapter (DAP) with launch and attach support
- **Tasks** — 19 built-in tasks for common Flutter and Dart workflows
- **FVM support** — all tasks and the debug adapter respect `useFvm` for Flutter Version Management

## Requirements

- [Flutter SDK](https://flutter.dev/docs/get-started/install) (includes `dart` and `flutter` CLI)
- Optional: [fvm](https://fvm.app) for Flutter Version Management

## Tasks

Run via `task: spawn` (`cmd+shift+p` → "task: spawn"):

| Task | Command |
|------|---------|
| flutter: run | `flutter run` |
| flutter: run (release) | `flutter run --release` |
| flutter: pub get | `flutter pub get` |
| flutter: pub upgrade | `flutter pub upgrade` |
| flutter: build apk | `flutter build apk` |
| flutter: build ios | `flutter build ios` |
| flutter: build web | `flutter build web` |
| flutter: test | `flutter test` |
| flutter: test $file | `flutter test <current file>` |
| flutter: clean | `flutter clean` |
| flutter: devices | `flutter devices` |
| flutter: doctor | `flutter doctor` |
| dart: run $file | `dart run <current file>` |
| dart: test $file | `dart test <current file>` |
| dart: format | `dart format .` |
| dart: analyze | `dart analyze` |
| fvm: flutter run | `fvm flutter run` |
| fvm: flutter pub get | `fvm flutter pub get` |
| fvm: flutter test | `fvm flutter test` |

## Debugging

Add a debug configuration to your workspace `.zed/debug.json`:

```json
[
  {
    "label": "Flutter (Chrome)",
    "adapter": "Dart",
    "type": "flutter",
    "request": "launch",
    "program": "lib/main.dart",
    "device_id": "chrome",
    "platform": "web"
  },
  {
    "label": "Flutter (iOS Simulator)",
    "adapter": "Dart",
    "type": "flutter",
    "request": "launch",
    "program": "lib/main.dart",
    "device_id": "ios",
    "platform": "ios"
  },
  {
    "label": "Flutter (Android)",
    "adapter": "Dart",
    "type": "flutter",
    "request": "launch",
    "program": "lib/main.dart",
    "device_id": "android"
  },
  {
    "label": "Dart CLI",
    "adapter": "Dart",
    "type": "dart",
    "request": "launch",
    "program": "bin/main.dart"
  }
]
```

### FVM

Add `"useFvm": true` to any debug configuration to use the project's pinned Flutter version:

```json
{
  "label": "Flutter (FVM)",
  "adapter": "Dart",
  "type": "flutter",
  "request": "launch",
  "program": "lib/main.dart",
  "useFvm": true
}
```

## LSP Settings

Override the Dart LSP binary or pass custom settings via your Zed `settings.json`:

```json
{
  "lsp": {
    "dart": {
      "binary": {
        "path": "/path/to/dart",
        "arguments": ["language-server", "--protocol=lsp"]
      },
      "settings": {
        "lineLength": 120,
        "enableSdkFormatter": true,
        "completeFunctionCalls": true
      }
    }
  }
}
```

## License

MIT
