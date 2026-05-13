# Flutter for Zed

Flutter and Dart support for the [Zed](https://zed.dev) editor — LSP, syntax highlighting, debugging, tasks, and slash commands.

---

## Features

| Feature | Details |
|---------|---------|
| **Dart LSP** | Autocomplete, diagnostics, go-to-definition, hover, rename, find references, code actions via Dart Analysis Server |
| **Syntax highlighting** | Full Tree-sitter Dart grammar including Dart 3 patterns, records, sealed classes |
| **Auto-indent** | Bracket/brace/block-aware indentation for classes, functions, if/for/while, try/catch, literals |
| **Code outline** | Classes, mixins, enums, extensions, functions, getters, setters, constructors, operator overloads |
| **Snippets** | 120+ snippets — Dart, Flutter widgets, lifecycle, BLoC, Riverpod, Provider, Freezed, go_router, json_serializable |
| **Debug locator** | Auto-converts `flutter run` / `dart run` tasks to debug sessions — no manual `.zed/debug.json` needed |
| **Debugging** | Flutter and Dart debug adapter (DAP) — launch and attach, FVM-aware |
| **Tasks** | 50+ built-in tasks for Flutter, Dart, and FVM workflows including build_runner, coverage, web |
| **Runnables** | `main()`, `test()`, `testWidgets()`, `blocTest()`, `group()` detected as runnable |
| **Slash commands** | `/flutter`, `/dart`, `/fvm` in the Zed assistant |
| **FVM support** | Tasks, DAP, and debug locator all respect FVM for Flutter Version Management |
| **SDK auto-detection** | Finds SDK via LSP settings → PATH → `FLUTTER_ROOT` → FVM default → asdf → mise |
| **Rich completions** | Typed labels for Class, Function, Constructor, Method, Property, Variable, Enum, Field, Module |

---

## Requirements

- [Flutter SDK](https://flutter.dev/docs/get-started/install) — includes `dart` and `flutter`
- **Optional:** [fvm](https://fvm.app) for Flutter Version Management
- **Optional:** [asdf](https://asdf-vm.com) or [mise](https://mise.jdx.dev) — shims auto-detected

The extension automatically searches for the Dart/Flutter SDK in this priority order:

1. `dart.sdkPath` / `dart.flutterSdkPath` in Zed LSP settings
2. `dart` / `flutter` on `PATH`
3. `FLUTTER_ROOT` environment variable
4. `~/.fvm/default/bin/` (FVM global default)
5. `~/.asdf/shims/` (asdf shims)
6. `~/.local/share/mise/shims/` (mise shims)

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
      "binary": {
        "path": "/path/to/dart",
        "arguments": ["language-server", "--protocol=lsp"]
      },
      "settings": {
        "dart": {
          "lineLength": 120,
          "enableSdkFormatter": true,
          "completeFunctionCalls": true,
          "sdkPath": "/usr/local/flutter/bin/cache/dart-sdk",
          "flutterSdkPath": "/usr/local/flutter",
          "env": {
            "PUB_CACHE": "/custom/pub/cache"
          }
        }
      }
    }
  }
}
```

### Notable LSP settings

| Key | Type | Description |
|-----|------|-------------|
| `dart.sdkPath` | string | Path to Dart SDK root — used for both LSP binary and DAP resolution |
| `dart.flutterSdkPath` | string | Path to Flutter SDK root |
| `dart.env` | object | Extra environment variables injected into the LSP process and debug adapter |
| `dart.lineLength` | number | Formatter line length (default 80) |
| `dart.enableSdkFormatter` | bool | Use the SDK formatter |
| `dart.completeFunctionCalls` | bool | Insert argument placeholders on completion |

All `dart.*` settings are forwarded verbatim to the Dart Analysis Server.

---

## Debugging

Add debug configurations to `.zed/debug.json` in your project root.

### Flutter — launch

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

### Flutter — attach to running app

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

### FVM project

```json
[
  {
    "label": "Flutter (FVM)",
    "adapter": "Dart",
    "type": "flutter",
    "request": "launch",
    "program": "lib/main.dart",
    "useFvm": true
  }
]
```

### All supported debug config fields

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
| `platform` | string | — | Platform override |
| `useFvm` | bool | `false` | Use FVM-managed Flutter |
| `flutterMode` | string | `"debug"` | `"debug"`, `"profile"`, or `"release"` |
| `debugSdkLibraries` | bool | `false` | Step into SDK source |
| `debugExternalPackageLibraries` | bool | `false` | Step into pub package source |
| `webRenderer` | string | — | `"html"`, `"canvaskit"` — injected as `--web-renderer=<value>` |
| `vmServiceUri` | string | — | VM service URI for attach |
| `stopOnEntry` | bool | `false` | Break on first line |

### Device ID resolution

`device_id` is matched against `flutter devices --machine` output using the same fuzzy priority as VS Code Dart-Code:

1. Exact ID match
2. Exact name match
3. ID starts with value
4. Name starts with value
5. ID contains value
6. Name contains value

Common aliases: `"chrome"`, `"ios"`, `"android"`, `"macos"`, `"linux"`, `"windows"`, `"web"`.

---

## Tasks

Run via `cmd+shift+p` → **"task: spawn"**.

### Flutter

| Task | Command |
|------|---------|
| flutter: run | `flutter run` |
| flutter: run (release) | `flutter run --release` |
| flutter: run -d chrome | `flutter run -d chrome` |
| flutter: run --web-renderer canvaskit | `flutter run --web-renderer canvaskit` |
| flutter: test | `flutter test` |
| flutter: test $ZED_STEM | `flutter test <current file>` |
| flutter: test --coverage | `flutter test --coverage` |
| flutter: clean | `flutter clean` |
| flutter: upgrade | `flutter upgrade` |
| flutter: devices | `flutter devices` |
| flutter: doctor | `flutter doctor` |
| flutter: emulators | `flutter emulators` |
| flutter: attach | `flutter attach` |
| flutter: screenshot | `flutter screenshot` |
| flutter: gen-l10n | `flutter gen-l10n` |
| flutter: pub get | `flutter pub get` |
| flutter: pub upgrade | `flutter pub upgrade` |
| flutter: pub outdated | `flutter pub outdated` |
| flutter: pub run build_runner build | `flutter pub run build_runner build --delete-conflicting-outputs` |
| flutter: pub run build_runner watch | `flutter pub run build_runner watch --delete-conflicting-outputs` |
| flutter: build apk | `flutter build apk` |
| flutter: build apk --release | `flutter build apk --release` |
| flutter: build appbundle | `flutter build appbundle` |
| flutter: build ios | `flutter build ios` |
| flutter: build ios --release | `flutter build ios --release` |
| flutter: build ipa | `flutter build ipa` |
| flutter: build web | `flutter build web` |

### Dart

| Task | Command |
|------|---------|
| dart: run $ZED_FILE | `dart run <current file>` |
| dart: test $ZED_STEM | `dart test <current file>` |
| dart: test --coverage | `dart test --coverage=coverage` |
| dart: format | `dart format .` |
| dart: analyze | `dart analyze` |
| dart: fix | `dart fix --apply` |
| dart: compile exe | `dart compile exe <current file>` |
| dart: pub get | `dart pub get` |
| dart: pub upgrade | `dart pub upgrade` |
| dart: pub outdated | `dart pub outdated` |
| dart: build_runner build | `dart run build_runner build --delete-conflicting-outputs` |
| dart: build_runner watch | `dart run build_runner watch --delete-conflicting-outputs` |

### FVM

| Task | Command |
|------|---------|
| fvm: flutter run | `fvm flutter run` |
| fvm: flutter run -d chrome | `fvm flutter run -d chrome` |
| fvm: flutter test | `fvm flutter test` |
| fvm: flutter pub get | `fvm flutter pub get` |
| fvm: flutter pub upgrade | `fvm flutter pub upgrade` |
| fvm: flutter emulators | `fvm flutter emulators` |
| fvm: flutter devices | `fvm flutter devices` |
| fvm: flutter doctor | `fvm flutter doctor` |
| fvm: flutter build apk | `fvm flutter build apk` |
| fvm: flutter build ios | `fvm flutter build ios` |
| fvm: flutter build web | `fvm flutter build web` |
| fvm: flutter build appbundle | `fvm flutter build appbundle` |
| fvm: flutter clean | `fvm flutter clean` |
| fvm: flutter pub run build_runner build | `fvm flutter pub run build_runner build --delete-conflicting-outputs` |
| fvm: flutter pub run build_runner watch | `fvm flutter pub run build_runner watch --delete-conflicting-outputs` |

## Snippets

120+ snippets triggered by prefix in any `.dart` file.

### Flutter Widgets

| Prefix | Inserts |
|--------|---------|
| `stless` | StatelessWidget |
| `stful` | StatefulWidget |
| `stfulinit` | StatefulWidget + initState |
| `stcons` | ConsumerWidget (Riverpod) |
| `stconsful` | ConsumerStatefulWidget (Riverpod) |
| `scaffold` | Scaffold |
| `matapp` | MaterialApp entry point |
| `cupeapp` | CupertinoApp entry point |
| `listviewb` | ListView.builder |
| `gridviewb` | GridView.builder |
| `gvc` | GridView.count |
| `streambuilder` | StreamBuilder |
| `futurebuilder` | FutureBuilder |
| `animbuilder` | AnimatedBuilder |
| `stfulB` | StatefulBuilder |
| `layoutB` | LayoutBuilder |

### Lifecycle & Overrides

| Prefix | Inserts |
|--------|---------|
| `build` | `build()` override |
| `initstate` | `initState()` override |
| `dispose` | `dispose()` override |
| `didchange` | `didChangeDependencies()` override |
| `didupdate` | `didUpdateWidget()` override |
| `reassemble` | `reassemble()` override |

### State Management

| Prefix | Inserts |
|--------|---------|
| `blocEvent` | BLoC sealed event class |
| `blocState` | BLoC sealed state class |
| `blocClass` | BLoC class with handler |
| `blocBuilder` | BlocBuilder widget |
| `blocProvider` | BlocProvider widget |
| `changeNotifier` | ChangeNotifier class |
| `cnProvider` | ChangeNotifierProvider |
| `consumer` | Provider Consumer widget |
| `stateProvider` | Riverpod StateProvider |
| `futureProvider` | Riverpod FutureProvider |
| `streamProvider` | Riverpod StreamProvider |
| `notifierProvider` | Riverpod NotifierProvider |
| `asyncNotifierProvider` | Riverpod AsyncNotifierProvider |

### Data Classes

| Prefix | Inserts |
|--------|---------|
| `freezed` | Freezed immutable data class with fromJson/toJson |
| `freezedUnion` | Freezed sealed union type |
| `jsonser` | json_serializable annotated class |
| `equatable` | Equatable value equality class |

### Navigation (go_router)

| Prefix | Inserts |
|--------|---------|
| `goRouter` | GoRouter configuration |
| `goRoute` | Single GoRoute entry |
| `ctxGo` | `context.go('/')` |
| `ctxPush` | `context.push('/')` |

### Navigation (Navigator)

| Prefix | Inserts |
|--------|---------|
| `navpush` | `Navigator.push(...)` |
| `navnamed` | `Navigator.pushNamed(...)` |

### Utilities

| Prefix | Inserts |
|--------|---------|
| `importM` | `import 'package:flutter/material.dart'` |
| `importC` | `import 'package:flutter/cupertino.dart'` |
| `edgeall` | `EdgeInsets.all(...)` |
| `edgesym` | `EdgeInsets.symmetric(...)` |
| `snackbar` | `ScaffoldMessenger.of(context).showSnackBar(...)` |
| `dialog` | `showDialog(...)` |
| `theme` | `Theme.of(context)` |
| `mediaquery` | `MediaQuery.of(context)` |

---



Use in the Zed AI assistant panel.

### `/flutter <subcommand>`

Runs `flutter <subcommand>` in the workspace root.

Completions: `run`, `test`, `devices`, `doctor`, `attach`, `pub get`, `pub upgrade`, `pub outdated`, `emulators`, `clean`, `screenshot`, `gen-l10n`, `create`, `build apk`, `build apk --release`, `build appbundle`, `build ios`, `build ios --release`, `build ipa`, `build web`

### `/dart <subcommand>`

Runs `dart <subcommand>` in the workspace root.

Completions: `run`, `test`, `analyze`, `format .`, `fix`, `pub get`, `pub upgrade`, `pub outdated`, `create`, `compile`, `doc`

### `/fvm flutter <subcommand>`

Runs `fvm flutter <subcommand>`. First argument must be `flutter`, then accepts all Flutter subcommand completions.

---

## What is not ported (Zed API limitations)

These VS Code Dart-Code features have no equivalent Zed extension API surface yet:

| Feature | Reason |
|---------|--------|
| Hot reload / hot restart | No API to send custom messages to a running debug session |
| Flutter Outline panel | No custom tree-view or secondary panel API |
| Dart DevTools | No browser-launch or webview API |
| Coverage gutters | No gutter decoration API |
| Widget inspector | Requires custom UI panel |
| Profile / memory views | Requires custom UI panel |
| Snippet completions | ✅ Available — 120+ snippets in `snippets/dart.json` |
| Wrap with Widget / extract method | No custom code action provider API |

---

## License

MIT — see [LICENSE](LICENSE).

