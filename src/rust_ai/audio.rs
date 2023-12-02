use std::path::Path;

use reqwest::{Client, multipart};
use tokio;

// Define structures for deserializing response JSON.
// Note: The actual structure may vary depending on OpenAI's API response format.
#[derive(serde::Deserialize)]
struct TranscriptionResponse {
   text: String,
}

const API_URL: &str = "https://api.openai.com/v1/audio/transcriptions";

pub async fn transcription(file_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let model_name = "whisper-1";

    // Read the file content into a byte vector
    let file_content = tokio::fs::read(file_path).await?;

    // Create a multipart form
    let part = multipart::Part::bytes(file_content)
        .file_name("audio.m4a")
        .mime_str("audio/m4a")?;  // Make sure to set the correct MIME type for your audio file

    let form = multipart::Form::new()
        .part("file", part)
        .text("model", model_name.to_string());

    // Build client and make the request
    let client = Client::new();
    
    let response = client.post(API_URL)
        .bearer_auth(api_key)
        .multipart(form)
        .send()
        .await?;

    if response.status().is_success() {
        if let Ok(transcription_response) = response.json::<TranscriptionResponse>().await {
            return Ok(transcription_response.text.clone());
        } else {
            Err("Failed to parse JSON response".into())
        }
        
    } else {
        Err(format!("Error making request: {:?}", response.status()).into())
    }
}


