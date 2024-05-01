use std::{collections::HashMap, fs, io, path::Path};
use fst::{automaton::{Str, Subsequence}, IntoStreamer, Map, MapBuilder, Streamer}; // Import fst-related items
use serde_json; // Ensure serde_json is available for JSON processing

pub struct Tokenizer {
    vocab: Map<Vec<u8>>, // Using fst::Map to store vocabulary
    input: String, // the text to be tokenized
}

impl Tokenizer {
    pub fn new(input: String) -> Result<Self, io::Error> {
        let initial_vocab_path = "output/vocabulary.json";

        if !Path::new(initial_vocab_path).exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Vocab file does not exist"));
        }

        let json = fs::read_to_string(initial_vocab_path)?;
        let map: HashMap<String, i32> = serde_json::from_str(&json)?;

        // Collect only the keys from the map and sort them
        let mut tokens: Vec<String> = map.keys().cloned().collect();
        //println!("Tokens")
        tokens.sort_unstable(); // Sort the tokens lexicographically
        println!("Tokens: {:?}", tokens);
        // Creating a MapBuilder to build an fst::Map
        let mut builder = MapBuilder::memory();
        for token in tokens {
            println!("Inserting \"{}\"", token);
            builder.insert(&token, 0); // Insert each token with a placeholder value
        }

        let fst_map = builder.into_map(); // Construct the fst::Map
        println!("{}", fst_map.len());
        Ok(Tokenizer {
            input,
            vocab: fst_map, // Store the constructed map
        })
    }

    // Function to tokenize input string using fst Map
    pub fn tokenize(&self) -> Vec<String> {
        let mut results = Vec::new();
        let automaton = Subsequence::new(&self.input);
        let mut stream = self.vocab.search(automaton).into_stream();
        /*
        let mut test_stream = self.vocab.stream();
            while let Some((key, _)) = test_stream.next() {
                println!("Stored key: '{}'", String::from_utf8_lossy(key));
        }
        */


        while let Some((token, _)) = stream.next() {
            if let Ok(matched_str) = std::str::from_utf8(token) {
                results.push(matched_str.to_string());
            }
        }

        results
    }
}

/*
fn main() {
    let input = "test input with various words including vocabulary words".to_string();
    let tokenizer = Tokenizer::new(input).expect("Failed to create tokenizer");

    let tokens = tokenizer.tokenize();
    println!("Tokens found: {:?}", tokens);
}
*/
