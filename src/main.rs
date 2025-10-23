// The modern, idiomatic way to import the necessary random-generation tools.
use rand::prelude::*;

#[derive(Debug, Clone)]
pub struct Phoneme {
    grapheme: String,
    sound_type: String,
}

pub struct PhoneticInventory {
    phonemes: Vec<Phoneme>,
}

impl PhoneticInventory {
    // This function returns a random consonant from the inventory.
    fn get_random_consonant(&self) -> Option<&Phoneme> {
        self.phonemes
            .iter()
            .filter(|p| p.sound_type == "Consonant")
            .collect::<Vec<_>>()
            // .choose() is now in scope because we imported the prelude.
            .choose(&mut rand::rng())
            .copied()
    }

    // This function returns a random vowel from the inventory.
    fn get_random_vowel(&self) -> Option<&Phoneme> {
        self.phonemes
            .iter()
            .filter(|p| p.sound_type == "Vowel")
            .collect::<Vec<_>>()
            .choose(&mut rand::rng())
            .copied()
    }
}

// This function is our generation engine.
fn generate_word(inventory: &PhoneticInventory) -> String {
    // Our first, simple rule: A word is one Consonant-Vowel-Consonant (CVC) syllable.
    let consonant1 = inventory.get_random_consonant().unwrap();
    let vowel = inventory.get_random_vowel().unwrap();
    let consonant2 = inventory.get_random_consonant().unwrap();

    // Combine the graphemes of the chosen phonemes into a single word.
    format!("{}{}{}", consonant1.grapheme, vowel.grapheme, consonant2.grapheme)
}

fn main() {
    let inventory = PhoneticInventory {
        phonemes: vec![
            Phoneme { grapheme: "p".to_string(), sound_type: "Consonant".to_string() },
            Phoneme { grapheme: "t".to_string(), sound_type: "Consonant".to_string() },
            Phoneme { grapheme: "k".to_string(), sound_type: "Consonant".to_string() },
            Phoneme { grapheme: "m".to_string(), sound_type: "Consonant".to_string() },
            Phoneme { grapheme: "s".to_string(), sound_type: "Consonant".to_string() },
            Phoneme { grapheme: "a".to_string(), sound_type: "Vowel".to_string() },
            Phoneme { grapheme: "i".to_string(), sound_type: "Vowel".to_string() },
            Phoneme { grapheme: "u".to_string(), sound_type: "Vowel".to_string() },
            Phoneme { grapheme: "o".to_string(), sound_type: "Vowel".to_string() },
        ],
    };

    println!("--- Genesis Engine: Lexicon ---");
    println!("Generating 10 random words with a CVC structure...");

    for _ in 0..10 {
        let word = generate_word(&inventory);
        println!("{}", word);
    }
}