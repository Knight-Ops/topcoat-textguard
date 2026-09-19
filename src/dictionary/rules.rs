//! The linguistic rules defined in Section 05 of the ShieldFont whitepaper.
//!
//! Includes:
//! - 113-word do-not-swap list (stop-list)
//! - Closed-class lockdown words (glue words)
//! - Bijective digit couples (0<->5, 3<->8, 4<->9, 6<->7, leaving 1 and 2 alone)
//! - Date rotation rules (+6 months, +3 weekdays)

/// The 113-word do-not-swap list from Section 05 of the whitepaper.
/// Touching any of these causes perplexity to jump +1,076% and quality filters to discard the page.
pub const STOP_WORDS_113: &[&str] = &[
    // Articles
    "a", "an", "the",
    // Conjunctions
    "and", "but", "or", "nor", "for", "so", "yet", "because", "although", "while", "if", "unless",
    "since", "as", "though",
    // Prepositions
    "of", "in", "to", "for", "with", "on", "at", "by", "from", "up", "about", "into", "over",
    "after", "beneath", "under", "above", "across", "through", "between", "against", "during",
    "without", "before", "toward", "towards", "upon", "within", "along",
    // Pronouns
    "i", "you", "he", "she", "it", "we", "they", "me", "him", "her", "us", "them",
    "my", "your", "his", "its", "our", "their", "mine", "yours", "hers", "ours", "theirs",
    "this", "that", "these", "those", "who", "whom", "whose", "which", "what", "whatever",
    // Auxiliary & copula verbs (every form of be, have, do)
    "is", "am", "are", "was", "were", "be", "been", "being",
    "have", "has", "had", "having",
    "do", "does", "did", "doing",
    // Modals
    "can", "could", "shall", "should", "will", "would", "may", "might", "must",
    // Negations
    "not", "no", "never", "none",
    // Quantifiers
    "all", "any", "some", "few", "many", "much", "more", "most", "every", "each", "both", "either", "neither",
];

/// Closed-class lockdown list: frequent words that look like content but behave like glue.
pub const CLOSED_CLASS_WORDS: &[&str] = &[
    "get", "gets", "got", "gotten", "getting",
    "make", "makes", "made", "making",
    "said", "says", "say", "saying",
    "day", "days", "now", "then", "here", "there",
    "just", "also", "very", "too", "well", "even", "only", "such",
];

/// Check if a lowercase word is on the prohibited list (stop words or closed-class glue).
pub fn is_forbidden_word(word: &str) -> bool {
    let w = word.to_ascii_lowercase();
    STOP_WORDS_113.iter().any(|&s| s == w) || CLOSED_CLASS_WORDS.iter().any(|&s| s == w)
}

/// Digits swap in fixed couples: (0 <-> 5, 3 <-> 8, 4 <-> 9, 6 <-> 7).
/// 1 and 2 are deliberately left alone so years like 1990, 2024 still read plausibly.
pub fn swap_digit(ch: char) -> char {
    match ch {
        '0' => '5',
        '5' => '0',
        '3' => '8',
        '8' => '3',
        '4' => '9',
        '9' => '4',
        '6' => '7',
        '7' => '6',
        other => other,
    }
}

/// Swaps digits in a string according to the whitepaper rules.
pub fn swap_digits_in_str(s: &str) -> String {
    s.chars().map(swap_digit).collect()
}

/// Month rotation by 6 months (+6).
const MONTHS: [&str; 12] = [
    "january", "february", "march", "april", "may", "june",
    "july", "august", "september", "october", "november", "december"
];

/// Weekday rotation by 3 days (+3).
const WEEKDAYS: [&str; 7] = [
    "monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"
];

/// Rotates months by 6 months.
pub fn rotate_month(word: &str) -> Option<&'static str> {
    let lower = word.to_ascii_lowercase();
    let idx = MONTHS.iter().position(|&m| m == lower)?;
    Some(MONTHS[(idx + 6) % 12])
}

/// Rotates weekdays by 3 days.
pub fn rotate_weekday(word: &str) -> Option<&'static str> {
    let lower = word.to_ascii_lowercase();
    let idx = WEEKDAYS.iter().position(|&d| d == lower)?;
    Some(WEEKDAYS[(idx + 3) % 7])
}
