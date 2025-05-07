use anyhow::{anyhow, Result};
use dotenvy::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Serialize)]
struct ChunkRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize, Debug)]
struct ChunkResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: Message,
}

pub struct HuggingFaceClient;

impl HuggingFaceClient {
    pub fn new() -> Self {
        HuggingFaceClient
    }
    pub async fn chunk_markdown(
        &self,
        client: &Client,
        api_key: &str,
        markdown: &str,
    ) -> Result<Vec<String>> {
        retry_with_backoff(|| async {
        let request = ChunkRequest {
            model: "meta-llama/Llama-3.3-70B-Instruct".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are a helpful assistant that chunks markdown content into logical sections. Each chunk should be a complete thought or section.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: format!("Please chunk the following markdown into logical sections. Separate each section with '---':\n\n{}", markdown),
                },
            ],
            temperature: 0.7,
        };

        let response = client
            .post("https://api-inference.huggingface.co/models/meta-llama/Llama-3.3-70B-Instruct/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("API request failed: {}", error_text));
        }

        let chunk_response: ChunkResponse = response.json().await?;

        let content = chunk_response.choices
            .first()
            .ok_or_else(|| anyhow!("No choices in response"))?
            .message
            .content
            .clone();

        // Diviser le contenu en chunks en utilisant le séparateur '---'
        let chunks: Vec<String> = content
            .split("---")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if chunks.is_empty() {
            Ok(vec![content]) // Retourner le contenu entier si aucun chunk n'est trouvé
        } else {
            Ok(chunks)
        }
        }).await
    }
}

async fn retry_with_backoff<F, Fut, T>(mut f: F) -> Result<T, anyhow::Error>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, anyhow::Error>>,
{
    let mut attempts = 0;
    let max_attempts = 3;
    let mut delay = Duration::from_millis(100);

    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if attempts < max_attempts => {
                println!("Attempt {} failed: {}. Retrying...", attempts + 1, e);
                attempts += 1;
                sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }
}

#[tokio::test]
async fn test_chunk() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;
    let api_key = std::env::var("HF_API_KEY").expect("HF_API_KEY must be set");
    let markdown = "# Petit Exemple Markdown

## Section Principale

- **Liste à puces**
  * Élément 1
  * Élément 2
  * Élément 3

### Caractéristiques

`code.inline()` et :

```rust
fn main() {
    println!(Hello World!);
}";
    println!("Markdown: {}", markdown);
    let hf_client = HuggingFaceClient::new();
    let chunks = hf_client
        .chunk_markdown(&client, &api_key, markdown)
        .await?;

    println!("Number of chunks: {}", chunks.len());
    for (i, chunk) in chunks.iter().enumerate() {
        println!("Chunk {}: {}", i + 1, chunk);
    }

    assert!(!chunks.is_empty(), "Expected at least one chunk");
    Ok(())
}
