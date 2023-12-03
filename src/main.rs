mod rust_ai;
use std::path::Path;

use crate::rust_ai::chat::{ask_ai, ask_ai_streaming};
use crate::rust_ai::audio::{speech_to_text, text_to_speech};
use tokio;

#[tokio::main]
async fn main() {
    // The question you want to ask the AI
    // Ask to enter a question
    println!("Enter a question: ");
    let mut question = String::new();
    std::io::stdin().read_line(&mut question).unwrap();

    // Call the ask_ai function and await its result
    match ask_ai(&question).await {
        Ok(response) => {
            println!("{:?}", response)
        }
        Err(e) => eprintln!("An error occurred: {}", e),
    }

    println!("Streamed version: ");

    // Call the ask_ai_streaming function and await its result
    match ask_ai_streaming(&question).await {
        Ok(response) => {
            println!("{:?}", response)
        }
        Err(e) => eprintln!("An error occurred: {}", e),
    }

    let file_path = Path::new("./python/question.m4a");

    match speech_to_text(&file_path).await {
        Ok(response) => {
            println!("{:?}", response)
        }
        Err(e) => eprintln!("An error occurred: {}", e),
    }

    // Call our function with example parameters.
    match text_to_speech("The quick brown fox jumped over the lazy dog.", "alloy").await {
        Ok(path) => println!("Text-to-Speech audio saved to {:?}", path),
        Err(e) => println!("An error occurred: {}", e),
    }

}
