use std::{collections::HashMap, fs, io, path::Path};
use rayon::prelude::*;
use serde_json; // Ensure serde_json is available for JSON processing
use colored::{Colorize, CustomColor};
use rand::{thread_rng, Rng};

pub type CharInfo = (String, Option<usize>);

#[derive(Default)]
pub struct Tokeniser {
    vocab: Vec<String>, // The list of tokens
    decoded: Option<Vec<String>>, // the final output
    vocab_map: HashMap<String, usize>, // mapping each string to its index
    colour_map: HashMap<usize, (u8, u8, u8)>,
}


impl Tokeniser {
    pub fn new() -> Result<Self, io::Error> {
        let initial_vocab_path = "output/vocabulary.json";

        if !Path::new(initial_vocab_path).exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Vocab file does not exist"));
        }

        let json = fs::read_to_string(initial_vocab_path)?;
        let map: HashMap<String, i32> = serde_json::from_str(&json)?;

        // Collect only the keys (tokens) from the map and sort them by length
        let mut tokens: Vec<String> = map.keys().cloned().collect();
        tokens.sort_by(|a, b| b.len().cmp(&a.len())); // sort them in decreasing order by length
        //println!("Tokens: {:?}", tokens);
    
        let vocab_map: HashMap<String, usize> = tokens
            .par_iter()
            .enumerate()
            .map(|(index, token)| (token.to_owned(), index))
            .collect();

        let mut rng = thread_rng(); // random colours for each token
        let mut colour_map: HashMap<usize, (u8, u8, u8)> = HashMap::new();

        for i in 0..tokens.len() {
            let r: u8 = rng.gen_range(0..=255);
            let g: u8 = rng.gen_range(0..=255);
            let b: u8 = rng.gen_range(0..=255);

            let colour = (r, g, b);

            colour_map.insert(i, colour);
        }

        Ok(Tokeniser {
            vocab: tokens, 
            decoded: None,
            vocab_map,
            colour_map,
        })
    }

    pub fn tokenize(&mut self, input: String) -> Vec<String> {
        if input.is_empty() {
            return Vec::new();
        }

        let input_size = input.len();

        let mut position: Vec<CharInfo> = input.par_chars()
            .filter(|&c| c != '\n')
            .map(|c| (c.to_ascii_lowercase().to_string(), None))
            .collect();


        // iterate through every token
        // slide a window of said token over position vector -- ie if token is 3 characters long look at the 3 consecutive indices
        // if theres a match add to an index vector
        // then iterate through the index vector and check that theres space for this string (ie another token isnt already in position)
        //    if true:  copy across token char by char into the position vector and update 
        //    if false: continue to the next occurence
        // continue through all tokens

        // this works because the tokens are sorted - the larger tokens filters as much as possible and all remaining tokens can be done by character
        for (location, token) in self.vocab.iter().enumerate() { // every character in the string is guarenteed to be covered by one of the tokens
            let window_size = token.len();
            
            if window_size > input_size { // if a tokens length is greater than the input its not made up of the token
                continue;
            }

            for i in 0..=position.len() - window_size { // create the window
                let window = &position[i..i + window_size]; // slide window across
                if window.iter().map(|(c, _)| c.to_owned()).collect::<Vec<_>>().join("") == *token && window.iter().all(|(_, b)| *b == None) {
                    for index in i..i + window_size {
                        position[index].1 = Some(location); // Mark with the token index
                    }
                }
            }
        }

        let mut count = 0; // Debugging and Error information
        let mut missing_vec = Vec::new();
        
        for item in position.iter() {
            let (token, value) = item;
            if value.is_none() {
                count += 1;
                missing_vec.push(token.to_owned())
            }
        }

        if !missing_vec.is_empty() { // unknown or unencoded tokens
            println!("{}", count);
            println!("{:?}", missing_vec);
        }

        let output =  self.recreate_string(&position);

        self.decoded = Some(output.clone()); // For now
        output
    }

    fn recreate_string(&self, position_vector: &[CharInfo]) -> Vec<String> {
        let mut result = Vec::new();
        let mut last_token: Option<usize> = None;

        // WARNING: for the time being this could get rid of intential successive equal tokens - add count value to the option usize ie Option<usize, usize)
        for (_, e) in position_vector {
            if let Some(num) = e {
                if Some(num) != last_token.as_ref() {
                    last_token = Some(*num);
                    result.push(num);
                }
            }
        }
        result.into_par_iter().map(|index| self.vocab[*index].clone()).collect()
    }

    pub fn pretty_print(&self) {
        if self.decoded.is_none() {
            println!("Text not yet tokenized");
            return
        }

        for token in self.decoded.as_ref().unwrap() {
            let token_index = self.vocab_map.get(token).unwrap_or(&usize::max_value()); // if token not encountered make it white
            let token_colour = self.colour_map.get(token_index).unwrap_or(&(0, 0, 0));

            print!("{}", token.as_str().custom_color(CustomColor {
                r: token_colour.0,
                g: token_colour.1,
                b: token_colour.2,
            }));
        }
        println!();
    }

    pub fn _compare_to_original(&mut self, original_string: String, tokenized_data: Vec<String>) {
        for token in tokenized_data {
            let token_index = self.vocab_map.get(&token).unwrap_or(&usize::max_value()); // if token not encountered make it white
            let token_colour = self.colour_map.get(token_index).unwrap_or(&(0, 0, 0));

            print!("{}", token.as_str().custom_color(CustomColor {
                r: token_colour.0,
                g: token_colour.1,
                b: token_colour.2,
            }));
        }
        println!();
        self.tokenize(original_string);
        self.pretty_print();
    }

    pub fn _get_back_out(&self, string: String) -> String {
        println!("calling: {}", string);
        return string.clone();
    }
    
}
