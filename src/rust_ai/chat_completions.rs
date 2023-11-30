use std::{env, io};
use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::Write;

//Constants
const COMPLETION_URL: &str = "https://api.openai.com/v1/chat/completions";

#[derive(Debug, Deserialize)]
struct ChatChunkDelta {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatChunkChoice {
    delta: ChatChunkDelta
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChunk {
    choices: Vec<ChatChunkChoice>,
}

/* 
    Text Completion with ChatGPT OpenAI API (non-streaming)
    This function takes a question as input and returns the answer from the AI.
*/
pub async fn ask_ai(question: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Create a new HTTP client
    let client = Client::new();

    // Simplify request building with chained calls
    let response = client
        .post(COMPLETION_URL)
        .bearer_auth(env::var("OPENAI_API_KEY")?)
        .json(&json!({
           "model": "gpt-4-0613",
           "messages": [{"role": "user", "content": question}],
           "temperature": 0.7
        }))
        .send()
        .await?;

    if response.status().is_success() {
        let response_json: Value = response.json().await?;

        if let Some(text) = response_json["choices"]
            .get(0)
            .and_then(|choice| choice["message"]["content"].as_str())
        {
            Ok(text.to_string())
        } else {
            Err("No text found in the response".into())
        }
    } else {
        Err(format!("Error: {:?}", response.status()).into())
    }
}

/*
    Text Completion with ChatGPT OpenAI API (streaming)
    This function takes a question as input and returns the answer from the AI.
*/
pub async fn ask_ai_streaming(question: &str) -> Result<()> {
    // Create a new HTTP client
    let client = Client::new();

    // Simplify request building with chained calls
    let mut response = client
        .post(COMPLETION_URL)
        .bearer_auth(env::var("OPENAI_API_KEY")?)
        .json(&json!({
            "model": "gpt-4-0613",
            "messages": [{"role": "user", "content": question}],
            "temperature": 0.7,
            "stream": true
        }))
        .send()
        .await?;

    // Buffer for incomplete chunks
    let mut buffer = String::new();

    // Read the response body as chunks
    while let Some(chunk) = response.chunk().await? {
        // Convert chunk bytes to string and add it to the buffer
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        // Process each line separately within the buffered data
        while let Some(pos) = buffer.find("\n\n") {
            let line = &buffer[..pos]; // Get one line from the buffer

            if line == "data: [DONE]" {
                return Ok(());
            }

            // Parse the line as JSON
            if let Some(json_data) = line.strip_prefix("data: ") {
                match serde_json::from_str::<ChatCompletionChunk>(json_data) {
                    Ok(chat_chunk) => {
                        if let Some(choice) = chat_chunk.choices.get(0) {
                            if let Some(content) = &choice.delta.content {
                                print!("{}", content);
                                io::stdout().flush()?;
                            }
                        }
                    }
                    Err(e) => eprintln!("Error parsing JSON: {}", e),
                }
            }

            // Remove the processed line from the buffer including delimiter "\n\n"
            buffer.drain(..=pos + 1);
        }
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ask_ai() {
        let question = "Say just Hello";
        let expected_answer = "Hello!"; // The expected answer might vary due to AI's nature
        let response = ask_ai(question).await;
        println!("RESPONSE {:?}", response);

        match response {
            Ok(answer) => assert_eq!(answer.trim(), expected_answer),
            Err(e) => panic!("Test failed with error: {}", e),
        }
    }

    #[tokio::test]
    async fn test_ask_ai_streaming() {
        let question = "Say just Hello";
        let is_answer_present = ask_ai_streaming(question).await.is_ok();
        assert_eq!(is_answer_present, true);
    }
}
