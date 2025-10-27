use genesis_engine_lexicon::{initialize_from_config};
use clap::{Parser, Subcommand};


#[derive(Parser, Debug)]
#[command(author, version, about = "Genesis Engine: A Procedural Language Simulator")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate words using a language configuration file
    Generate {
        /// Path to the language JSON file
        #[arg(short, long)]
        lang: String,

        /// Number of words to generate
        #[arg(short, long, default_value_t = 20)]
        count: usize,
    },
    /// Validate the syntax of a language configuration file
    Validate {
        /// Path to the language JSON file to validate
        #[arg(short, long)]
        lang: String,
    },
}


fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate { lang, count } => {
            println!("--- Genesis Engine: Morphological Engine v0.8 ---");
            println!("Loading language from: {}", lang);
            
            // Now we handle the Result from the initialization function.
            match initialize_from_config(lang) {
                Ok((inventory, mut generator)) => {
                    let lexicon = generator.generate_lexicon(*count, &inventory);

                    for i in 0..*count {
                        let word = generator.derive_word(&lexicon[i], &inventory);
                        println!("{}: {} (Meaning:{})", i + 1, word.form, word.derived_meaning);
                    }

                }
                Err(e) => {
                    // If initialization fails, print a user-friendly error message.
                    eprintln!("\nError: Failed to initialize generator.");
                    eprintln!("Reason: {}", e);
                }
            }
        }
        Commands::Validate { lang } => {
            println!("Validating configuration file: {}", lang);
            
            // --- THE NEW VALIDATE LOGIC ---
            match initialize_from_config(lang) {
                Ok(_) => {
                    // If the function returns Ok, it means the file was read and parsed successfully.
                    println!("\n✅ Success: Configuration file is valid and well-formed.");
                }
                Err(e) => {
                    // If it returns an Err, we print a specific, helpful error message.
                    eprintln!("\n❌ Error: Configuration file is invalid.");
                    eprintln!("Reason: {}", e);
                }
            }
        }
    }
}