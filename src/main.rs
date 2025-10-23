/// A representation of a phoneme in a constructed language.
pub struct Phoneme {
    /// The textual representation of the sound, using IPA or a custom notation.
    grapheme: String,
    
    /// The type of sound (e.g., Vowel, Consonant, etc.).
    /// We will expand this later.
    sound_type: String, 
}

fn main() {
    // This is the first phoneme of our first language.
    let first_sound = Phoneme {
        grapheme: "a".to_string(),
        sound_type: "Vowel".to_string(),
    };

    println!("The first sound created is: {}, and it's sound type is {}", first_sound.grapheme, first_sound.sound_type);
}