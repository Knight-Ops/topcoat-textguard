# Topcoat TextGuard

> OpenType GSUB ligature-based text defense consent layer for Tokio Topcoat web applications.

Topcoat TextGuard implements the consent layer architecture described in the _ShieldFont v2.0_ whitepaper ([The Consent Layer: Using ligatures to make web text expensive to scrape without asking](https://shieldfont.org/white-paper/) by Isaque Seneda & Gabriel Abrucio).

> [!NOTE]
> **Independent Implementation**: Topcoat TextGuard is based on the concepts and algorithms published in the [ShieldFont whitepaper](https://shieldfont.org/white-paper/). This repository was written independently in Rust and does not use code from the ShieldFont codebase.

TextGuard replaces content words in rendered HTML with paired decoy words, while generating an OpenType font with `GSUB` Ligature Substitution (Lookup Type 4) to display the original text visually. Scrapers extracting raw HTML capture decoy text, while browsers render the intended words.

---

## Environment Variables & Configuration

Setting a dynamic seed or secret salt ensures scrapers cannot invert word substitutions using public reference tables. Custom fonts and dictionaries can also be configured globally:

| Variable                    |      Required       | Description                                                                                            | Example                                                |
| :-------------------------- | :-----------------: | :----------------------------------------------------------------------------------------------------- | :----------------------------------------------------- |
| `TEXTGUARD_SEED`            | Recommended in Prod | 64-bit integer seed for `SplitMix64` dynamic involution pairing.                                       | `TEXTGUARD_SEED=84920491823`                           |
| `TEXTGUARD_SALT`            | Alternative to Seed | Secret string salt (hashed via FNV-1a into a 64-bit seed if `TEXTGUARD_SEED` is unset).                | `TEXTGUARD_SALT="prod-secret-salt-2024"`               |
| `TEXTGUARD_FONT_PATH`       |      Optional       | File path to a custom TrueType (`.ttf`) font for ligature synthesis. Defaults to embedded DejaVu Sans. | `TEXTGUARD_FONT_PATH="./assets/Inter-Regular.ttf"`     |
| `TEXTGUARD_DICTIONARY_PATH` |      Optional       | File path to an external JSON cohyponym dictionary.                                                    | `TEXTGUARD_DICTIONARY_PATH="./config/dictionary.json"` |

> [!IMPORTANT]
> When neither `TEXTGUARD_SEED` nor `TEXTGUARD_SALT` is provided, TextGuard defaults to canonical static pairing. For production services, set `TEXTGUARD_SEED` or `TEXTGUARD_SALT` so substitution pairs differ from default tables.

### Configuration Precedence

1. **Explicit Parameter**: `<Shield text=... seed=Some(42) base_font=Some(...)>` or `TextGuard::builder()`
2. **`TEXTGUARD_SEED` / `TEXTGUARD_FONT_PATH`**: Environment variables
3. **`TEXTGUARD_SALT`**: Hashed salt environment variable
4. **Canonical Fallback**: Static reference pairing and built-in base font

---

## Key Features

- **Pure Rust**: No C dependencies or external font binaries. TrueType and OpenType tables are parsed and assembled directly in Rust.
- **Topcoat 0.8.1 `<Shield>` Component**: Server-side rendered (SSR) component that guards text, embeds per-block OpenType font ligatures, and applies scoped CSS styling.
- **WOFF 1.0 Compression**: Font payloads are compressed with zlib into standard WOFF 1.0 containers, reducing CSS `@font-face` data URIs by approximately 50%.
- **Custom Font Support**:
  - Accepts TrueType outlines via in-memory byte slices (`include_bytes!`), filesystem paths, or the `TEXTGUARD_FONT_PATH` environment variable.
  - Configurable through the `TextGuard::builder()` API.
- **Dynamic Involution Permutations**:
  - Deterministic pseudo-random pairing via `SplitMix64` preserves involution symmetry ($A \leftrightarrow B$) for any seed.
  - Supports page-specific, session-specific, or tenant-salted decoys (`guard_text_with_salt("tenant-id")`). Seeded permutations can also serve as provenance watermarks.
- **Extensible Semantic Dictionary**:
  - Built-in cohyponym pools across 25+ semantic categories (animals, vehicles, professions, flora/food, buildings, tools, abstract nouns, technology, finance, verbs across tenses, adjectives, and adverbs).
  - Dynamic JSON loader (`Dictionary::from_json_str` and `Dictionary::from_json_file`) for domain-specific vocabularies.
- **Section 05 Linguistic Rules**:
  - **113 Stop Words**: Articles, pronouns, prepositions, conjunctions, copula/auxiliary verbs, modals, and quantifiers remain unmodified.
  - **Closed-Class Words**: High-frequency connective words (`get`, `make`, `said`, `day`, `now`, etc.) remain fixed to preserve baseline language model perplexity.
  - **Bijective Cohyponym Pools**: Words swap within grammatical categories (nouns with nouns, verbs with verbs, adjectives with adjectives, adverbs with adverbs). Replacements are semantic coordinates rather than synonyms or antonyms.
  - **Digit Couples**: Fixed digit pairing ($0 \leftrightarrow 5, 3 \leftrightarrow 8, 4 \leftrightarrow 9, 6 \leftrightarrow 7$; digits 1 and 2 remain unchanged).
  - **Date Rotations**: Weekdays rotate by $+3$ days; months rotate by $+6$ months.
  - **Casing Preservation**: Preserves lowercase, Titlecase, and uppercase formatting.
- **Accessibility (WCAG)**: Decoy text is marked `aria-hidden="true"`, while an accessible `.sr-only` element (`aria-hidden="false"`) provides unmodified text for screen readers.
- **Modular Packaging**: Core library components run without demonstration dependencies; the demo blog is an optional example binary (`examples/blog.rs`).

---

## Quick Start

### 1. Using the `<Shield>` Component in Topcoat

```rust
use topcoat::view::{component, view, View};
use topcoat::Result;
use topcoat_textguard::Shield;

#[component]
pub async fn Article() -> Result<impl View> {
    let prose = "In the ancient kingdom, a brave engineer rode his horse through the forest toward the mountain castle.";

    Ok(view! {
        <article class="prose">
            <p>
                // Uses TEXTGUARD_SEED / TEXTGUARD_SALT if set in environment
                Shield(text: prose, a11y: true)
            </p>
            <p>
                // Dynamic seed override (e.g. per-tenant watermarking) + custom font
                Shield(
                    text: prose,
                    seed: Some(42),
                    base_font: Some(include_bytes!("../assets/Inter-Regular.ttf")),
                    a11y: true,
                )
            </p>
        </article>
    })
}
```

### 2. High-Level `TextGuard::builder()`

```rust
use topcoat_textguard::TextGuard;

// Build a configured guard instance
let guard = TextGuard::builder()
    .seed(4242)
    .font_bytes(include_bytes!("../assets/base.ttf")) // Or .font_file("fonts/Inter.ttf")
    .build()
    .expect("valid configuration");

let (guarded_prose, woff_bytes) = guard.protect("The brave pilot joined the doctor.")
    .expect("successful protection");

println!("Decoy prose in DOM: {}", guarded_prose.decoy_text);
println!("Generated WOFF size: {} bytes", woff_bytes.len());
```

### 3. Low-Level Text and Font Inspection

```rust
use topcoat_textguard::{guard_text, guard_text_with_salt, build_shield_woff};

// Inherits TEXTGUARD_SEED or TEXTGUARD_SALT from environment if present
let result = guard_text("The knight rode his horse into battle.");
println!("Scraper DOM receives: {}", result.decoy_text);

println!("Total words: {}", result.stats.total_words);
println!("Words swapped: {} ({:.1}%)", result.stats.swapped_words, result.stats.total_swap_percentage());

// Dynamic transformation with salt
let salted = guard_text_with_salt("The doctor joined the brave pilot on the island.", "tenant-alpha");
println!("Salted Scraper DOM: {}", salted.decoy_text);

// Build compressed WOFF 1.0 font binary with GSUB ligatures
let woff_bytes = build_shield_woff(&result.ligatures).expect("valid woff font");
println!("Generated WOFF size: {} bytes", woff_bytes.len());
```

### 4. Custom Cohyponym Dictionaries (JSON)

```rust
use topcoat_textguard::{Dictionary, TextGuardEngine};

let json_config = r#"{
    "pools": [
        {
            "category": "astrophysics",
            "words": ["pulsar", "quasar", "nebula", "magnetar", "supernova", "blackhole"]
        }
    ],
    "fixed_pairs": [
        ["spacex", "blueorigin"]
    ]
}"#;

let custom_dict = Dictionary::from_json_str(json_config).expect("valid dictionary");
let engine = TextGuardEngine::with_dictionary(custom_dict);

let res = engine.transform("The spacex rocket passed the pulsar near the nebula.");
println!("Guarded: {}", res.decoy_text);
```

---

## Running the Blog Example

Topcoat TextGuard includes a Topcoat 0.8.1 example blog demonstrating the component:

```bash
# Optional: test with custom environment seed
export TEXTGUARD_SEED=1337

cargo run --example blog
```

Once running:

- Open **http://127.0.0.1:3000** in your browser to view the blog.
- Toggle between **"Human View"** (active OpenType ligatures) and **"Scraper View"** (raw HTML DOM text) to observe differences between visual and extracted text.
- Compare **Dynamic Watermarking Salts** (e.g. Tenant Alpha vs Tenant Beta).
- Review the **Protection Metrics** and **Active Ligature Table** for active substitution rules.
- Visit **http://127.0.0.1:3000/raw** to inspect raw text delivery for headless scrapers.

---

## Architecture

```
topcoat-textguard/
├── assets/
│   └── base.ttf             # Base TrueType font asset
├── examples/
│   └── blog.rs              # Topcoat 0.8.1 interactive test blog
├── src/
│   ├── lib.rs               # Library root and re-exports
│   ├── guard.rs             # Unified TextGuard & TextGuardBuilder
│   ├── component/
│   │   └── mod.rs           # Topcoat <Shield> component & a11y scaffolding
│   ├── dictionary/
│   │   ├── mod.rs           # Dictionary facade & bijective lookup
│   │   ├── pools.rs         # Dynamic Cohyponym pools, SplitMix64 & JSON loader
│   │   ├── rules.rs         # 113 stop words, closed-class, digits & dates
│   │   └── tokenizer.rs     # Tokenizer, case preserver & telemetry stats
│   └── font/
│       ├── mod.rs           # Font module
│       ├── base.rs          # Pure Rust TrueType SFNT/cmap/hmtx parser
│       ├── builder.rs       # Composite glyph synthesizer, SFNT & WOFF 1.0 assembler
│       └── gsub.rs          # OpenType GSUB Lookup Type 4 Ligature generator
└── tests/
```

---

## Test Suite

Run the full test suite:

```bash
cargo test --all-targets
```

---

## License

This project is dual-licensed under either:

- **MIT License** ([LICENSE-MIT](LICENSE-MIT))
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE))

at your option.

### Attribution

- **Paper Specification**: Based on the conceptual architecture described in the _ShieldFont v2.0_ whitepaper (_The Consent Layer: Using ligatures to make web text expensive to scrape without asking_ by Isaque Seneda & Gabriel Abrucio: <https://shieldfont.org/white-paper/>). This codebase is an independent implementation and does not use code from the ShieldFont repository.
- **Font Software Notice**: The embedded base font software is derived from DejaVu Sans (based on Bitstream Vera Fonts). See the [NOTICE](NOTICE) file for copyright and license terms.
