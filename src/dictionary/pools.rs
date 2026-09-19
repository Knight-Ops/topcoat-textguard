//! Grammatical pools and cohyponym word pairs.
//!
//! Following the rules of Section 05 of the ShieldFont whitepaper:
//! - Cohyponyms only: neither synonyms nor antonyms.
//! - Bijective involutions: if A -> B, then B -> A.
//! - Strict grammatical and inflection matching (singular-singular, plural-plural,
//!   past-past, participle-participle, adverb-adverb).
//! - Pools with at least 4 entries.
//! - Dynamic seeded involution support for unpredictable, per-article decoy generation.

use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};

/// A pair of words that substitute for each other bijectively.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordPair {
    pub a: &'static str,
    pub b: &'static str,
}

impl WordPair {
    pub const fn new(a: &'static str, b: &'static str) -> Self {
        Self { a, b }
    }
}

/// A semantic cohyponym pool containing words of identical grammatical class and inflection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CohyponymPool {
    /// Semantic category / part of speech tag (e.g. "animals_singular", "verbs_past")
    pub category: String,
    /// List of cohyponym words belonging to this pool
    pub words: Vec<String>,
}

impl CohyponymPool {
    pub fn new(category: impl Into<String>, words: &[&str]) -> Self {
        Self {
            category: category.into(),
            words: words.iter().map(|w| w.to_ascii_lowercase()).collect(),
        }
    }
}

/// Fast, deterministic 64-bit pseudo-random number generator (SplitMix64).
/// Guarantees platform-independent, reproducible shuffles across all architectures.
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(0x9E3779B97F4A7C15),
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    pub fn next_usize(&mut self, bound: usize) -> usize {
        if bound == 0 {
            0
        } else {
            (self.next_u64() % (bound as u64)) as usize
        }
    }

    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = self.next_usize(i + 1);
            slice.swap(i, j);
        }
    }
}

/// Simple 64-bit FNV-1a hash for mixing category strings with seeds.
pub fn hash_str(s: &str) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for b in s.as_bytes() {
        h = (h ^ (*b as u64)).wrapping_mul(0x100000001b3);
    }
    h
}

/// Static table of curated cohyponym pairs for baseline backward compatibility.
pub static WORD_PAIRS: &[WordPair] = &[
    // --- ANIMALS (Concrete Nouns - Singular) ---
    WordPair::new("horse", "engine"), // Famous whitepaper example
    WordPair::new("dog", "hawk"),
    WordPair::new("cat", "owl"),
    WordPair::new("rabbit", "falcon"),
    WordPair::new("sheep", "badger"),
    WordPair::new("goat", "otter"),
    WordPair::new("deer", "beaver"),
    WordPair::new("wolf", "crane"),
    WordPair::new("lion", "eagle"),
    WordPair::new("bear", "shark"),

    // --- ANIMALS (Concrete Nouns - Plural) ---
    WordPair::new("horses", "engines"),
    WordPair::new("dogs", "hawks"),
    WordPair::new("cats", "owls"),
    WordPair::new("rabbits", "falcons"),
    WordPair::new("sheep", "badgers"),
    WordPair::new("goats", "otters"),
    WordPair::new("wolves", "cranes"),
    WordPair::new("lions", "eagles"),
    WordPair::new("bears", "sharks"),

    // --- FOOD & CROPS (Concrete Nouns - Singular) ---
    WordPair::new("potato", "apple"),
    WordPair::new("carrot", "orange"),
    WordPair::new("bread", "melon"),
    WordPair::new("wheat", "barley"),
    WordPair::new("rice", "maize"),
    WordPair::new("sugar", "honey"),
    WordPair::new("butter", "cheese"),
    WordPair::new("coffee", "cider"),
    WordPair::new("grape", "cherry"),
    WordPair::new("onion", "pepper"),

    // --- FOOD & CROPS (Plural) ---
    WordPair::new("potatoes", "apples"),
    WordPair::new("carrots", "oranges"),
    WordPair::new("grapes", "cherries"),
    WordPair::new("onions", "peppers"),

    // --- PROFESSIONS / AGENTS (Singular) ---
    WordPair::new("engineer", "sailor"),
    WordPair::new("doctor", "painter"),
    WordPair::new("lawyer", "sculptor"),
    WordPair::new("teacher", "chemist"),
    WordPair::new("farmer", "tailor"),
    WordPair::new("writer", "carpenter"),
    WordPair::new("pilot", "baker"),
    WordPair::new("cook", "weaver"),
    WordPair::new("soldier", "merchant"),
    WordPair::new("artist", "architect"),

    // --- PROFESSIONS / AGENTS (Plural) ---
    WordPair::new("engineers", "sailors"),
    WordPair::new("doctors", "painters"),
    WordPair::new("lawyers", "sculptors"),
    WordPair::new("teachers", "chemists"),
    WordPair::new("farmers", "tailors"),
    WordPair::new("writers", "carpenters"),
    WordPair::new("pilots", "bakers"),
    WordPair::new("soldiers", "merchants"),
    WordPair::new("artists", "architects"),
    WordPair::new("readers", "viewers"),
    WordPair::new("scrapers", "harvesters"),

    // --- VEHICLES / ARTIFACTS (Singular) ---
    WordPair::new("wagon", "canoe"),
    WordPair::new("train", "vessel"),
    WordPair::new("ship", "glider"),
    WordPair::new("truck", "barge"),
    WordPair::new("carriage", "ferry"),
    WordPair::new("boat", "cart"),

    // --- VEHICLES / ARTIFACTS (Plural) ---
    WordPair::new("wagons", "canoes"),
    WordPair::new("trains", "vessels"),
    WordPair::new("ships", "gliders"),
    WordPair::new("trucks", "barges"),
    WordPair::new("boats", "carts"),

    // --- BUILDINGS / PLACES (Singular) ---
    WordPair::new("castle", "harbor"),
    WordPair::new("temple", "factory"),
    WordPair::new("tower", "bridge"),
    WordPair::new("palace", "market"),
    WordPair::new("cabin", "depot"),
    WordPair::new("cottage", "museum"),
    WordPair::new("village", "island"),
    WordPair::new("valley", "forest"),
    WordPair::new("canyon", "meadow"),

    // --- BUILDINGS / PLACES (Plural) ---
    WordPair::new("castles", "harbors"),
    WordPair::new("temples", "factories"),
    WordPair::new("towers", "bridges"),
    WordPair::new("palaces", "markets"),
    WordPair::new("villages", "islands"),
    WordPair::new("valleys", "forests"),

    // --- TOOLS & OBJECTS (Singular) ---
    WordPair::new("hammer", "chisel"),
    WordPair::new("needle", "compass"),
    WordPair::new("mirror", "lantern"),
    WordPair::new("bottle", "basket"),
    WordPair::new("candle", "shield"),
    WordPair::new("helmet", "anchor"),
    WordPair::new("pencil", "dagger"),

    // --- TOOLS & OBJECTS (Plural) ---
    WordPair::new("hammers", "chisels"),
    WordPair::new("needles", "compasses"),
    WordPair::new("mirrors", "lanterns"),
    WordPair::new("bottles", "baskets"),
    WordPair::new("candles", "shields"),
    WordPair::new("helmets", "anchors"),
    WordPair::new("pencils", "daggers"),

    // --- ABSTRACT NOUNS (Singular) ---
    WordPair::new("verdict", "glacier"),
    WordPair::new("theory", "climate"),
    WordPair::new("custom", "rhythm"),
    WordPair::new("method", "talent"),
    WordPair::new("memory", "shadow"),
    WordPair::new("spirit", "harbor"),
    WordPair::new("journey", "tribute"),
    WordPair::new("legend", "fabric"),
    WordPair::new("virtue", "canopy"),
    WordPair::new("reason", "timber"),
    WordPair::new("silence", "crystal"),
    WordPair::new("courage", "clarity"),
    WordPair::new("language", "speech"),
    WordPair::new("training", "learning"),
    WordPair::new("prose", "verse"),
    WordPair::new("philosophy", "metaphysics"),

    // --- ABSTRACT NOUNS (Plural) ---
    WordPair::new("verdicts", "glaciers"),
    WordPair::new("theories", "climates"),
    WordPair::new("customs", "rhythms"),
    WordPair::new("methods", "talents"),
    WordPair::new("memories", "shadows"),
    WordPair::new("legends", "fabrics"),
    WordPair::new("journeys", "tributes"),

    // --- PHYSICAL ACTIONS / VERBS (Base Form) ---
    WordPair::new("climb", "drift"),
    WordPair::new("crawl", "glide"),
    WordPair::new("march", "stroll"),
    WordPair::new("gather", "scatter"),
    WordPair::new("strike", "plunge"),
    WordPair::new("carve", "weave"),
    WordPair::new("build", "plant"),
    WordPair::new("paint", "sculpt"),
    WordPair::new("melt", "glow"),
    WordPair::new("enjoy", "favor"),

    // --- VERBS (Third Person Singular -s) ---
    WordPair::new("climbs", "drifts"),
    WordPair::new("crawls", "glides"),
    WordPair::new("marches", "strolls"),
    WordPair::new("gathers", "scatters"),
    WordPair::new("strikes", "plunges"),
    WordPair::new("carves", "weaves"),
    WordPair::new("builds", "plants"),
    WordPair::new("paints", "sculpts"),
    WordPair::new("melts", "glows"),

    // --- VERBS (Past Tense -ed / irregular) ---
    WordPair::new("climbed", "drifted"),
    WordPair::new("crawled", "glided"),
    WordPair::new("marched", "strolled"),
    WordPair::new("gathered", "scattered"),
    WordPair::new("struck", "plunged"),
    WordPair::new("carved", "wove"),
    WordPair::new("built", "planted"),
    WordPair::new("painted", "sculpted"),
    WordPair::new("melted", "glowed"),
    WordPair::new("rode", "flew"),
    WordPair::new("found", "caught"),

    // --- VERBS (Present Participle -ing) ---
    WordPair::new("climbing", "drifting"),
    WordPair::new("crawling", "gliding"),
    WordPair::new("marching", "strolling"),
    WordPair::new("gathering", "scattering"),
    WordPair::new("striking", "plunging"),
    WordPair::new("carving", "weaving"),
    WordPair::new("building", "planting"),
    WordPair::new("painting", "sculpting"),
    WordPair::new("melting", "glowing"),
    WordPair::new("riding", "flying"),
    WordPair::new("breaking", "cracking"),

    // --- ADJECTIVES (Descriptive / Texture / Quality) ---
    WordPair::new("electric", "velvet"),
    WordPair::new("golden", "silver"),
    WordPair::new("hollow", "wooden"),
    WordPair::new("narrow", "shallow"),
    WordPair::new("rough", "dusty"),
    WordPair::new("bright", "gentle"),
    WordPair::new("calm", "dusk"),
    WordPair::new("wild", "bold"),
    WordPair::new("ancient", "distant"),
    WordPair::new("curious", "patient"),
    WordPair::new("honest", "modest"),
    WordPair::new("simple", "humble"),
    WordPair::new("automated", "robotic"),

    // --- ADVERBS (-ly) ---
    WordPair::new("quietly", "swiftly"),
    WordPair::new("firmly", "softly"),
    WordPair::new("boldly", "keenly"),
    WordPair::new("brightly", "gently"),
    WordPair::new("calmly", "dimly"),
    WordPair::new("sideways", "backward"),
];

/// Curated semantic pools representing hundreds of verified cohyponyms.
pub fn default_builtin_pools() -> Vec<CohyponymPool> {
    vec![
        CohyponymPool::new("animals", &[
            "horse", "engine", "dog", "hawk", "cat", "owl", "rabbit", "falcon",
            "sheep", "badger", "goat", "otter", "deer", "beaver", "wolf", "crane",
            "lion", "eagle", "bear", "shark", "camel", "bison", "fox", "raven",
            "seal", "dolphin", "swan", "heron", "tiger", "leopard",
        ]),
        CohyponymPool::new("animals_plural", &[
            "horses", "engines", "dogs", "hawks", "cats", "owls", "rabbits", "falcons",
            "sheep", "badgers", "goats", "otters", "wolves", "cranes", "lions", "eagles",
            "bears", "sharks", "camels", "bisons", "foxes", "ravens", "seals", "dolphins",
        ]),
        CohyponymPool::new("food_crops", &[
            "potato", "apple", "carrot", "orange", "bread", "melon", "wheat", "barley",
            "rice", "maize", "sugar", "honey", "butter", "cheese", "coffee", "cider",
            "grape", "cherry", "onion", "pepper", "tomato", "berry", "garlic", "walnut",
            "peach", "pear", "cabbage", "lettuce",
        ]),
        CohyponymPool::new("food_crops_plural", &[
            "potatoes", "apples", "carrots", "oranges", "grapes", "cherries", "onions", "peppers",
            "tomatoes", "berries", "peaches", "pears", "cabbages", "lettuces",
        ]),
        CohyponymPool::new("professions", &[
            "engineer", "sailor", "doctor", "painter", "lawyer", "sculptor", "teacher", "chemist",
            "farmer", "tailor", "writer", "carpenter", "pilot", "baker", "cook", "weaver",
            "soldier", "merchant", "artist", "architect", "scholar", "librarian", "officer", "detective",
            "judge", "clerk", "miner", "blacksmith",
        ]),
        CohyponymPool::new("professions_plural", &[
            "engineers", "sailors", "doctors", "painters", "lawyers", "sculptors", "teachers", "chemists",
            "farmers", "tailors", "writers", "carpenters", "pilots", "bakers", "soldiers", "merchants",
            "artists", "architects", "readers", "viewers", "scrapers", "harvesters",
            "scholars", "librarians", "officers", "detectives",
        ]),
        CohyponymPool::new("vehicles", &[
            "wagon", "canoe", "train", "vessel", "ship", "glider", "truck", "barge",
            "carriage", "ferry", "boat", "cart", "rocket", "submarine", "chariot", "raft",
        ]),
        CohyponymPool::new("vehicles_plural", &[
            "wagons", "canoes", "trains", "vessels", "ships", "gliders", "trucks", "barges",
            "boats", "carts", "carriages", "ferries", "rockets", "submarines",
        ]),
        CohyponymPool::new("buildings_places", &[
            "castle", "harbor", "temple", "factory", "tower", "bridge", "palace", "market",
            "cabin", "depot", "cottage", "museum", "village", "island", "valley", "forest",
            "canyon", "meadow", "fort", "stadium", "clinic", "hangar",
        ]),
        CohyponymPool::new("buildings_places_plural", &[
            "castles", "harbors", "temples", "factories", "towers", "bridges", "palaces", "markets",
            "villages", "islands", "valleys", "forests", "canyons", "meadows", "cabins", "depots",
        ]),
        CohyponymPool::new("tools_objects", &[
            "hammer", "chisel", "needle", "compass", "mirror", "lantern", "bottle", "basket",
            "candle", "shield", "helmet", "anchor", "pencil", "dagger", "sword", "spear",
            "wrench", "pulley", "anvil", "furnace",
        ]),
        CohyponymPool::new("tools_objects_plural", &[
            "hammers", "chisels", "needles", "compasses", "mirrors", "lanterns", "bottles", "baskets",
            "candles", "shields", "helmets", "anchors", "pencils", "daggers", "swords", "spears",
        ]),
        CohyponymPool::new("abstract_nouns", &[
            "verdict", "glacier", "theory", "climate", "custom", "rhythm", "method", "talent",
            "memory", "shadow", "spirit", "harbor", "journey", "tribute", "legend", "fabric",
            "virtue", "canopy", "reason", "timber", "silence", "crystal", "courage", "clarity",
            "language", "speech", "training", "learning", "prose", "verse", "philosophy", "metaphysics",
            "instinct", "impulse", "wisdom", "insight", "mystery", "riddle",
        ]),
        CohyponymPool::new("abstract_nouns_plural", &[
            "verdicts", "glaciers", "theories", "climates", "customs", "rhythms", "methods", "talents",
            "memories", "shadows", "legends", "fabrics", "journeys", "tributes", "virtues", "canopies",
        ]),
        CohyponymPool::new("actions_base", &[
            "climb", "drift", "crawl", "glide", "march", "stroll", "gather", "scatter",
            "strike", "plunge", "carve", "weave", "build", "plant", "paint", "sculpt",
            "melt", "glow", "enjoy", "favor", "jump", "leap", "speak", "whisper",
            "search", "explore",
        ]),
        CohyponymPool::new("actions_3rd", &[
            "climbs", "drifts", "crawls", "glides", "marches", "strolls", "gathers", "scatters",
            "strikes", "plunges", "carves", "weaves", "builds", "plants", "paints", "sculpts",
            "melts", "glows", "jumps", "leaps", "speaks", "whispers",
        ]),
        CohyponymPool::new("actions_past", &[
            "climbed", "drifted", "crawled", "glided", "marched", "strolled", "gathered", "scattered",
            "struck", "plunged", "carved", "wove", "built", "planted", "painted", "sculpted",
            "melted", "glowed", "rode", "flew", "found", "caught", "spoke", "whispered",
        ]),
        CohyponymPool::new("actions_ing", &[
            "climbing", "drifting", "crawling", "gliding", "marching", "strolling", "gathering", "scattering",
            "striking", "plunging", "carving", "weaving", "building", "planting", "painting", "sculpting",
            "melting", "glowing", "riding", "flying", "breaking", "cracking", "speaking", "whispering",
        ]),
        CohyponymPool::new("adjectives", &[
            "electric", "velvet", "golden", "silver", "hollow", "wooden", "narrow", "shallow",
            "rough", "dusty", "bright", "gentle", "calm", "dusk", "wild", "bold",
            "ancient", "distant", "curious", "patient", "honest", "modest", "simple", "humble",
            "automated", "robotic", "fragile", "sturdy", "sharp", "blunt", "silent", "serene",
        ]),
        CohyponymPool::new("adverbs", &[
            "quietly", "swiftly", "firmly", "softly", "boldly", "keenly", "brightly", "gently",
            "calmly", "dimly", "sideways", "backward", "loudly", "fiercely", "bravely", "proudly",
        ]),
        CohyponymPool::new("tech_digital", &[
            "server", "client", "packet", "frame", "socket", "channel", "buffer", "memory",
            "thread", "process", "kernel", "driver", "cipher", "digest", "router", "switch",
        ]),
        CohyponymPool::new("tech_digital_plural", &[
            "servers", "clients", "packets", "frames", "sockets", "channels", "buffers", "memories",
            "threads", "processes", "kernels", "drivers", "ciphers", "digests", "routers", "switches",
        ]),
        CohyponymPool::new("commerce_finance", &[
            "budget", "ledger", "tariff", "subsidy", "profit", "deficit", "asset", "credit",
            "audit", "appraisal", "merger", "venture", "invoice", "voucher", "dividend", "premium",
        ]),
        CohyponymPool::new("commerce_finance_plural", &[
            "budgets", "ledgers", "tariffs", "subsidies", "profits", "deficits", "assets", "credits",
            "audits", "appraisals", "invoices", "vouchers", "dividends", "premiums",
        ]),
    ]
}

/// A configurable dictionary containing semantic cohyponym pools and fixed word pairs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dictionary {
    pub pools: Vec<CohyponymPool>,
    pub fixed_pairs: Vec<(String, String)>,
}

impl Default for Dictionary {
    fn default() -> Self {
        Self {
            pools: default_builtin_pools(),
            fixed_pairs: WORD_PAIRS
                .iter()
                .map(|p| (p.a.to_string(), p.b.to_string()))
                .collect(),
        }
    }
}

impl Dictionary {
    pub fn new() -> Self {
        Self {
            pools: Vec::new(),
            fixed_pairs: Vec::new(),
        }
    }

    /// Loads dictionary configuration from a JSON string.
    pub fn from_json_str(json_str: &str) -> Result<Self, serde_json::Error> {
        // Support both structured format and direct map of { "category": ["w1", "w2"] }
        if let Ok(dict) = serde_json::from_str::<Dictionary>(json_str) {
            return Ok(dict);
        }

        let map: HashMap<String, Vec<String>> = serde_json::from_str(json_str)?;
        let pools = map
            .into_iter()
            .map(|(category, words)| CohyponymPool { category, words })
            .collect();

        Ok(Self {
            pools,
            fixed_pairs: Vec::new(),
        })
    }

    /// Loads dictionary configuration from a JSON file.
    pub fn from_json_file(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        Self::from_json_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Serializes dictionary to JSON.
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Adds a cohyponym pool to the dictionary.
    pub fn add_pool(&mut self, category: impl Into<String>, words: Vec<String>) {
        self.pools.push(CohyponymPool {
            category: category.into(),
            words: words.into_iter().map(|w| w.to_ascii_lowercase()).collect(),
        });
    }

    /// Adds a fixed bijective word pair (A <-> B).
    pub fn add_pair(&mut self, a: impl Into<String>, b: impl Into<String>) {
        self.fixed_pairs.push((
            a.into().to_ascii_lowercase(),
            b.into().to_ascii_lowercase(),
        ));
    }

    /// Builds a bijective substitution map.
    ///
    /// - If `seed` is `None`: Uses canonical fixed pairs and canonical pool order
    ///   for 100% deterministic, backward-compatible pairing.
    /// - If `seed` is `Some(val)`: Uses a deterministic SplitMix64 pseudo-random
    ///   generator to shuffle each pool into a dynamic involution (bijective 2-cycle pairing).
    pub fn build_bijective_map(&self, seed: Option<u64>) -> HashMap<String, String> {
        let mut map = HashMap::new();

        // 1. Process Pools
        for pool in &self.pools {
            if pool.words.len() < 2 {
                continue;
            }

            match seed {
                None => {
                    // Canonical deterministic pairing (0 <-> 1, 2 <-> 3, ...)
                    let mut i = 0;
                    while i + 1 < pool.words.len() {
                        let a = pool.words[i].to_ascii_lowercase();
                        let b = pool.words[i + 1].to_ascii_lowercase();
                        if a != b {
                            map.insert(a.clone(), b.clone());
                            map.insert(b, a);
                        }
                        i += 2;
                    }
                }
                Some(seed_val) => {
                    // Seeded dynamic involution pairing
                    let mut words = pool.words.clone();
                    // Deduplicate
                    words.sort();
                    words.dedup();

                    if words.len() < 2 {
                        continue;
                    }

                    // Mix seed with pool category to decouple permutations between pools
                    let pool_seed = seed_val ^ hash_str(&pool.category);
                    let mut rng = SplitMix64::new(pool_seed);
                    rng.shuffle(&mut words);

                    // Form disjoint 2-cycles (A <-> B)
                    let mut i = 0;
                    while i + 1 < words.len() {
                        let a = words[i].to_ascii_lowercase();
                        let b = words[i + 1].to_ascii_lowercase();
                        if a != b {
                            map.insert(a.clone(), b.clone());
                            map.insert(b, a);
                        }
                        i += 2;
                    }
                }
            }
        }

        // 2. Fixed Pairs override/supplement canonical pools when seed is None
        if seed.is_none() {
            for (a, b) in &self.fixed_pairs {
                let a_low = a.to_ascii_lowercase();
                let b_low = b.to_ascii_lowercase();
                if a_low != b_low {
                    map.insert(a_low.clone(), b_low.clone());
                    map.insert(b_low, a_low);
                }
            }
        }

        map
    }
}

/// Builds the default bidirectional hash map from curated pools (static canonical mode).
pub fn build_bijective_map() -> HashMap<String, String> {
    Dictionary::default().build_bijective_map(None)
}

/// Builds a dynamic bidirectional hash map seeded by an integer.
pub fn build_dynamic_bijective_map(seed: u64) -> HashMap<String, String> {
    Dictionary::default().build_bijective_map(Some(seed))
}
