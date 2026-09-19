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
- **Framework**: Topcoat (`topcoat = "0.8.1"`), Tokio, Serde
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
