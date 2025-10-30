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

    /// Generate a lexicon and visualize it as a graph
    Visualize {
        /// Path to the language JSON file
        #[arg(short, long)]
        lang: String,

        /// Number of root words to generate
        #[arg(short, long, default_value_t = 10)]
        count: usize,

        /// Number of derivation passes to run
        #[arg(short, long, default_value_t = 2)]
        passes: usize,

        /// Output file path for the .dot file
        #[arg(short, long, default_value = "lexicon.dot")]
        output: String,
    },

    /// Generate sample sentences from a language
    Narrate {
        /// Path to the language JSON file
        #[arg(short, long)]
        lang: String,

        /// Number of root words to build the lexicon with
        #[arg(long, default_value_t = 50)]
        roots: usize,

        /// Number of derivation passes to run
        #[arg(long, default_value_t = 2)]
        passes: usize,

        /// Number of sentences to generate
        #[arg(short, long, default_value_t = 5)]
        num: usize,
    },
}


fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate { lang, count } => {
            println!("--- Genesis Engine: Morphological Engine ---");
            println!("Loading language from: {}", lang);
            
            match initialize_from_config(lang) {
                Ok((inventory, generator)) => {
                    // Generate the entire graph with 2 derivation passes.
                    let lexicon = generator.build_etymological_graph(*count, &inventory, 2);

                    println!("\n--- Final Lexicon ({} total words) ---", lexicon.graph.len());
                    for (id, lexeme) in &lexicon.graph {
                        if lexeme.parent_id.is_none() {
                            println!("[ROOT] {}: {} ({})", lexeme.form, lexeme.meaning, lexeme.part_of_speech);
                        } else {
                            let parent = lexicon.graph.get(&lexeme.parent_id.unwrap()).unwrap();
                            println!("[DERIVED] {}: {} ({}) <-- from '{}' via '{}'", lexeme.form, lexeme.meaning, lexeme.part_of_speech, parent.form, lexeme.rule_applied.as_ref().unwrap());
                        }
                    }
                }
                Err(e) => { /* ... error handling ... */ }
            }
        }
        Commands::Validate { lang } => {
            println!("Validating configuration file: {}", lang);
            

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

        Commands::Visualize { lang, count, passes, output } => {
            println!("--- Genesis Engine: Visualizer ---");
            println!("Loading language from: {}", lang);

            match initialize_from_config(lang) {
                Ok((inventory, generator)) => {
                    println!("Generating lexicon with {} roots and {} derivation passes...", count, passes);
                    let lexicon = generator.build_etymological_graph(*count, &inventory, *passes);
                    
                    println!("Exporting graph to DOT format...");
                    let dot_output = genesis_engine_lexicon::export_to_dot(&lexicon);

                    match std::fs::write(output, dot_output) {
                        Ok(_) => {
                            println!("\n✅ Success: Graph saved to '{}'", output);
                            println!("To render it, install Graphviz and run:");
                            println!("dot -Tpng {} -o lexicon.png", output);
                        },
                        Err(e) => {
                            eprintln!("\n❌ Error: Failed to write to output file.");
                            eprintln!("Reason: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("\nError: Failed to initialize generator.");
                    eprintln!("Reason: {}", e);
                }
            }
        }
            
    


        Commands::Narrate { lang, roots, passes, num } => {
                println!("--- Genesis Engine: Narrator ---");
                println!("Loading language from: {}", lang);

                match initialize_from_config(lang) {
                    Ok((inventory, generator)) => {

                        println!("Generating lexicon with {} roots and {} derivation passes...", roots, passes);
                        let lexicon = generator.build_etymological_graph(*roots, &inventory, *passes);
                        println!("Lexicon created with {} total words.", lexicon.graph.len());

                        println!("\n--- Sample Sentences ---");
                        if lexicon.graph.is_empty() {
                            println!("Lexicon is empty, cannot generate sentences.");
                        } else {
                            for i in 0..*num {
                                let sentence = generator.generate_sentence(&lexicon);
                                println!("{}. {}", i + 1, sentence);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("\nError: Failed to initialize generator.");
                        eprintln!("Reason: {}", e);
                    }
            }
        }
    }
}