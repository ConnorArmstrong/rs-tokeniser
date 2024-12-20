#![allow(unused)]

// My goal is to implement a Byte Pair Encoder in Rust
// Currently only handle lower case, unpunctuated data.

use std::fs::{self, File};
use std::collections::HashMap;
use std::path::Path;
use std::{io, time};
use rayon::prelude::*;
use std::io::{Read, Write};
use serde_json;
use std::io::{BufRead, BufReader};
use std::time::Instant;

use crate::tokeniser::Tokeniser;
use crate::visualiser::run;

mod tokeniser;
mod visualiser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut visualiser = visualiser::run();
    
    println!("Reading file...");
    let filename = "src/text8.txt"; // 124_301_826 words
    
    /*
        let file_content = fs::read_to_string(filename)?;
    let mut contents: Vec<String> = file_content // need to have a Vec<String>, where each string is a single character.
        .chars()
        .map(|c| c.to_string())
        .filter(|f| f != "\n")
        .collect();
    */

    

    //let contents = _read_words(&filename, 120_301_826);
    //println!("file read.");


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

    //run();

    /*
        println!("Size of contents: {} bytes", std::mem::size_of_val(&contents));
    let initial_vocab_path = "output/initial_vocab.json";
    
    if !Path::new(initial_vocab_path).exists() {
        let initial_vocab = initialize_vocab(&contents);
        save_initial_vocab(&initial_vocab, initial_vocab_path)?;
    }
    

    let initial_vocab = load_initial_vocab(initial_vocab_path)?;
    let (vocab, _tokenized_string) = bpe(contents, 10000, initial_vocab);

    save_vocabulary(&vocab, "output/vocabulary.json")?;

    let mut tokenizer = Tokeniser::new().unwrap();
    let before = Instant::now();
    for _ in 0..10 {
        tokenizer.tokenise(&text.to_string());
    }
    
    tokenizer.pretty_print();
    println!("\n");
    for _ in 0..10 {
        tokenizer.tokenise(&other_text.to_string());
    }
    tokenizer.pretty_print();
    println!("Elapsed time: {:.2?}", before.elapsed());
     */

    let mut tokeniser = Tokeniser::new().unwrap();
    let read_start = Instant::now();

    println!("reading string");
    let initial = _read_file_to_string(filename).unwrap();
    println!("Time to read file: {:.2?}", read_start.elapsed());

    println!("starting");
    let starting_time = Instant::now();
    let tokens = tokeniser.get_tokens_from_text(&initial);
    let tokenising_time = Instant::now();
    println!("Time to tokenise: {:.2?}", starting_time.elapsed());
    println!("tokenised.");
    let text = tokeniser.reconstruct(&tokens);
    let reconstruct_time = Instant::now();
    println!("Time to reconstruct: {:.2?}", tokenising_time.elapsed());
    //println!("{:?}", tokens);
    //println!("{:?}", text);
    println!("reconstructed.");

    println!("It took {:?} to tokenise, and {:?} to reconstruct for a total of {:?}", tokenising_time.duration_since(starting_time), reconstruct_time.duration_since(tokenising_time), reconstruct_time.duration_since(starting_time));
    
    _write_string_to_file("tokenised.txt", &format!("{:?}", tokens));
    Ok(())
}


fn bpe(mut corpus: Vec<String>, vocab_size: usize, initial_vocab: HashMap<String, i32>) -> (HashMap<String, i32>, Vec<String>) {
    println!("Beginning BPE process");

    let mut vocab = initial_vocab;
    let mut pair_count;
    let mut count = 0;
    //println!("Initial Vocab: {:?}", vocab);  // Debug print statement

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
    // Estimate the capacity to reduce rehashing
    tokens.par_windows(2)
        .fold(
            || HashMap::new(),
            |mut local_map, window| {
                let token1 = &window[0];
                let token2 = &window[1];
                *local_map.entry((token1.clone(), token2.clone())).or_insert(0) += 1;
                local_map
            }
        )
        .reduce(
            || HashMap::new(),
            |mut acc, elem| {
                for (key, value) in elem {
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
    while i < data.len() - 1 {
        if data[i] == pair.0 && data[i + 1] == pair.1 {
            data[i] = new_token.clone();
            data.remove(i + 1); // Remove the next element as it's now been merged
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
                contents.push(char.to_string().to_ascii_lowercase());
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

fn _read_file_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn _write_string_to_file<P: AsRef<Path>>(path: P, contents: &str) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(contents.as_bytes())?;
    Ok(())
}