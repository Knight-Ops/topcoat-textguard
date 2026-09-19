pub mod pools;
pub mod rules;
pub mod tokenizer;

pub use pools::{
    CohyponymPool, Dictionary, WORD_PAIRS, WordPair, build_bijective_map,
    build_dynamic_bijective_map, default_builtin_pools, hash_str,
};
pub use rules::{CLOSED_CLASS_WORDS, STOP_WORDS_113, is_forbidden_word, swap_digits_in_str};
pub use tokenizer::{ActiveLigature, CaseStyle, ShieldResult, ShieldStats, TextGuardEngine};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whitepaper_example() {
        let engine = TextGuardEngine::new();
        let input = "The knight rode his horse into battle.";
        let res = engine.transform(input);

        println!("Original: {}", res.original_text);
        println!("Decoy:    {}", res.decoy_text);
        println!(
            "Swaps:    {}/{} ({:.1}%)",
            res.stats.swapped_words,
            res.stats.total_words,
            res.stats.total_swap_percentage()
        );

        // Stop words "The", "his", "into" must remain untouched
        assert!(res.decoy_text.starts_with("The"));
        assert!(res.decoy_text.contains("his"));
        assert!(res.decoy_text.contains("into"));

        // "horse" must become "engine"
        assert!(res.decoy_text.contains("engine"));

        // Ligature for engine -> horse must exist
        let lig = res
            .ligatures
            .iter()
            .find(|l| l.decoy == "engine")
            .expect("engine ligature");
        assert_eq!(lig.original, "horse");
    }

    #[test]
    fn test_bijective_symmetry_canonical() {
        let engine = TextGuardEngine::new();
        let input = "The potato was eaten by the dog.";
        let res1 = engine.transform(input);
        assert!(res1.decoy_text.contains("apple"));
        assert!(res1.decoy_text.contains("hawk"));

        let res2 = engine.transform(&res1.decoy_text);
        assert_eq!(res2.decoy_text, input);
    }

    #[test]
    fn test_seeded_dynamic_involutions() {
        let engine1 = TextGuardEngine::with_seed(1001);
        let engine2 = TextGuardEngine::with_seed(9999);
        let input = "The doctor joined the brave pilot on the island.";

        let res1 = engine1.transform(input);
        let res2 = engine2.transform(input);

        println!("Seed 1001 decoy: {}", res1.decoy_text);
        println!("Seed 9999 decoy: {}", res2.decoy_text);

        // Both engines produce valid text with substitutions
        assert_ne!(res1.decoy_text, input);
        assert_ne!(res2.decoy_text, input);

        // Different seeds produce different decoys
        assert_ne!(res1.decoy_text, res2.decoy_text);

        // Crucial test: Bijective involution holds for ANY seed!
        let roundtrip1 = engine1.transform(&res1.decoy_text);
        assert_eq!(
            roundtrip1.decoy_text, input,
            "Involution failed for seed 1001"
        );

        let roundtrip2 = engine2.transform(&res2.decoy_text);
        assert_eq!(
            roundtrip2.decoy_text, input,
            "Involution failed for seed 9999"
        );
    }

    #[test]
    fn test_custom_json_dictionary() {
        let json_data = r#"{
            "pools": [
                {
                    "category": "space_objects",
                    "words": ["pulsar", "quasar", "nebula", "asteroid", "comet", "galaxy"]
                }
            ],
            "fixed_pairs": [
                ["spacex", "blueorigin"]
            ]
        }"#;

        let dict = Dictionary::from_json_str(json_data).expect("valid JSON");
        let engine = TextGuardEngine::with_dictionary(dict);

        let input = "The spacex rocket passed the pulsar near the asteroid.";
        let res = engine.transform(input);

        assert!(res.decoy_text.contains("blueorigin"));
        assert!(res.decoy_text.contains("quasar")); // pulsar paired with quasar (0 <-> 1)
        assert!(res.decoy_text.contains("nebula")); // asteroid paired with nebula (2 <-> 3)

        let roundtrip = engine.transform(&res.decoy_text);
        assert_eq!(roundtrip.decoy_text, input);
    }

    #[test]
    fn test_env_var_configuration() {
        let input = "The doctor joined the brave pilot on the island.";

        // Test TEXTGUARD_SEED lookup
        let engine_seed = TextGuardEngine::from_env_with_lookup(|key| match key {
            "TEXTGUARD_SEED" => Some("8888".to_string()),
            _ => None,
        });
        let res_seed = engine_seed.transform(input);
        let res_explicit = TextGuardEngine::with_seed(8888).transform(input);
        assert_eq!(res_seed.decoy_text, res_explicit.decoy_text);

        // Test TEXTGUARD_SALT lookup
        let engine_salt = TextGuardEngine::from_env_with_lookup(|key| match key {
            "TEXTGUARD_SALT" => Some("my-production-salt".to_string()),
            _ => None,
        });
        let res_salt = engine_salt.transform(input);
        let res_salt_explicit = TextGuardEngine::with_salt("my-production-salt").transform(input);
        assert_eq!(res_salt.decoy_text, res_salt_explicit.decoy_text);
    }
}
