#![allow(unused)]

// My goal is to implement a Byte Pair Encoder in Rust
// Currently only handle lower case, unpunctuated data.
use std::fs::{self, File};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::{io, mem};
use rayon::prelude::*;
use std::io::Write;
use serde_json;
use serde::{Serialize, Deserialize};


fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Reading file...");
    let filename = "src/text8.txt";
    let file_content = fs::read_to_string(filename)?;
    let mut contents: Vec<String> = file_content // need to have a Vec<String>, where each string is a single character.
        .chars()
        .map(|c| if c == ' ' { "_".to_string() } else { c.to_string()})
        .collect();
    println!("file read.");

    println!("Size of contents: {} bytes", std::mem::size_of_val(&contents));
    let initial_vocab_path = "output/initial_vocab.json";
    /* 
    
    if !Path::new(initial_vocab_path).exists() {
        let initial_vocab = initialize_vocab(&contents);
        save_initial_vocab(&initial_vocab, initial_vocab_path)?;
    }
    */

    let initial_vocab = load_initial_vocab(initial_vocab_path)?;
    let vocab = bpe(contents, 10000, initial_vocab);

    save_vocabulary(&vocab, "output/vocabulary.json")?;

    Ok(())
}


fn bpe(mut corpus: Vec<String>, vocab_size: usize, initial_vocab: HashMap<String, i32>) -> HashMap<String, i32> {
    println!("Beginning BPE process");

    let mut vocab = initial_vocab;
    let mut pair_count;
    let mut count = 0;
    println!("vocab: {:?}", vocab);  // Debug print statement

    while vocab.len() < vocab_size {
        pair_count = count_adjacent_pairs(&corpus);
        //println!("pair_count: {:?}", pair_count);
        
        if let Some(best_pair) = find_most_frequent_pair(&pair_count) {
            println!("Merging {} {}", best_pair.0, best_pair.1);
            corpus = merge_pair(best_pair, &mut vocab, corpus);
        } else {
            println!("No best pair found");
            break;
        }

        count += 1;

        if (count % 50 == 0)  {
            println!("Iterations: {}, Vocab Size: {}", count, vocab.len());
        }
    }

    println!("Finished iterations");
    println!("Count: {}", count);
    println!("Vocabulary: {:?}", vocab);
    //println!("Tokenized Data: {:?}", corpus);

    vocab
}

fn count_adjacent_pairs(tokens: &Vec<String>) -> HashMap<(&String, &String), i32> {
    let mut pair_count = HashMap::new();

    // Iterate over the tokens using a sliding window of size 2
    for window in tokens.windows(2) {
        let token1 = window[0].to_string();
        let token2 = window[1].to_string();

        // Increment the count for the pair (token1, token2)
        *pair_count.entry((&window[0], &window[1])).or_insert(0) += 1;
    }

    pair_count
}


fn find_most_frequent_pair<'a>(pair_count: &'a HashMap<(&'a String, &'a String), i32>) -> Option<(&'a String, &'a String)> {
    pair_count.par_iter()
        .max_by_key(|&(_, &count)| count)
        .map(|(pair, _)| pair.clone())
}

fn merge_pair(pair: (&String, &String), vocab: &mut HashMap<String, i32>, mut data: Vec<String>) -> Vec<String> {
    let new_token = format!("{}{}", pair.0, pair.1);
    let count_token1 = vocab.remove(pair.0).unwrap_or(0);
    let count_token2 = vocab.remove(pair.1).unwrap_or(0);
    vocab.insert(new_token.clone(), count_token1 + count_token2);

    let mut new_data = Vec::new();
    let mut i = 0;
    while i < data.len() {
        if i + 1 < data.len() && &data[i] == pair.0 && &data[i + 1] == pair.1 {
            new_data.push(new_token.clone());
            i += 2; // Skip the next element as it's part of the merged pair
        } else {
            new_data.push(data[i].clone());
            i += 1;
        }
    }
    new_data
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


fn initialize_vocab(data: &str) -> HashMap<String, i32> {
    let mut vocab = HashMap::new();
    for char in data.chars() {
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