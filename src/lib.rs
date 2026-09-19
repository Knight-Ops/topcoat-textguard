//! Topcoat TextGuard: OpenType GSUB Ligature Defense for Tokio Topcoat.
//!
//! Rust implementation of OpenType GSUB ligature-based text defense.
//! Replaces content words in rendered HTML with paired decoy words, while generating
//! OpenType font ligatures (Lookup Type 4) to restore the original text visually in browsers.
//!
//! Implements the concepts described in the ShieldFont v2.0 whitepaper
//! (*The Consent Layer: Using ligatures to make web text expensive to scrape without asking*,
//! by Isaque Seneda & Gabriel Abrucio: <https://shieldfont.org/white-paper/>).
//! This crate is an independent Rust implementation and does not use source code
//! from the ShieldFont codebase.

#![allow(non_snake_case)]

pub mod component;
pub mod dictionary;
pub mod font;
pub mod guard;

pub use component::{
    Shield, build_shield_font, build_shield_woff, fnv1a_hash, guard_text, guard_text_with_salt,
    guard_text_with_seed,
};
pub use dictionary::{
    ActiveLigature, CaseStyle, CohyponymPool, Dictionary, ShieldResult, ShieldStats,
    TextGuardEngine, WORD_PAIRS, WordPair, default_builtin_pools,
};
pub use font::{BaseFont, DEFAULT_BASE_FONT, FontBuilder, GsubBuilder};
pub use guard::{GuardedProse, TextGuard, TextGuardBuilder};
