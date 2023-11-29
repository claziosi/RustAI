#![allow(dead_code)]

use std::{env, io};

use anyhow::Result;
use futures_util::StreamExt;
use reqwest::{Client, header};
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::Write;

#[derive(Debug, Deserialize)]
struct ChatChunkDelta {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatChunkChoice {
    delta: ChatChunkDelta,
    index: usize,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChunk {
    id: String,
    object: String,
    created: usize,
    model: String,
    choices: Vec<ChatChunkChoice>,
}

// Text Completion with ChatGPT OpenAI API
pub async fn ask_ai(question: &str) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY")?;
    let endpoint_url = "https://api.openai.com/v1/chat/completions";

    let client = Client::new();

    // Simplify request building with chained calls
    let response = client
        .post(endpoint_url)
        .bearer_auth(api_key)
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

pub async fn ask_ai_streaming(question: &str) -> Result<()> {
    let url = "https://api.openai.com/v1/chat/completions";
    let api_key = env::var("OPENAI_API_KEY")?;

    let body = json!({
        "model": "gpt-3.5-turbo",
        "messages": [{
            "role": "user",
            "content": question
        }],
        "stream": true,
    });

    let client = Client::new();

    let mut res = client
        .post(url)
        .json(&body)
        .header(header::CONTENT_TYPE, "application/json")
        .bearer_auth(api_key)
        .send()
        .await?;

    println!("ChatGPT Says:\n");

    // Buffer for incomplete chunks
    let mut buffer = String::new();

    while let Some(chunk) = res.chunk().await? {
        // Convert chunk bytes to string and add it to the buffer
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        // Process each line separately within the buffered data
        while let Some(pos) = buffer.find("\n\n") {
            let line = &buffer[..pos]; // Get one line from the buffer

            if line == "data: [DONE]" {
                println!("\n[Done.]");
                return Ok(());
            }

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

fn parse_chunk(chunk: &[u8]) -> Result<Option<ChatCompletionChunk>> {
    // Convert bytes to &str
    let text_chunk = std::str::from_utf8(chunk)?;
    print!("{}", text_chunk);
    // Process each line separately to handle DONE signal and data chunks
    for line in text_chunk.lines() {
        // Check if we've reached the end of the stream
        if line == "[DONE]" {
            return Ok(None);
        }

        // Parse JSON data prefixed with 'data: '
        if let Some(json_data) = line.strip_prefix("data: ") {
            return Ok(Some(serde_json::from_str(json_data)?));
        }
    }
    return Ok(None); // No relevant data in this chunk or not finished yet
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
        let question = "Who is Charlemagne?";
        let is_answer_present = ask_ai_streaming(question).await.is_ok();
        assert_eq!(is_answer_present, true);
    }
}
