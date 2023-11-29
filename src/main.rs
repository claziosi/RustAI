use lib_ai::{ask_ai, ask_ai_streaming};
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
}
