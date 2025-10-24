use rand::prelude::*;
use serde::Deserialize;
use std::{fs::File, io::Read};

#[derive(Debug, Clone, Deserialize)]
pub struct Phoneme {
    grapheme: String,
    sound_type: String,
}

pub struct PhoneticInventory {
    phonemes: Vec<Phoneme>,
}

impl PhoneticInventory {
    fn get_random_consonant(&self) -> Option<&Phoneme> {
        self.phonemes.iter().filter(|p| p.sound_type == "Consonant").collect::<Vec<_>>().choose(&mut rand::rng()).copied()
    }

    fn get_random_vowel(&self) -> Option<&Phoneme> {
        self.phonemes.iter().filter(|p| p.sound_type == "Vowel").collect::<Vec<_>>().choose(&mut rand::rng()).copied()
    }
}

#[derive(Debug, Clone)]
pub struct SyllableRule {
    pattern: String,
}

impl SyllableRule {
    pub fn new(pattern: &str) -> Self {
        Self { pattern: pattern.to_string() }
    }
}

pub struct SyllableGenerator {
    pub rules: Vec<SyllableRule>,
    pub min_syllables: usize,
    pub max_syllables: usize,
}

impl SyllableGenerator {
    pub fn new(rules: Vec<SyllableRule>, min_syllables: usize, max_syllables: usize) -> Self {
        Self { rules, min_syllables, max_syllables }
    }

    pub fn generate_syllable(&self, inventory: &PhoneticInventory) -> String {
        let rule = self.rules.choose(&mut rand::rng()).unwrap();
        let mut syllable = String::new();
        for c in rule.pattern.chars() {
            match c {
                'C' => {
                    if let Some(consonant) = inventory.get_random_consonant() { syllable.push_str(&consonant.grapheme); }
                }
                'V' => {
                    if let Some(vowel) = inventory.get_random_vowel() { syllable.push_str(&vowel.grapheme); }
                }
                _ => {}
            }
        }
        syllable
    }

    pub fn generate_word(&self, inventory: &PhoneticInventory) -> String {
        let num_syllables = rand::rng().random_range(self.min_syllables..=self.max_syllables);
        let mut word = String::new();
        for _ in 0..num_syllables {
            word.push_str(&self.generate_syllable(inventory));
        }
        word
    }
}

#[derive(Deserialize)]
pub struct LanguageConfig {
    pub phonemes: Vec<Phoneme>,
    pub syllable_rules: Vec<String>,
    pub min_syllables: usize,
    pub max_syllables: usize,
}

/// Reads a config file and builds the core engine components.
fn initialize_from_config(config_path: &str) -> (PhoneticInventory, SyllableGenerator) {
    // 1. Read and parse the configuration file
    let mut config_file = File::open(config_path).expect("Failed to open config file");
    let mut contents = String::new();
    config_file.read_to_string(&mut contents).expect("Failed to read config file");
    let config: LanguageConfig = serde_json::from_str(&contents).expect("Failed to parse JSON");

    // 2. Build the engine components from the configuration
    let inventory = PhoneticInventory { phonemes: config.phonemes };
    let syllable_rules = config.syllable_rules.iter().map(|r| SyllableRule::new(r)).collect();

    let syllable_generator = SyllableGenerator::new(
        syllable_rules,
        config.min_syllables,
        config.max_syllables,
    );
    
    // 3. Return the constructed components as a tuple
    (inventory, syllable_generator)
}

fn main() {
    // 1. Initialize the engine from our configuration file.
    let (inventory, generator) = initialize_from_config("language.json");

    // 2. Run the generation process.
    println!("--- Genesis Engine: Configurable Lexicon ---");
    println!("Generating 20 random words from {}...\n", "language.json");
    for i in 0..20 {
        let word = generator.generate_word(&inventory);
        println!("Word {}: {}", i + 1, word);
    }
}