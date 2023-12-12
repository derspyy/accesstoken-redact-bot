use std::error::Error;

pub async fn redact(url: &str) -> Result<String, Box<dyn Error>> {
    let content = reqwest::get(url).await?.text().await?;
    let mut new_string = String::new();
    let words: Vec<(usize, &str)> = content.split_inclusive(' ').enumerate().collect();
    for word in words.iter().skip(1) {
        let previous_word = word.0 - 1;
        match words[previous_word].1 {
            "--accessToken " => new_string.push_str("*** "),
            _ => new_string.push_str(word.1),
        }
    }
    Ok(new_string)
}
