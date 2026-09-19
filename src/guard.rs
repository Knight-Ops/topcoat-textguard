//! High-level unified builder and guard orchestration.

use crate::dictionary::pools::{Dictionary, hash_str};
use crate::dictionary::tokenizer::{ActiveLigature, ShieldStats, TextGuardEngine};
use crate::font::builder::FontBuilder;
use std::path::{Path, PathBuf};

/// Result of protecting text with font ligature generation.
#[derive(Debug, Clone)]
pub struct GuardedProse {
    /// Decoy text for the HTML DOM.
    pub decoy_text: String,
    /// Author original text.
    pub original_text: String,
    /// Active ligatures synthesized for the font.
    pub ligatures: Vec<ActiveLigature>,
    /// Metrics on substitutions.
    pub stats: ShieldStats,
}

impl GuardedProse {
    /// Builds an uncompressed TrueType SFNT binary with active ligatures.
    pub fn build_font<'a>(&self, builder: &FontBuilder<'a>) -> Result<Vec<u8>, String> {
        builder.build_font(&self.ligatures)
    }

    /// Builds a compressed WOFF 1.0 font binary with active ligatures.
    pub fn build_woff<'a>(&self, builder: &FontBuilder<'a>) -> Result<Vec<u8>, String> {
        builder.build_woff(&self.ligatures)
    }
}

/// A unified configured TextGuard instance containing both the substitution engine
/// and font synthesizer.
pub struct TextGuard<'a> {
    engine: TextGuardEngine,
    font_builder: FontBuilder<'a>,
}

impl<'a> TextGuard<'a> {
    /// Creates a new builder for configuring TextGuard.
    pub fn builder() -> TextGuardBuilder<'a> {
        TextGuardBuilder::new()
    }

    /// Reference to the underlying dictionary tokenizer engine.
    pub fn engine(&self) -> &TextGuardEngine {
        &self.engine
    }

    /// Reference to the underlying font builder.
    pub fn font_builder(&self) -> &FontBuilder<'a> {
        &self.font_builder
    }

    /// Transforms input prose and builds a compressed WOFF 1.0 font payload.
    pub fn protect(&self, text: &str) -> Result<(GuardedProse, Vec<u8>), String> {
        let result = self.engine.transform(text);
        let woff = self.font_builder.build_woff(&result.ligatures)?;
        let prose = GuardedProse {
            decoy_text: result.decoy_text,
            original_text: result.original_text,
            ligatures: result.ligatures,
            stats: result.stats,
        };
        Ok((prose, woff))
    }
}

/// Builder for constructing a configured `TextGuard` instance.
#[derive(Default)]
pub struct TextGuardBuilder<'a> {
    seed: Option<u64>,
    salt: Option<String>,
    dictionary: Option<Dictionary>,
    font_bytes: Option<&'a [u8]>,
    font_path: Option<PathBuf>,
}

impl<'a> TextGuardBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets an explicit deterministic numeric seed.
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Sets a secret string salt (hashed to derive a seed).
    pub fn salt(mut self, salt: &str) -> Self {
        self.salt = Some(salt.to_string());
        self
    }

    /// Sets a custom cohyponym dictionary.
    pub fn dictionary(mut self, dict: Dictionary) -> Self {
        self.dictionary = Some(dict);
        self
    }

    /// Loads a custom cohyponym dictionary from a JSON file.
    pub fn dictionary_file<P: AsRef<Path>>(mut self, path: P) -> Result<Self, String> {
        let p = path.as_ref();
        let dict = Dictionary::from_json_file(p)
            .map_err(|e| format!("Failed to read dictionary file {}: {}", p.display(), e))?;
        self.dictionary = Some(dict);
        Ok(self)
    }

    /// Uses a custom TrueType font from in-memory bytes (e.g. `include_bytes!`).
    pub fn font_bytes(mut self, bytes: &'a [u8]) -> Self {
        self.font_bytes = Some(bytes);
        self
    }

    /// Loads a custom TrueType font from a file path.
    pub fn font_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.font_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Builds the configured `TextGuard` instance.
    pub fn build(self) -> Result<TextGuard<'a>, String> {
        let dict = self.dictionary.unwrap_or_else(|| {
            if let Ok(path) = std::env::var("TEXTGUARD_DICTIONARY_PATH") {
                Dictionary::from_json_file(std::path::Path::new(&path)).unwrap_or_default()
            } else {
                Dictionary::default()
            }
        });

        let seed = if let Some(s) = self.seed {
            Some(s)
        } else if let Some(ref salt) = self.salt {
            Some(hash_str(salt))
        } else if let Ok(seed_str) = std::env::var("TEXTGUARD_SEED") {
            seed_str.trim().parse::<u64>().ok()
        } else if let Ok(salt_str) = std::env::var("TEXTGUARD_SALT") {
            Some(hash_str(salt_str.trim()))
        } else {
            None
        };

        let engine = TextGuardEngine::with_dictionary_and_seed(dict, seed);

        let font_builder = if let Some(bytes) = self.font_bytes {
            FontBuilder::from_bytes(bytes)?
        } else if let Some(ref path) = self.font_path {
            FontBuilder::from_file(path)?
        } else {
            FontBuilder::from_env()
        };

        Ok(TextGuard {
            engine,
            font_builder,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::base::DEFAULT_BASE_FONT;

    #[test]
    fn test_textguard_builder_custom_font() {
        let guard = TextGuard::builder()
            .seed(4242)
            .font_bytes(DEFAULT_BASE_FONT)
            .build()
            .expect("valid builder configuration");

        let input = "The doctor joined the brave pilot on the island.";
        let (prose, woff_bytes) = guard.protect(input).expect("protection success");

        assert_ne!(prose.decoy_text, input);
        assert!(!prose.ligatures.is_empty());
        assert!(!woff_bytes.is_empty());
        assert_eq!(&woff_bytes[0..4], b"wOFF");
    }
}
