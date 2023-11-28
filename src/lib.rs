use std::env;

use reqwest::Client;
use serde_json::{json, Value};
use futures_util::StreamExt; // Required for using `.next()` on streams

use std::error::Error;

// Text Completion with ChatGPT OpenAI API 
pub async fn ask_ai(question: &str) -> Result<String, Box<dyn std::error::Error>>{
    
    let api_key = env::var("OPENAI_API_KEY")?;
    let endpoint_url = "https://api.openai.com/v1/chat/completions";

    let client = Client::new();
    
     // Simplify request building with chained calls 
     let response = client.post(endpoint_url)
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
        
        if let Some(text) = response_json["choices"].get(0)
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

pub async fn ask_ai_streaming(question: &str) -> Result<String, Box<dyn Error>> {
    let api_key = env::var("OPENAI_API_KEY")?;

    let endpoint_url = "https://api.openai.com/v1/chat/completions";

    let client = Client::new();

    let response = client.post(endpoint_url)
        .bearer_auth(api_key)
        .json(&json!({
            "model": "gpt-4-0613",
            "messages": [{"role": "user", "content": question}],
            "temperature": 0.7,
            "stream": true // Enable streaming
        }))
        .send()
        .await?;

    if response.status().is_success() {
        let mut complete_response_text = String::new();

        // Use the `.bytes_stream()` method from Reqwest's Response
        let mut byte_stream = response.bytes_stream();

        while let Some(item) = byte_stream.next().await {
            let bytes = item?;
            complete_response_text.push_str(&String::from_utf8_lossy(&bytes));
            
            // Process each part of the response as it arrives...
            
            // In this simple example, we are just collecting all parts into one string.
        }

        Ok(complete_response_text)
        
    } else {
        Err(format!("Error: {:?}", response.status()).into())
    }
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
        let expected_answer = "Hello!"; // The expected answer might vary due to AI's nature
        let response = ask_ai_streaming(question).await;
        println!("RESPONSE {:?}", response);

        match response {
            Ok(answer) => assert_eq!(answer.trim(), expected_answer),
            Err(e) => panic!("Test failed with error: {}", e),
        }
    }
}

