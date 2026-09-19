//! Text tokenizer, casing preservation, and substitution engine.

use super::pools::{Dictionary, hash_str};
use super::rules::{is_forbidden_word, rotate_month, rotate_weekday, swap_digits_in_str};
use std::collections::HashMap;

/// Casing convention for words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseStyle {
    Lowercase,
    Titlecase,
    Uppercase,
    Mixed,
}

impl CaseStyle {
    pub fn detect(word: &str) -> Self {
        let chars: Vec<char> = word.chars().collect();
        if chars.is_empty() {
            return Self::Lowercase;
        }

        let all_upper = chars.iter().all(|c| !c.is_alphabetic() || c.is_uppercase());
        if all_upper {
            return Self::Uppercase;
        }

        let first_upper = chars[0].is_uppercase();
        let rest_lower = chars
            .iter()
            .skip(1)
            .all(|c| !c.is_alphabetic() || c.is_lowercase());
        if first_upper && rest_lower {
            return Self::Titlecase;
        }

        let all_lower = chars.iter().all(|c| !c.is_alphabetic() || c.is_lowercase());
        if all_lower {
            return Self::Lowercase;
        }

        Self::Mixed
    }

    pub fn apply(&self, word: &str) -> String {
        match self {
            Self::Lowercase => word.to_ascii_lowercase(),
            Self::Uppercase => word.to_ascii_uppercase(),
            Self::Titlecase => {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().collect::<String>()
                            + &chars.as_str().to_ascii_lowercase()
                    }
                }
            }
            Self::Mixed => word.to_string(),
        }
    }
}

/// Statistics on the shielding transformation.
#[derive(Debug, Clone, Default)]
pub struct ShieldStats {
    pub total_words: usize,
    pub content_words: usize,
    pub swapped_words: usize,
    pub stop_words_protected: usize,
    pub date_or_digits_swapped: usize,
}

impl ShieldStats {
    pub fn total_swap_percentage(&self) -> f64 {
        if self.total_words == 0 {
            0.0
        } else {
            (self.swapped_words as f64 / self.total_words as f64) * 100.0
        }
    }

    pub fn content_swap_percentage(&self) -> f64 {
        if self.content_words == 0 {
            0.0
        } else {
            (self.swapped_words as f64 / self.content_words as f64) * 100.0
        }
    }
}

/// A substituted pair to feed into the OpenType GSUB font builder:
/// (decoy_word, original_word)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActiveLigature {
    /// The decoy word that appears in the HTML source code (e.g. "engine")
    pub decoy: String,
    /// The original word that the human reader must see rendered (e.g. "horse")
    pub original: String,
}

/// Result of transforming a text block.
#[derive(Debug, Clone)]
pub struct ShieldResult {
    /// The scrambled decoy text that goes into the HTML DOM for scrapers to see.
    pub decoy_text: String,
    /// The original text as authored.
    pub original_text: String,
    /// The active ligature substitutions needed for the font.
    pub ligatures: Vec<ActiveLigature>,
    /// Metrics on coverage and protection.
    pub stats: ShieldStats,
}

/// Engine that performs text guarding using linguistic rules.
pub struct TextGuardEngine {
    dict: HashMap<String, String>,
}

impl Default for TextGuardEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TextGuardEngine {
    /// Creates a default engine using environment variables if present (`TEXTGUARD_SEED`, `TEXTGUARD_SALT`, `TEXTGUARD_DICTIONARY_PATH`),
    /// falling back to canonical static pairing if no environment variables are set.
    pub fn new() -> Self {
        Self::from_env()
    }

    /// Initializes engine configuration from environment variables:
    /// - `TEXTGUARD_SEED`: Integer seed (e.g. `42` or `12345678`)
    /// - `TEXTGUARD_SALT`: Secret string salt (e.g. `"app-secret-salt"`)
    /// - `TEXTGUARD_DICTIONARY_PATH`: Path to a custom JSON dictionary file
    pub fn from_env() -> Self {
        Self::from_env_with_lookup(|key| std::env::var(key).ok())
    }

    /// Initializes engine configuration with a custom environment lookup function.
    pub fn from_env_with_lookup<F>(lookup: F) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        let dict = if let Some(path) = lookup("TEXTGUARD_DICTIONARY_PATH") {
            Dictionary::from_json_file(std::path::Path::new(&path)).unwrap_or_else(|e| {
                eprintln!(
                    "Warning: failed to load TEXTGUARD_DICTIONARY_PATH ({}): {}",
                    path, e
                );
                Dictionary::default()
            })
        } else {
            Dictionary::default()
        };

        let seed = if let Some(seed_str) = lookup("TEXTGUARD_SEED") {
            seed_str.trim().parse::<u64>().ok()
        } else {
            lookup("TEXTGUARD_SALT").map(|salt_str| hash_str(salt_str.trim()))
        };

        Self::with_dictionary_and_seed(dict, seed)
    }

    /// Creates an engine with an explicit dynamic integer seed for randomized bijective pairing.
    pub fn with_seed(seed: u64) -> Self {
        Self::with_dictionary_and_seed(Dictionary::default(), Some(seed))
    }

    /// Creates an engine with an explicit string salt (e.g. article slug, user ID, tenant ID).
    pub fn with_salt(salt: &str) -> Self {
        Self::with_seed(hash_str(salt))
    }

    /// Creates an engine with a custom dictionary in canonical static mode.
    pub fn with_dictionary(dict: Dictionary) -> Self {
        Self::with_dictionary_and_seed(dict, None)
    }

    /// Creates an engine with a custom dictionary and an optional seed.
    pub fn with_dictionary_and_seed(dict: Dictionary, seed: Option<u64>) -> Self {
        Self {
            dict: dict.build_bijective_map(seed),
        }
    }

    /// Transforms input prose into decoy text while generating ligature rules.
    pub fn transform(&self, input: &str) -> ShieldResult {
        let mut decoy_out = String::with_capacity(input.len());
        let mut ligatures = Vec::new();
        let mut stats = ShieldStats::default();

        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            // 1. Process Alphabetic Word tokens
            if ch.is_alphabetic() {
                let start = i;
                while i < chars.len() && chars[i].is_alphabetic() {
                    i += 1;
                }
                let raw_word: String = chars[start..i].iter().collect();
                stats.total_words += 1;

                let lower = raw_word.to_ascii_lowercase();
                let casing = CaseStyle::detect(&raw_word);

                // Check 1: Is it a protected stop word or glue word?
                if is_forbidden_word(&lower) {
                    stats.stop_words_protected += 1;
                    decoy_out.push_str(&raw_word);
                    continue;
                }

                stats.content_words += 1;

                // Check 2: Is it a month name? (+6 months)
                if let Some(rotated) = rotate_month(&lower) {
                    let decoy = casing.apply(rotated);
                    ligatures.push(ActiveLigature {
                        decoy: decoy.clone(),
                        original: raw_word.clone(),
                    });
                    decoy_out.push_str(&decoy);
                    stats.swapped_words += 1;
                    stats.date_or_digits_swapped += 1;
                    continue;
                }

                // Check 3: Is it a weekday? (+3 days)
                if let Some(rotated) = rotate_weekday(&lower) {
                    let decoy = casing.apply(rotated);
                    ligatures.push(ActiveLigature {
                        decoy: decoy.clone(),
                        original: raw_word.clone(),
                    });
                    decoy_out.push_str(&decoy);
                    stats.swapped_words += 1;
                    stats.date_or_digits_swapped += 1;
                    continue;
                }

                // Check 4: Check bijective cohyponym pool
                if let Some(replacement) = self.dict.get(&lower) {
                    let decoy = casing.apply(replacement);
                    ligatures.push(ActiveLigature {
                        decoy: decoy.clone(),
                        original: raw_word.clone(),
                    });
                    decoy_out.push_str(&decoy);
                    stats.swapped_words += 1;
                } else {
                    // Content word without pair in pool: remains unchanged
                    decoy_out.push_str(&raw_word);
                }
            }
            // 2. Process Digits (Fixed digit couples)
            else if ch.is_ascii_digit() {
                let start = i;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                let raw_digits: String = chars[start..i].iter().collect();
                let swapped = swap_digits_in_str(&raw_digits);
                if swapped != raw_digits {
                    ligatures.push(ActiveLigature {
                        decoy: swapped.clone(),
                        original: raw_digits.clone(),
                    });
                    stats.date_or_digits_swapped += 1;
                }
                decoy_out.push_str(&swapped);
            }
            // 3. Delimiters, Punctuation, Whitespace
            else {
                decoy_out.push(ch);
                i += 1;
            }
        }

        // Deduplicate ligatures while preserving order
        let mut unique_ligs = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for lig in ligatures {
            if seen.insert(lig.decoy.clone()) {
                unique_ligs.push(lig);
            }
        }

        ShieldResult {
            decoy_text: decoy_out,
            original_text: input.to_string(),
            ligatures: unique_ligs,
            stats,
        }
    }
}
