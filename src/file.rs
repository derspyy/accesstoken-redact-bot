use ureq;
use std::fs::File;
use std::fs::create_dir;
use std::path::PathBuf;
use std::io::prelude::*;

pub fn redact(url: &String, filename: &PathBuf) -> Result<(), ureq::Error> {
    let content = ureq::get(&url).call()?.into_string()?;
    let words = content.split_inclusive(" ");
    let mut word_vec: Vec<&str>  = content.split_inclusive(" ").collect();
    for word in words.enumerate() {
        if word.1 == "--accessToken " {
            let token = word.0 + 1;
            word_vec[token] = "*** ";
        }
    }
    let mut new_string = String::new();
    for word in word_vec {
        new_string.push_str(word);
    }
    let mut file = File::create(filename).unwrap();
    file.write_all(new_string.as_bytes())?;
    Ok(())
}