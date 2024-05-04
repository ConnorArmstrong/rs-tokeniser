//#![allow(unused)]

// My goal is to implement a Byte Pair Encoder in Rust
// Currently only handle lower case, unpunctuated data.

use std::fs::{self, File};
use std::collections::HashMap;
use std::path::Path;
use std::io;
use rayon::prelude::*;
use std::io::Write;
use serde_json;
use std::io::{BufRead, BufReader};

use crate::tokeniser::Tokeniser;

mod tokeniser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    
    println!("Reading file...");
    let filename = "src/text8.txt"; // 124_301_826 words
    /*
        let file_content = fs::read_to_string(filename)?;
    let contents: Vec<String> = file_content // need to have a Vec<String>, where each string is a single character.
        .chars()
        .map(|c| c.to_string())
        .filter(|f| f != "\n")
        .collect();
    */

    let contents = _read_words(&filename, 1000000);
    println!("file read.");
    

    let text = "the quick brown fox jumped over the lazy dog and that was just the beginning of the tale 
        it told of its adventures throughout the forest the fox always loved to explore and discover new places and 
        today was no different as it made its way through the underbrush it came across many other creatures some 
        were fast and others were slow but all were a part of the vibrant ecosystem that the fox called home the sun was 
        high in the sky casting a warm glow over the land as the fox continued its journey it thought about the many days it had spent 
        roaming this terrain each day brought new surprises and challenges that kept the fox on its toes and as the light began to fade the fox found 
        its way back to its den settled in for the night and dreamed of the next days adventures";

    let other_text = "the zebra found a xylophone one sunny day when wandering near the edge of the meadow curious about the 
        strange object with colorful bars the zebra tapped on it gently with its hoof the sound that came out was magical and unlike 
        anything it had ever heard before delighted the zebra played a tune although it didnt know exactly how to play music it managed 
        to make a joyful melody that echoed throughout the savannah other animals gathered around drawn by the unique sounds even the birds 
        paused their singing to listen the zebra felt a surge of happiness as it shared this new discovery with its friends and as the sun set the zebra 
        knew it had found a new way to express its joy and creativity through the enchanting sounds of the xylophone"
            .replace("\n", "");

    /*
        let contents: Vec<String> = text
        .chars()
        .map(|c| c.to_string())
        .filter(|f| f != "\n")
        .collect();
    */


    println!("Size of contents: {} bytes", std::mem::size_of_val(&contents));
    let initial_vocab_path = "output/initial_vocab.json";
    
    if !Path::new(initial_vocab_path).exists() {
        let initial_vocab = initialize_vocab(&contents);
        save_initial_vocab(&initial_vocab, initial_vocab_path)?;
    }
    

    let initial_vocab = load_initial_vocab(initial_vocab_path)?;
    let (vocab, _tokenized_string) = bpe(contents, 55000, initial_vocab);

    save_vocabulary(&vocab, "output/vocabulary.json")?;

    let mut tokenizer: Tokeniser = Tokeniser::new().unwrap();


    //tokenizer.compare_to_original(text.to_string(), _tokenized_string);

    tokenizer.tokenize(text.to_string());
    tokenizer.pretty_print();
    println!("\n");
    tokenizer.tokenize(other_text.to_string());
    tokenizer.pretty_print();

    Ok(())
}


fn bpe(mut corpus: Vec<String>, vocab_size: usize, initial_vocab: HashMap<String, i32>) -> (HashMap<String, i32>, Vec<String>) {
    println!("Beginning BPE process");

    let mut vocab = initial_vocab;
    let mut pair_count;
    let mut count = 0;
    println!("Initial Vocab: {:?}", vocab);  // Debug print statement

    let corpus_ptr = &mut corpus as *mut Vec<String>; // Get a raw pointer to corpus to sidepass the borrow checker

    while vocab.len() < vocab_size {
        pair_count = count_adjacent_pairs(&corpus);
        //println!("pair_count: {:?}", pair_count);
        
        if let Some(best_pair) = find_most_frequent_pair(&pair_count) {
            println!("Merging \"{}\" \"{}\"", best_pair.0, best_pair.1);
            unsafe { // TODO: -- restructure this in the future! --
                merge_pair(best_pair, &mut vocab, &mut *corpus_ptr); // raw pointer shenanigans
            }
        } else {
            println!("No best pair found"); // most likely indicates an error
            break;
        }

        count += 1;

        if count % 50 == 0  {
            println!("Iterations: {}, Vocab Size: {}", count, vocab.len());
        }
    }

    //println!("Finished iterations");
    //println!("Count: {}", count);
    //println!("Vocabulary: {:?}", vocab);
    println!("Tokenized Data: {:?}", corpus);

    (vocab, corpus)
}

fn count_adjacent_pairs(tokens: &[String]) -> HashMap<(String, String), i32> {
    tokens.par_windows(2)
        .map(|window| {
            let token1 = window[0].clone();
            let token2 = window[1].clone();
            let mut local_map = HashMap::new();
            *local_map.entry((token1, token2)).or_insert(0) += 1;
            local_map
        })
        .reduce(
            || HashMap::new(),
            |mut acc, mut elem| {
                for (key, value) in elem.drain() {
                    *acc.entry(key).or_insert(0) += value;
                }
                acc
            }
        )
}

fn find_most_frequent_pair(pair_count: &HashMap<(String, String), i32>) -> Option<(String, String)> {
    pair_count.par_iter()
    .max_by_key(|&(_, &count)| count)
    .map(|(pair, _)| pair.clone())
}

fn merge_pair(pair: (String, String), vocab: &mut HashMap<String, i32>, data: &mut Vec<String>) {
    let new_token = format!("{}{}", pair.0, pair.1);

    let count_token1 = vocab.get(&pair.0).unwrap();
    let count_token2 = vocab.get(&pair.1).unwrap();

    vocab.insert(new_token.clone(), count_token1 + count_token2);

    let mut i = 0;
    while i < data.len() {
        if i + 1 < data.len() && data[i] == pair.0 && data[i + 1] == pair.1 {
            data[i] = new_token.clone();
            data.remove(i + 1); // Remove the next element as it's part of the merged pair
        } else {
            i += 1;
        }
    }
}

fn save_vocabulary(vocab: &HashMap<String, i32>, file_path: &str) -> io::Result<()> {
    // Ensure the directory exists
    let path = Path::new(file_path);
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }

    let file = File::create(path);
    match file {
        Ok(mut f) => {
            let json = serde_json::to_string_pretty(vocab).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            f.write_all(json.as_bytes())?;
            Ok(())
        },
        Err(e) => Err(e),
    }
}

fn initialize_vocab(data: &Vec<String>) -> HashMap<String, i32> {
    let mut vocab = HashMap::new();
    for char in data {
        *vocab.entry(char.to_string()).or_insert(0) += 1;
    }
    vocab
}

fn save_initial_vocab(vocab: &HashMap<String, i32>, file_path: &str) -> io::Result<()> {
    let path = Path::new(file_path);
    let file = File::create(path);
    match file {
        Ok(mut f) => {
            let json = serde_json::to_string_pretty(vocab).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            f.write_all(json.as_bytes())?;
            Ok(())
        },
        Err(e) => Err(e),
    }
}

fn load_initial_vocab(file_path: &str) -> io::Result<HashMap<String, i32>> {
    let json = fs::read_to_string(file_path)?;
    let vocab: HashMap<String, i32> = serde_json::from_str(&json).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(vocab)
}

fn _read_words(file_path: &str, word_count: usize) -> Vec<String> {
    // Open the file
    let file = File::open(file_path).expect("Unable to open the file");
    let reader = BufReader::new(file);
    
    // Initialize a vector to store the characters
    let mut contents = Vec::new();
    let mut total_words = 0;

    // Iterate through lines
    for line_result in reader.lines() {
        let line = line_result.expect("Unable to read line");

        // Split the line into words using whitespace as the delimiter
        for word in line.split_whitespace() {
            total_words += 1;

            // Convert each word to characters and add spaces between words
            for char in word.chars() {
                contents.push(char.to_string());
            }
            contents.push(" ".to_string()); // Add a space after each word
            
            // Stop if we've reached the desired word count
            if total_words >= word_count {
                break;
            }
        }

        // Stop the outer loop if we've reached the desired word count
        if total_words >= word_count {
            break;
        }
    }

    // Remove the last space added if it exists
    if let Some(last) = contents.last() {
        if last == " " {
            contents.pop();
        }
    }

    contents
}