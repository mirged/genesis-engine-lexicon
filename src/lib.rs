pub mod error;
pub use error::ConfigError;
use rand::prelude::*;
use serde::Deserialize;
use std::{fs::File, io::Read};

#[derive(Debug, Clone, Deserialize)]
pub struct Phoneme {
    grapheme: String,
    sound_type: String,
}

pub struct PhoneticInventory {
    vowels: Vec<Phoneme>,
    consonants: Vec<Phoneme>,
}

impl PhoneticInventory {
    pub fn new(all_phonemes: Vec<Phoneme>) -> Self {
        let vowels = all_phonemes.iter().filter(|p| p.sound_type == "Vowel").cloned().collect();
        let consonants = all_phonemes.iter().filter(|p| p.sound_type == "Consonant").cloned().collect();
        Self { vowels, consonants }
    }

    fn get_random_consonant(&self) -> Option<&Phoneme> {
        let mut rng = rand::rng();
        self.consonants.choose(&mut rng)
    }

    fn get_random_vowel(&self) -> Option<&Phoneme> {
        let mut rng = rand::rng();
        self.vowels.choose(&mut rng)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SyllablePattern {
    pattern: String,
}

impl SyllablePattern {
    pub fn new(pattern: &str) -> Self {
        Self { pattern: pattern.to_string() }
    }

    pub fn is_vowel_only(&self) -> bool {
        self.pattern.chars().all(|c| c == 'V')
    }
    
    pub fn starts_with(&self, pattern_type: &str) -> bool {
        self.pattern.starts_with(pattern_type)
    }
}


#[derive(Debug, Clone, Deserialize, Default)]
pub struct Morphology {
    // The source JSON contains a flat list `affixes`; we'll store that and
    // filter when we need prefixes/suffixes.
    #[serde(default)]
    pub affixes: Vec<Affix>,
}
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SequenceRules {
    pub max_vowel_syllables_in_a_row: usize,
    // (We'll focus on implementing the vowel rule first, as it's the most pressing)
    // pub max_consonant_syllables_in_a_row: usize,
    // pub allow_word_start_with: Vec<String>,
    // pub allow_word_end_with: Vec<String>,
}

use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LexiconGeneration {
    #[serde(default)]
    pub parts_of_speech: Vec<String>,
    #[serde(default)]
    pub meanings: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Root {
    pub form: String,
    pub part_of_speech: String, // e.g., "noun", "verb", "adj"
    pub meaning: String,          // e.g., "water", "to see", "big"
}

// An Affix now also has a meaning/function.
#[derive(Debug, Clone, Deserialize)]
pub struct Affix {
    pub form: String,
    pub affix_type: String, // "prefix" or "suffix"
    pub function: String,   // e.g., "negation", "agent_noun", "causative"
    pub meaning: String,    // e.g., "not", "one who does", "to make"
}

// A complete, generated word with its full history.
#[derive(Debug, Clone)]
pub struct Word {
    pub form: String,             // The final word, e.g., "un-kal-os"
    pub root: Root,               // The core root, e.g., {form: "kal", pos: "verb", meaning: "to see"}
    pub affixes: Vec<Affix>,      // The list of affixes applied
    pub derived_meaning: String,  // A constructed meaning, e.g., "one who does not see"
}

#[allow(dead_code)]
pub struct Lexicon {
    roots: Vec<Root>,
    affixes: Vec<Affix>,
    pub words: Vec<Word>,
}

pub struct WordGenerator {
    pub rules: Vec<SyllablePattern>,
    pub min_syllables: usize,
    pub max_syllables: usize,
    pub illegal_patterns: Vec<String>,
    pub morphology: Morphology,
    pub lexicon_generation: LexiconGeneration,
    pub sequence_rules: SequenceRules,
}

impl WordGenerator {
    pub fn new(
        rules: Vec<SyllablePattern>,
        min: usize,
        max: usize,
        illegal: Vec<String>,
        morphology: Morphology,
        lexicon_generation: LexiconGeneration,
        sequence_rules: SequenceRules,
    ) -> Self {
        Self {
            rules,
            min_syllables: min,
            max_syllables: max,
            illegal_patterns: illegal,
            morphology,
            lexicon_generation,
            sequence_rules,
        }
    }

    fn generate_syllable_from_pattern(&self, inventory: &PhoneticInventory, pattern: &SyllablePattern) -> String {
        let mut syllable = String::new();
        for c in pattern.pattern.chars() {
            match c {
                'C' => {
                    if let Some(consonant) = inventory.get_random_consonant() { 
                        syllable.push_str(&consonant.grapheme); 
                    }
                }
                'V' => {
                    if let Some(vowel) = inventory.get_random_vowel() { 
                        syllable.push_str(&vowel.grapheme); 
                    }
                }
                _ => {} // Ignore other characters
            }
        }
        syllable
    }

    fn contains_illegal_patterns(&self, word: &str) -> bool {
        self.illegal_patterns.iter().any(|pattern| word.contains(pattern))
    }

    pub fn generate_root(&self, inventory: &PhoneticInventory) -> String {
        let max_attempts = 100;
        let mut rng = rand::rng();

        for _ in 0..max_attempts {
            let num_syllables = rng.random_range(self.min_syllables..=self.max_syllables);
            let mut root_word = String::new();
            
            let mut consecutive_vowels = 0;

            for _ in 0..num_syllables {
                let mut possible_rules = self.rules.clone();

                if consecutive_vowels >= self.sequence_rules.max_vowel_syllables_in_a_row {
                    possible_rules.retain(|rule| !rule.is_vowel_only());
                }

                let chosen_rule = if possible_rules.is_empty() {
                    self.rules.choose(&mut rng).unwrap()
                } else {
                    possible_rules.choose(&mut rng).unwrap()
                };

                root_word.push_str(&self.generate_syllable_from_pattern(inventory, chosen_rule));

                if chosen_rule.is_vowel_only() {
                    consecutive_vowels += 1;
                } else {
                    consecutive_vowels = 0;
                }
            }

            if !self.contains_illegal_patterns(&root_word) {
                return root_word;
            }
        }

        panic!("Failed to generate a valid root after {} attempts. Check your language configuration for overly restrictive rules.", max_attempts);
    }

    pub fn generate_lexicon(&mut self, count: usize, phonetic_inventory: &PhoneticInventory) -> Vec<Root> {
        let mut roots = Vec::new();
        let mut used_forms = std::collections::HashSet::new();
        let mut rng = rand::rng();

        while roots.len() < count {
            
            let form = self.generate_root(phonetic_inventory);
            
            if !used_forms.contains(&form) {
                used_forms.insert(form.clone());
                
                // Pick a random part of speech and a meaning from `lexicon_generation` if available.
                let part_of_speech = if !self.lexicon_generation.parts_of_speech.is_empty() {
                    self.lexicon_generation
                        .parts_of_speech
                        .choose(&mut rng)
                        .unwrap()
                        .clone()
                } else {
                    String::from("noun")
                };

                let meaning = self
                    .lexicon_generation
                    .meanings
                    .get(&part_of_speech)
                    .and_then(|vec| vec.choose(&mut rng).cloned())
                    .unwrap_or_else(|| String::from("placeholder"));

                let root = Root { form, part_of_speech, meaning };
                
                roots.push(root);
            }
        }

        roots
    }

    pub fn derive_word(&self, root: &Root, _inventory: &PhoneticInventory) -> Word {
        let mut rng = rand::rng();

        // Start from the root form and randomly add affixes from the generator's morphology
        let mut form = root.form.clone();
        let mut derived_affixes: Vec<Affix> = Vec::new();

        // Randomly decide to add a prefix (morphology.affixes may contain both prefixes and suffixes)
        if rng.random_bool(0.5) {
            if let Some(prefix) = self.morphology.affixes.iter().filter(|a| a.affix_type == "prefix").choose(&mut rng) {
                // Affix forms in config include their hyphens (e.g. "re-" or "-os").
                form = format!("{}{}", prefix.form, form);
                derived_affixes.push(prefix.clone());
            }
        }

        // Randomly decide to add a suffix
        if rng.random_bool(0.5) {
            if let Some(suffix) = self.morphology.affixes.iter().filter(|a| a.affix_type == "suffix").choose(&mut rng) {
                form = format!("{}{}", form, suffix.form);
                derived_affixes.push(suffix.clone());
            }
        }

        // Construct the derived meaning
        let derived_meaning = derived_affixes.iter()
            .map(|a| a.meaning.clone())
            .chain(std::iter::once(root.meaning.clone()))
            .collect::<Vec<String>>()
            .join(" ");

        Word {
            form,
            root: root.clone(),
            affixes: derived_affixes,
            derived_meaning,
        }
    }


}

#[derive(Deserialize)]
pub struct LanguageConfig {
    pub phonemes: Vec<Phoneme>,
    pub syllable_rules: Vec<String>,
    #[serde(alias = "min_syllables_for_root")]
    pub min_syllables: usize,
    #[serde(alias = "max_syllables_for_root")]
    pub max_syllables: usize,
    #[serde(default)]
    pub illegal_patterns: Vec<String>,
    #[serde(default)]
    pub morphology: Morphology,
    #[serde(default)]
    pub sequence_rules: SequenceRules,
    #[serde(default)]
    pub lexicon_generation: LexiconGeneration,
}

pub fn initialize_from_config(config_path: &str) -> Result<(PhoneticInventory, WordGenerator), ConfigError> {
    let mut file = File::open(config_path).map_err(ConfigError::FileRead)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).map_err(ConfigError::FileRead)?;

    let config: LanguageConfig = serde_json::from_str(&contents).map_err(ConfigError::JsonParse)?;

    let inventory = PhoneticInventory::new(config.phonemes);
    let rules = config.syllable_rules.iter().map(|r| SyllablePattern::new(r)).collect::<Vec<SyllablePattern>>();
    let generator = WordGenerator::new(
        rules.clone(),
        config.min_syllables,
        config.max_syllables,
        config.illegal_patterns,
        config.morphology,
        config.lexicon_generation,
        config.sequence_rules,
    );

    Ok((inventory, generator))
}