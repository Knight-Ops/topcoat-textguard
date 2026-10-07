# Agent Guidelines: Topcoat TextGuard

This repository is equipped with **`codebase-memory-mcp`**, a high-precision knowledge graph containing pre-indexed ASTs, LSP call graphs, type references, and architectural clusters.

All AI agents working on this codebase are instructed to prioritize the `codebase-memory-mcp` graph tools for code discovery, relationship navigation, impact analysis, and architecture exploration before resorting to brute-force file scanning or text-based search.

---

## 1. MCP Project Configuration

When calling `codebase-memory-mcp` tools, always use this project identifier:

- **Project Identifier**: `topcoat-textguard`
- **Repository Path**: `.`

---

## 2. Tool Selection & Hierarchy

Always follow this operational hierarchy:

| Task / Intent                           | Primary Tool (`codebase-memory-mcp`)                                                                | Fallback / Complementary  |
| :-------------------------------------- | :-------------------------------------------------------------------------------------------------- | :------------------------ |
| **Orientation & Architecture**          | `get_architecture(project="...", aspects=["overview"])` or `["clusters", "boundaries", "hotspots"]` | `README.md`               |
| **Locating Functions, Structs, Traits** | `search_graph(project="...", query="..."                                                            | name_pattern="...")`      | `grep_search`     |
| **Reading Definitions / Symbols**       | `get_code_snippet(project="...", qualified_name="...")`                                             | `client_view_file`        |
| **Callers, Callees & Blast Radius**     | `trace_path(project="...", function_name="...", mode="calls")`                                      | Manual search / grep      |
| **Data Propagation & Flow**             | `trace_path(project="...", function_name="...", mode="data_flow", parameter_name="...")`            | Manual code trace         |
| **Complex Relationship Queries**        | `query_graph(project="...", query="...")`                                                           | Multi-file exploration    |
| **Coverage & Absence Verification**     | `check_index_coverage(project="...", paths=[...])`                                                  | File system check         |
| **Detecting Drift / Modified Code**     | `detect_changes(project="...", repo_path="...")`                                                    | `git status` / `git diff` |
| **Architecture Records**                | `manage_adr(project="...", mode="get"                                                               | "update")`                | Architecture docs |

> **RULE**: Do NOT use `grep_search` or exhaustive file reads for structural queries (finding definitions, usages, callers, or implementations). Always start with `search_graph`, `trace_path`, or `query_graph`. Use `grep_search` only for string literals, comments, configuration files (e.g. `Cargo.toml`), or files confirmed as unindexed.

---

## 3. Standard Workflows

### 3.1 Initial Exploration & Architectural Orientation

When beginning work on unfamiliar components or features:

1. Call `get_architecture` with `aspects=["overview"]` to inspect node distribution, module layers, and entry points.
2. Call `get_architecture` with `aspects=["clusters"]` to inspect Leiden-detected functional modules and cohesion scores.
3. Call `get_architecture` with `aspects=["hotspots"]` to identify high fan-in core nodes (e.g., `TextGuardEngine::new`, `FontBuilder::build_font`).

### 3.2 Symbol Discovery & Code Retrieval

1. Search for symbols with `search_graph`:
   - Natural language: `search_graph(query="transform text decoy", project="...")`
   - Pattern match: `search_graph(name_pattern=".*FontBuilder.*", project="...")`
   - Semantic query: `search_graph(semantic_query=["ligature", "opentype"], project="...")`
2. Retrieve the exact implementation with `get_code_snippet`:
   - `get_code_snippet(qualified_name="<qualified_name_from_search>", project="...")`

### 3.3 Refactoring & Impact Analysis

Before modifying functions, structs, or methods:

1. Run `trace_path` in `direction="inbound"` (or `"both"`) to see all upstream callers across the crate:
   ```json
   {
     "project": "topcoat-textguard",
     "function_name": "guard_text",
     "mode": "calls",
     "direction": "inbound"
   }
   ```
2. Verify if tests cover the target: `trace_path` with `include_tests=true`.
3. Check data propagation paths with `mode="data_flow"` if altering argument signatures or return types.

### 3.4 Verification & Coverage

1. Before asserting that a method, struct, or file does not exist, run `check_index_coverage(project="...", paths=[...])` to ensure the target scope was not skipped or partially parsed.
2. If fresh files were added outside MCP watch, check `index_status` or run `index_repository(repo_path=".")`.

---

## 4. Codebase Context

- **Language**: Rust (Edition 2024)
- **Framework**: Topcoat (`topcoat = "0.10.0"`), Tokio, Serde
- **Purpose**: Web anti-scraping and consent layer that replaces DOM text with decoy tokens while applying generated OpenType GSUB ligatures so human readers see authentic prose on screen.
- **Key Modules**:
  - `src/component/`: UI and DOM component bindings (`Shield`, `guard_text`).
  - `src/dictionary/`: Tokenizer, word substitution rules, and decoy pools (`TextGuardEngine`).
  - `src/font/`: Font parsers (`BaseFont`), ligature generator (`GsubBuilder`), composite glyph and WOFF serializer (`FontBuilder`).
  - `src/guard.rs`: Unified `TextGuard` engine and `TextGuardBuilder`.
  - `examples/`: End-to-end interactive demo application (`blog.rs`).
- **Guidelines**:
  - Keep font generation pure Rust with zero native C dependencies.
  - Test font outputs against HarfBuzz / standard OpenType format definitions.
  - Follow linguistic rules: preserve capitalization, frozen stopwords, and bijective semantic pairing.

<!-- scry:start -->
# Code Intelligence (Scry MCP & CLI)

This repository is indexed by **Scry**, a Tree-Sitter and Tree-sitter Stack Graphs code intelligence server and CLI.
Scry provides deterministic AST retrieval, call graph analysis, and cross-file definition resolution across Rust, Python, and TypeScript.

## Navigation Guidelines & Exploration Best Practices

- **Work Outside-In with AST Outlines**: When exploring unfamiliar code, run `get_file_outline` first to inspect structural declarations and exact line bounds without loading implementation bodies.
- **Targeted Line Reading**: Once you identify the target symbol or function from an outline or definition jump, read only that specific slice or function body to understand or edit logic.
- **Symbol & Navigation Priority**: Do NOT use `grep` or `ripgrep` as the primary way to locate symbol definitions, callers, or references in code—use `resolve_definition`, `find_references`, and `get_file_outline` first.
- **Avoid Loading Full Source Files for Scouting**: Do NOT dump entire large source files into context to search for symbols. Reserve whole-file reading for configs, documentation, tests, or when you explicitly need the full file context.
- **Verify Contracts**: Do NOT guess complex struct fields, enum variants, or method signatures when writing or refactoring code; inspect them with `get_type_contract`.
- **Grep for Non-Structural Searches**: Use `grep` or text search for string literals, error logs, user-facing messages, comments, environment variables, non-code files (Markdown, JSON, YAML, TOML, Dockerfiles), or as a fallback when AST indexing does not resolve a symbol.

## Invocation Modes (Choose the mode supported by your environment)

### 1. Direct MCP Tools (Codex, Cursor, Claude Code, Windsurf)
If Scry MCP tools are available directly in your agent environment, invoke them by name:
- `get_file_outline(file_path: "src/main.rs")`
- `resolve_definition(file_path: "src/main.rs", line: 42, column: 10)`
- `find_references(symbol: "MySymbol")`
- `get_type_contract(type_name: "MyType")`

### 2. MCP Gateway Tools (Google Antigravity / Gemini CLI)
In Antigravity, Scry tools are lazily loaded. Dispatch all tool calls through the `call_mcp_tool` gateway:
```json
call_mcp_tool(
  ServerName: "Scry",
  ToolName: "get_file_outline",
  Arguments: { "file_path": "src/main.rs" }
)
```

### 3. CLI Subcommands & Shell Fallback (Terminal / Non-MCP Environments)
If your agent operates via shell commands or lacks direct MCP tooling, run Scry tools directly via the `scry` CLI binary:
```bash
# Discover all tools and flags
scry --help
scry get-file-outline --help

# Direct tool invocations (kebab-case or snake_case)
scry get-file-outline src/main.rs
scry get-file-outline --file-path src/main.rs --compact
scry get-type-contract MyType
scry resolve-definition src/main.rs 42 10
scry find-references MySymbol
scry trace-call-hierarchy my_func --direction callers
scry list-projects
```

## Tool Selection Priority
1. `get_file_outline` (`scry get-file-outline`) — Inspect functions, structs, traits, enums, and signatures in a file without reading implementation bodies.
2. `get_type_contract` (`scry get-type-contract`) — View fields, enum variants, and trait implementations for a specific type.
3. `get_enclosing_scope` (`scry get-enclosing-scope`) — Find the smallest enclosing function, impl, or type at a given file and line.
4. `resolve_definition` (`scry resolve-definition`) — Jump directly to a symbol's definition across files using precise line/column coordinates.
5. `find_references` (`scry find-references`) — Find all usages of a symbol in the active project; set `scope_level: "dependencies"` to track workspace usages of external symbols, or `"workspace_all"` across all projects.
6. `get_symbol_docs` (`scry get-symbol-docs`) — Inspect public documentation, signatures, and code examples for external crate or workspace symbols.
7. `get_crate_outline` (`scry get-crate-outline`) — Orient yourself in an external crate's public API structure.
8. `search_dependency_symbols` (`scry search-dependency-symbols`) — Find unknown or partially-named types, traits, or functions across dependencies.
9. `trace_call_hierarchy` (`scry trace-call-hierarchy`) — Trace inbound callers or outbound callees of a function or method (up to depth 5).
10. `calculate_blast_radius` (`scry calculate-blast-radius`) — Assess downstream impact, dependents, and affected files before refactoring or deleting symbols.
11. `query_adrs` / `get_symbol_invariants` (`scry query-adrs`) — Search Architectural Decision Records and design invariants.
12. `record_adr` / `delete_adr` — Record or remove Architectural Decision Records in `docs/adr/`.

## Project & Index Management
- `list_projects` (`scry list-projects`) / `switch_active_project` — List registered projects and change the session's active project.
- `index_workspace` (`scry index`) / `get_indexing_status` (`scry status`) — Trigger an incremental or full re-index and check scanner progress. Pass `project: "0"` or `"dependencies"` to inspect the global Cargo dependency cache.
- `delete_project` (`scry delete`) — Unregister a project and delete its index data (source files are not touched).

## Payload Budgets & Selective Querying
- Responses enforce a default 1,000-token budget to protect context windows.
- To receive full output without truncation, pass `no_truncate: true` or specify a custom `max_tokens: N`.
- Prefer narrower queries and pagination over large dumps:
  - `get_indexing_status`: `project` (e.g. `"0"` or `"dependencies"` for the external Cargo cache).
  - `get_file_outline`: `query`, `kinds` (e.g. `["struct"]`), `start_line`/`end_line`, `compact: true`, `offset`/`limit`.
  - `find_references`: `file_filter`, `role`, `scope_level`, `offset`/`limit`.
  - `get_crate_outline`: `module_path`, `version`.
  - `search_dependency_symbols`: `crate_name`, `kind`, `offset`/`limit`.
  - `trace_call_hierarchy`: `file_filter`, `limit`.
  - `calculate_blast_radius`: `include_symbols: false` (to inspect affected files only), `limit`.
  - `query_adrs` / `get_symbol_invariants`: `status`, `compact: true`, `offset`/`limit`.
<!-- scry:end -->
