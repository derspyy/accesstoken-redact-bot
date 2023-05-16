use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use std::io::prelude::*;

pub fn redact(url: &str, filename: &PathBuf) -> Result<(), Box<dyn Error>> {
    let content = ureq::get(url).call()?.into_string()?;
    let mut new_string = String::new();
    let words: Vec<(usize, &str)> = content.split_inclusive(' ').enumerate().collect();
    for word in words.iter().skip(1) {
        let previous_word = word.0 - 1;
        match words[previous_word].1  {
            "--accessToken " => new_string.push_str("*** "),
            _ => new_string.push_str(word.1),
        }
    }
    let mut file = File::create(filename)?;
    file.write_all(new_string.as_bytes())?;
    Ok(())
}