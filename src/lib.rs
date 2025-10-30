pub mod error;
pub use error::ConfigError;
use rand::prelude::*;
use serde::Deserialize;
use std::{fs::File, io::Read};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct Phoneme {
    grapheme: String,
    sound_type: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Grammar {
    #[serde(default = "default_word_order")]
    pub word_order: String, 
}

fn default_word_order() -> String { "SVO".to_string() }

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
    pub derivational_rules: Vec<DerivationalRule>, // This replaces the old `affixes` field.
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
pub struct DerivationalRule {
    pub name: String,                   // e.g., "Agent Noun", "Augmentative"
    pub applies_to_pos: Vec<String>,    // e.g., ["verb"]
    pub output_pos: String,             // e.g., "noun"
    
    #[serde(flatten)] // This allows "type": "Prefix" or "type": "Suffix" in the JSON
    pub process: DerivationProcess,
    
    pub meaning_template: String,       // e.g., "great-{parent_meaning}"
    #[serde(default)] // This makes the field optional in the JSON
    pub constraints: RuleConstraints,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RuleConstraints {
    #[serde(default)]
    pub cannot_follow_rules: Vec<String>, // e.g., "LocationOf" can't follow "LocationOf"
    // We could later add things like:
    // pub cannot_be_followed_by: Vec<String>,
    // pub required_parent_tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum DerivationProcess {
    Prefix { form: String },
    Suffix { form: String },
    // We can add more later, like Infix or Compounding
}

// A complete, generated word with its full history.
#[derive(Debug, Clone)]
pub struct Lexeme {
    pub id: Uuid,
    pub form: String,
    pub part_of_speech: String,
    pub meaning: String,
    
    // Graph-related fields
    pub parent_id: Option<Uuid>,      // Which lexeme did this derive from?
    pub rule_applied: Option<String>, // The name of the rule that created it.
}



#[allow(dead_code)]
pub struct Lexicon {
    // We use a HashMap to easily look up any lexeme by its ID.
    pub graph: HashMap<Uuid, Lexeme>,
    pub roots: Vec<Uuid>, // A list of IDs for the "generation 0" root words.
}

impl Lexicon {
    pub fn new() -> Self {
        Self {
            graph: HashMap::new(),
            roots: Vec::new(),
        }
    }

    pub fn add_lexeme(&mut self, lexeme: Lexeme) {
        if lexeme.parent_id.is_none() {
            self.roots.push(lexeme.id);
        }
        self.graph.insert(lexeme.id, lexeme);
    }
}

pub struct WordGenerator {
    pub rules: Vec<SyllablePattern>,
    pub min_syllables: usize,
    pub max_syllables: usize,
    pub illegal_patterns: Vec<String>,
    pub morphology: Morphology,
    pub lexicon_generation: LexiconGeneration,
    pub sequence_rules: SequenceRules,
    pub grammar: Grammar,
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
        grammar: Grammar,
    ) -> Self {
        Self {
            rules,
            min_syllables: min,
            max_syllables: max,
            illegal_patterns: illegal,
            morphology,
            lexicon_generation,
            sequence_rules,
            grammar,
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

    pub fn build_etymological_graph(&self, root_count: usize, inventory: &PhoneticInventory, derivation_passes: usize) -> Lexicon {
        let mut lexicon = Lexicon::new();
        let mut rng = rand::rng();
        let mut form_to_id_map: HashMap<String, Uuid> = HashMap::new();

        let mut used_forms = std::collections::HashSet::new();
        while lexicon.roots.len() < root_count {
            let form = self.generate_root(inventory);
            if !used_forms.contains(&form) {
                used_forms.insert(form.clone());
                let part_of_speech = self.lexicon_generation.parts_of_speech.choose(&mut rng).unwrap().clone();
                let meaning = self.lexicon_generation.meanings.get(&part_of_speech).and_then(|v| v.choose(&mut rng).cloned()).unwrap_or_default();
                if !form_to_id_map.contains_key(&form){
                    let root_lexeme = Lexeme {
                    id: Uuid::new_v4(),
                    form,
                    part_of_speech,
                    meaning,
                    parent_id: None,
                    rule_applied: None,
                    };

                    form_to_id_map.insert(root_lexeme.form.clone(), root_lexeme.id);
                    lexicon.add_lexeme(root_lexeme);
                }
            }
        }
    

        let mut current_generation_ids: Vec<Uuid> = lexicon.roots.clone();

        for i in 0..derivation_passes {
            println!("\n--- Derivation Pass {} ---", i + 1);
            let mut next_generation_ids = Vec::new();
            let mut newly_derived_lexemes: Vec<Lexeme> = Vec::new();
            
            for parent_id in &current_generation_ids {
                let parent_lexeme = lexicon.graph.get(parent_id).unwrap();

                for rule in &self.morphology.derivational_rules {
                    if rule.applies_to_pos.contains(&parent_lexeme.part_of_speech) {
                        
                        let mut is_constrained = false;
                        if let Some(parent_rule_name) = &parent_lexeme.rule_applied {
                            if rule.constraints.cannot_follow_rules.contains(parent_rule_name) {
                                is_constrained = true;
                            }
                        }
                        if !is_constrained {
                            let (new_form, new_pos, new_meaning) = Self::apply_rule(&parent_lexeme, rule);
                            if !form_to_id_map.contains_key(&new_form) {

                                let child_lexeme = Lexeme {
                                    id: Uuid::new_v4(),
                                    form: new_form,
                                    part_of_speech: new_pos,
                                    meaning: new_meaning,
                                    parent_id: Some(parent_lexeme.id),
                                    rule_applied: Some(rule.name.clone()),
                                };
                                form_to_id_map.insert(child_lexeme.form.clone(), child_lexeme.id);
                                
                                println!("  Derived '{}' ({}) from '{}' using rule '{}'", child_lexeme.form, child_lexeme.meaning, parent_lexeme.form, rule.name);
                                next_generation_ids.push(child_lexeme.id);
                                newly_derived_lexemes.push(child_lexeme);
                            }

                        };

                    }
                }
            }
            
            if next_generation_ids.is_empty() {
                println!("No new words derived. Halting derivation.");
                break; // Stop if a pass yields no new words
            }

            for lexeme in newly_derived_lexemes {
                lexicon.graph.insert(lexeme.id, lexeme);
            }
            
            current_generation_ids = next_generation_ids;
        }

        lexicon
    }

    fn apply_rule(parent: &Lexeme, rule: &DerivationalRule) -> (String, String, String) {
    let new_form = match &rule.process {
        DerivationProcess::Prefix { form } => format!("{}{}", form, parent.form),
        DerivationProcess::Suffix { form } => format!("{}{}", parent.form, form),
    };

    let new_pos = if rule.output_pos == "SameAsInput" {
        parent.part_of_speech.clone()
    } else {
        rule.output_pos.clone()
    };
    
    let new_meaning = rule.meaning_template.replace("{parent_meaning}", &parent.meaning);

    (new_form, new_pos, new_meaning)
    }

    pub fn generate_sentence(&self, lexicon: &Lexicon) -> String {
        let mut rng = rand::rng();

        // Find a random noun for the subject
        let subject = lexicon.graph.values()
            .filter(|l| l.part_of_speech == "noun")
            .choose(&mut rng)
            .map_or("<noun>", |l| &l.form);

        // Find a random verb
        let verb = lexicon.graph.values()
            .filter(|l| l.part_of_speech == "verb")
            .choose(&mut rng)
            .map_or("<verb>", |l| &l.form);

        // Find a random noun for the object
        let object = lexicon.graph.values()
            .filter(|l| l.part_of_speech == "noun")
            .choose(&mut rng)
            .map_or("<noun>", |l| &l.form);

        // Arrange them based on the grammar rule
        let sentence = match self.grammar.word_order.as_str() {
            "SVO" => format!("{} {} {}", subject, verb, object),
            "SOV" => format!("{} {} {}", subject, object, verb),
            "VSO" => format!("{} {} {}", verb, subject, object),
            "VOS" => format!("{} {} {}", verb, object, subject),
            "OSV" => format!("{} {} {}", object, subject, verb),
            "OVS" => format!("{} {} {}", object, verb, subject),

            _ => format!("{} {} {}", subject, verb, object), // Default to SVO
        };
        
        // Capitalize first letter and add a period.
        let mut c = sentence.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str() + ".",
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
    #[serde(default)]
    pub grammar: Grammar,
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
        config.grammar,
    );

    Ok((inventory, generator))
}

pub fn export_to_dot(lexicon: &Lexicon) -> String {
    let mut dot_string = String::from("digraph GenesisLexicon {\n");
    dot_string.push_str("  rankdir=LR;\n"); // Layout left-to-right
    dot_string.push_str("  node [shape=box, style=rounded];\n\n");

    // First, define all the nodes
    for (id, lexeme) in &lexicon.graph {
        let label = format!(
            "\"{} [{}]\\n'{}'\"", // Format: "form [pos]\n'meaning'"
            lexeme.form.replace('"', "\\\""), // Escape quotes
            lexeme.part_of_speech,
            lexeme.meaning.replace('"', "\\\"")
        );
        
        let color = if lexeme.parent_id.is_none() { "lightblue" } else { "lightgray" };

        dot_string.push_str(&format!(
            "  \"{}\" [label={}, style=filled, fillcolor={}];\n",
            id, label, color
        ));
    }

    dot_string.push_str("\n");

    // Second, define all the edges (relationships)
    for (id, lexeme) in &lexicon.graph {
        if let Some(parent_id) = lexeme.parent_id {
            let rule_label = lexeme.rule_applied.as_deref().unwrap_or("");
            dot_string.push_str(&format!(
                "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
                parent_id, id, rule_label
            ));
        }
    }

    dot_string.push_str("}\n");
    dot_string
}