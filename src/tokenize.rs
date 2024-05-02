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
        //println!("Tokens: {:?}", tokens);
        // Creating a MapBuilder to build an fst::Map
        let mut builder = MapBuilder::memory();
        let mut count = 0;
        for token in tokens {
            builder.insert(&token, count); // Insert each token with a placeholder value
            count += 1;
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
        todo!();
        let mut results = Vec::new();
        let mut position = 0;
    
        while position < self.input.len() {
            let slice = &self.input[position..];
            let mut longest_match = None;
            let mut longest_length = 0;
    
            {
                let mut stream = self.vocab.range().into_stream();
                while let Some((token, _)) = stream.next() {
                    let token_str = match std::str::from_utf8(token) {
                        Ok(s) => s,
                        Err(_) => continue,
                    };
    
                    if slice.starts_with(token_str) && token_str.len() > longest_length {
                        longest_match = Some(token_str);
                        longest_length = token_str.len();
                    }
                }
            }
    
            if let Some(match_str) = longest_match {
                results.push(match_str.to_string());
                position += longest_length;
            } else {
                position += 1;
            }
        }
    
        results
    }
}
