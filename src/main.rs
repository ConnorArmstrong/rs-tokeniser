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
    let filename = "src/text8.txt";
    let contents = std::fs::read_to_string(filename).unwrap().replace(" ", "_");
    println!("Size of contents: {} bytes", std::mem::size_of_val(&contents));

    let initial_vocab_path = "output/initial_vocab.json";
    if !Path::new(initial_vocab_path).exists() {
        let initial_vocab = initialize_vocab(&contents);
        save_initial_vocab(&initial_vocab, initial_vocab_path)?;
    }

    let initial_vocab = load_initial_vocab(initial_vocab_path)?;
    let vocab = bpe(contents, 10000, initial_vocab);

    save_vocabulary(&vocab, "output/vocabulary.json")?;

    Ok(())
}


fn bpe(corpus: String, vocab_size: usize, initial_vocab: HashMap<String, i32>) -> HashMap<String, i32> {
    let mut corpus = corpus.replace(" ", "_");  // Use underscore to represent space
    let mut vocab = initial_vocab;
    let mut pair_count;
    let mut count = 0;
    println!("vocab: {:?}", vocab);  // Debug print statement

    while vocab.len() < vocab_size {
        pair_count = count_pairs(&vocab);
        //println!("pair_count: {:?}", pair_count);
        
        if let Some(best_pair) = find_most_frequent_pair(&pair_count) {
            println!("Merging {} {}", best_pair.0, best_pair.1);
            merge_pair(&best_pair, &mut vocab, &mut corpus);
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
fn initialize_vocab(data: &str) -> HashMap<String, i32> {
    let mut vocab = HashMap::new();
    for char in data.chars() {
        *vocab.entry(char.to_string()).or_insert(0) += 1;
    }
    vocab
}

fn load_initial_vocab(file_path: &str) -> io::Result<HashMap<String, i32>> {
    let json = fs::read_to_string(file_path)?;
    let vocab: HashMap<String, i32> = serde_json::from_str(&json).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(vocab)
}

fn count_pairs(vocab: &HashMap<String, i32>) -> HashMap<(String, String), i32> {
    let mut pair_count = HashMap::new();

    for (token1, &count1) in vocab {
        for (token2, &count2) in vocab {
            let pair = (token1.clone(), token2.clone());
            *pair_count.entry(pair).or_insert(0) += count1 * count2;
        }
    }

    pair_count
}


fn find_most_frequent_pair(pair_count: &HashMap<(String, String), i32>) -> Option<(String, String)> {
    pair_count.iter()
        .max_by_key(|&(_, &count)| count)
        .map(|(pair, _)| pair.clone())
}

fn merge_pair(pair: &(String, String), vocab: &mut HashMap<String, i32>, data: &mut String) {
    let new_token = format!("{}{}", pair.0, pair.1);

    // Check if the new token exceeds a certain length threshold
    // Replace all occurrences of the pair in the corpus with the new token
    let re = regex::Regex::new(&format!("{}{}", regex::escape(&pair.0), regex::escape(&pair.1))).unwrap();
    let count = re.find_iter(&*data).count() as i32;
    *data = re.replace_all(&*data, &new_token).to_string();

    // Update the vocabulary counts for the merged token
    *vocab.entry(new_token.clone()).or_insert(0) += count;

    // Update the counts of the individual characters
    if let Some(entry_0) = vocab.get_mut(&pair.0) {
        *entry_0 -= count;
    }
    if let Some(entry_1) = vocab.get_mut(&pair.1) {
        *entry_1 -= count;
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