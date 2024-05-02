use std::{collections::HashMap, fs, io, path::Path};
use serde_json; // Ensure serde_json is available for JSON processing



pub struct Decoder {
    vocab: Vec<String>, 
    input: String, // the text to be tokenized
}

impl Decoder {
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
        tokens.sort_by(|a, b| b.len().cmp(&a.len())); // sort them in decreasing order by length
        println!("Tokens: {:?}", tokens);
    
        Ok(Decoder {
            input,
            vocab: tokens, // Store the constructed map
        })
    }

    pub fn tokenize(&self) {
        type CharInfo = (String, Option<usize>); // where the usize would be the corresponding token index in the vocab array
        
        let length = self.input.len();
        let output = String::with_capacity(length);
        
        let mut position: Vec<CharInfo> = self.input
                                    .clone()
                                    .chars()
                                    .map(|c| (c.to_string(), None))
                                    .collect();


        // iterate through every token
        // slide a window of said token over position vector -- ie if token is 3 characters long look at the 3 consecutive indices
        // if theres a match add to an index vector
        // then iterate through the index vector and check that theres space for this string (ie another token isnt already in position)
        //    if true:  copy across token char by char into the position vector and update 
        //    if false: continue to the next occurence
        // continue through all tokens

        for (location, token) in self.vocab.iter().enumerate() {
            let window_size = token.len();
            let token_chars: Vec<String> = token.chars().map(|c| c.to_string()).collect();


            for i in 0..=position.len() - window_size { // create our window
                let window = &position[i..i + window_size];
                if window.iter().map(|(c, _)| c.to_owned()).collect::<Vec<_>>().join("") == *token && window.iter().all(|(_, b)| *b == None) {
                    for index in i..i + window_size {
                        position[index].1 = Some(index); // Mark the token index
                    }
                }
            }
        }
        
        self.recreate_string(&position);
    }

    fn recreate_string(&self, position_vector: &Vec<(String, Option<usize>)>) {
        let mut length_map: HashMap<usize, String> = HashMap::new(); // Map each token index to its specified length
        for (index, token) in self.vocab.iter().enumerate() {
            length_map.insert(index, token.to_owned());
        }

        let mut result = Vec::new();

        for item in position_vector.iter().map(|(_, e)| e).collect::<Vec<_>>() {
            let num = item.expect("ERROR HANDLING CHARACTER");
            if result.last() != Some(&num) {
                result.push(num);
            }
        }

        let output: Vec<String> = result.iter().map(|element| length_map.get(element).unwrap().to_owned()).collect();
        println!("{:?}", output);
    }
    
}
