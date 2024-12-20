use std::{collections::HashMap, fs, io, path::Path};
use rayon::prelude::*;
use serde_json; // Ensure serde_json is available for JSON processing
use colored::{Colorize, CustomColor};
use rand::{thread_rng, Rng};
use daachorse::{CharwiseDoubleArrayAhoCorasickBuilder, MatchKind, CharwiseDoubleArrayAhoCorasick};

pub type CharInfo = (char, Option<(usize, usize)>); // Might need to make this CharInfo = (char, Option<(usize, usize))

#[derive(Default)]
pub struct Tokeniser {
    vocab: Vec<String>, // The list of tokens
    decoded: Option<Vec<String>>, // the final output
    vocab_map: HashMap<String, usize>, // mapping each string to its index
    colour_map: HashMap<usize, (u8, u8, u8)>, // maps each token to a unique colour (thats light enough to read text against)
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
        println!("Token Amount: {}", tokens.len());

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

    pub fn tokenise(&mut self, input: &String) -> Vec<String> {
        let input = input.to_ascii_lowercase(); // only trained on lowercase letters

        if input.is_empty() { // Default cases
            return Vec::new();
        } else if input.len() == 1 {
            return vec![input.to_string()];
        }

        let input_size = input.len();

        let mut position: Vec<CharInfo> = input.chars()
            .filter(|&c| c != '\n')
            .map(|c| (c, None)) 
            .collect();


        // iterate through every token
        // slide a window of said token over position vector -- ie if token is 3 characters long look at the 3 consecutive indices
        // if theres a match add to an index vector
        // then iterate through the index vector and check that theres space for this string (ie another token isnt already in position)
        //    if true:  copy across token char by char into the position vector and update 
        //    if false: continue to the next occurence
        // continue through all tokens

        // this works because the tokens are sorted - the larger tokens filters as much as possible and all remaining tokens can be done by character
        for (token_index, token) in self.vocab.iter().enumerate() { // every character in the string is guarenteed to be covered by one of the tokens
            let window_size = token.len();
            //let mut count = 0; // this would keep track of the occurences of successive tokens

            if window_size > input_size { // if a tokens length is greater than the input its not made up of the token
                continue; // for the time being I cant filter the vocab list because i need the specific index
            }

            if position.iter().all(|(_, c)| c.is_some()) { // end the token checking if all characters are accounted for
                break;
            }
            let mut count = 0;

            for i in 0..=position.len() - window_size { // create the window
                let window = &position[i..i + window_size]; // slide window across
                if window.iter().zip(token.chars()).all(|(&(c, b), t)| c == t && b.is_none()) { // check if the token matches
                    for index in i..i + window_size {
                        position[index].1 = Some((token_index, count)); // Mark with the token index
                    }
                    count += 1;
                }
            } // window.iter().map(|(c, _)| c.to_owned()).collect::<Vec<_>>().join("") == *token && window.iter().all(|(_, b)| *b == None) - 124s
        }     // window.iter().map(|&(c, _)| c).eq(token.chars()) && window.iter().all(|(_, b)| *b == None) - 800ms (current 550-600ms)

        let mut count = 0; // Debugging and Error information
        let mut missing_vec = Vec::new();
        
        for item in position.iter() {
            let (token, value) = item;
            if value.is_none() { // character not covered by the token set (currently mainly punctuation)
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


    // CharInfo = (char, Option<(usize, usize)>)
    fn recreate_string(&self, position_vector: &[CharInfo]) -> Vec<String> {
        let mut result = Vec::new();
        let mut last_token: Option<(usize, usize)> = None;

        for (_, e) in position_vector { // condense each (char, index) map to just the respective token index and count
            if let Some(num) = e {
                if Some(num) != last_token.as_ref() {
                    last_token = Some(*num);
                    result.push(num);
                }
            }
        }
        result.into_iter().map(|index| self.vocab[index.0].clone()).collect() // map the index to the token
    }

    pub fn get_tokens_from_text(&self, text: &String) -> Vec<usize> {
        // same process as tokenise()
        let input = text.to_ascii_lowercase(); // only trained on lowercase letters

        if input.is_empty() { // Default cases
            return Vec::new();
        } else if input.len() == 1 {
            return vec![*self.vocab_map.get(&input).unwrap_or(&(0 as usize))];
        }

        let input_size = input.len();

        let mut position: Vec<CharInfo> = input.chars()
            .filter(|&c| c != '\n')
            .map(|c| (c, None)) 
            .collect();

        for (token_index, token) in self.vocab.iter().enumerate() { // every character in the string is guarenteed to be covered by one of the tokens
            let window_size = token.len();
            //let mut count = 0; // this would keep track of the occurences of successive tokens

            if window_size > input_size { // if a tokens length is greater than the input its not made up of the token
                continue; // for the time being I cant filter the vocab list because i need the specific index
            }

            if position.iter().all(|(_, c)| c.is_some()) { // end the token checking if all characters are accounted for
                break;
            }

            let mut count = 0;

            for i in 0..=position.len() - window_size { // create the window
                let window = &position[i..i + window_size]; // slide window across
                if window.iter().zip(token.chars()).all(|(&(c, b), t)| c == t && b.is_none()) { // check if the token matches
                    for index in i..i + window_size {
                        position[index].1 = Some((token_index, count)); // Mark with the token index
                    }
                    count += 1;
                }
            }
        }
        // ie we now should have a completed Vec<CharInfo>

        let mut result = Vec::new();
        let mut last_token: Option<(usize, usize)> = None;

        for (_, e) in position { // condense each (char, index) map to just the respective token index and count
            if let Some(num) = e {
                if Some(num) != last_token {
                    last_token = Some(num);
                    result.push(num);
                }
            }
        }
        result.into_iter().map(|(a, _)| a).collect()
    }

    pub fn reconstruct(&self, tokens: &[usize]) -> String {
        /// Similar process to recreate_string()  but this is for reconstructing tokens called from outside the tokeniser
        tokens.into_iter().map(|index| self.vocab[*index].clone()).collect()
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
        self.tokenise(&original_string);
        self.pretty_print();
    }   
}
