use std::{path::{Path, PathBuf}, error::Error, fs::File, io::Write};

use reqwest::{Client, multipart};
use tokio;

// Define structures for deserializing response JSON.
// Note: The actual structure may vary depending on OpenAI's API response format.
#[derive(serde::Deserialize)]
struct TranscriptionResponse {
   text: String,
}

// Define a structure for the request body.
#[derive(serde::Serialize)]
struct TextToSpeechRequest {
    model: String,
    input: String,
    voice: String,
}

const API_URL: &str = "https://api.openai.com/v1/audio/transcriptions";

pub async fn speech_to_text(file_path: &Path) 
-> Result<String, Box<dyn std::error::Error>> {
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


pub async fn text_to_speech(input_text: &str, voice: &str,
) -> Result<PathBuf, Box<dyn Error>> {
    
    const API_URL: &str = "https://api.openai.com/v1/audio/speech";
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");

    // Prepare the request body.
    let body = TextToSpeechRequest {
        model: "tts-1".to_string(),
        input: input_text.to_string(),
        voice: voice.to_string(),
    };

    // Create an HTTP client instance.
    let client = Client::new();

    // Perform the POST request.
    let response_bytes = client
        .post(API_URL)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await?
        .error_for_status()? // Ensure we have a successful response code (e.g., 2xx).
        .bytes()
        .await?;

    // Write the received bytes into an MP3 file.
    let output_path = PathBuf::from("speech.mp3");
    let mut file = File::create(&output_path)?;
    
    file.write_all(&response_bytes)?;

    Ok(output_path)
}

