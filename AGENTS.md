# CLAUDE.md & AGENTS.md

Guidance and rules for agents in this repo. `AGENTS.md` and `CLAUDE.md` are byte-identical — edit both together.

<!-- CodeGraph -->
## CodeGraph

**IMPORTANT: This project is indexed by CodeGraph (`.codegraph/`). ALWAYS reach
for CodeGraph BEFORE using Grep/Glob/Read to explore the codebase.** It is
faster, cheaper (fewer tokens), and gives structural context (callers,
dependents, blast radius) that file scanning cannot.

Available both ways — use whichever is loaded:

- **MCP tool**: `codegraph_explore`, `codegraph_node`
- **Shell**: `codegraph explore "<question>"`, `codegraph node <symbol>`

### When to use CodeGraph FIRST

- **Exploring code**: `codegraph explore "<question or symbol names>"` instead of Grep
- **Understanding impact**: `codegraph impact <symbol>` instead of manually tracing callers
- **Reading one symbol**: `codegraph node <symbol>` for its source plus caller/callee trail
- **Finding relationships**: `codegraph callers` / `codegraph callees`
- **Locating a symbol**: `codegraph query <search>`

Fall back to Grep/Glob/Read for literal string sweeps, and for the files
CodeGraph does not index (see below).

### Key commands

| Command | Use when |
| --------- | ---------- |
| `codegraph explore "<query>"` | Default first move — relevant symbols' source + call paths in one shot |
| `codegraph node <symbol\|file>` | One symbol's source + caller/callee trail, or a file read line-numbered |
| `codegraph impact <symbol>` | Blast radius before changing a shared function |
| `codegraph callers` / `callees` | Tracing one direction of the call graph |
| `codegraph query <search>` | Finding a symbol by name or keyword |
| `codegraph files` / `status` | Indexed file structure / index statistics |

### Scope in this repo

CodeGraph indexes `src/lib.rs` and `.github/workflows/ci.yml` — that is the
whole of the executable surface. The Tree-sitter queries (`languages/dart/*.scm`),
snippets, `tasks.json`, `extension.toml`, and the debug adapter schema are
declarative data with no symbols, so read those files directly.

### Workflow

1. The index auto-syncs — `codegraph serve --mcp` runs a file watcher. Run
   `codegraph sync` by hand only if the index looks stale.
2. Start with `codegraph explore`; it usually replaces several narrower calls.
3. Use `codegraph impact` before editing anything with more than one caller.
