# Rust gRPC Embedder Service

A high-performance embedding service built with Rust, providing text embeddings via gRPC. This service leverages both FastEmbed for embedding generation and Hugging Face's Llama-3.3-70B-Instruct model for intelligent markdown chunking.

## Overview

This service provides a gRPC API for text embedding generation and markdown chunking, allowing clients to send text chunks or markdown documents and receive vector embeddings in response. It's designed for high throughput and low latency, making it suitable for production environments.

## Features

- Fast text embedding generation using FastEmbed models
- Intelligent markdown chunking using Hugging Face's Llama-3.3-70B-Instruct model
- gRPC API for efficient communication
- Dockerized deployment for easy scaling
- Robust error handling with retry mechanisms and exponential backoff
- Support for batched embedding requests

## Architecture

The service is built using:

- **Tonic**: A Rust implementation of gRPC
- **FastEmbed**: A lightweight embedding model from LangChain
- **Hugging Face API**: For intelligent text chunking using Llama-3.3-70B-Instruct
- **Tokio**: Asynchronous runtime for Rust
- **Reqwest**: HTTP client for API requests
- **Anyhow**: For error handling

## Getting Started

### Prerequisites

- Rust 1.70+ (2021 edition)
- Protobuf compiler (protoc)
- Docker (for containerized deployment)
- Hugging Face API key (for accessing Llama-3.3-70B-Instruct)

### Environment Setup

Create a `.env` file with the following variables:

```
HF_API_KEY=your_huggingface_api_key_here
```

### Installation

1. Clone the repository:

```bash
git clone https://github.com/salim4n/rust-grpc-embedder.git
cd rust-grpc-embedder
```

2. Build the project:

```bash
cargo build --release
```

3. Run the service:

```bash
cargo run --release
```

The service will start on port 50051 by default.

### Docker Deployment

Build and run using Docker:

```bash
docker build -t rust-grpc-embedder .
docker run -p 50051:50051 --env-file .env rust-grpc-embedder
```

## Usage

### gRPC API

The service exposes the following gRPC endpoints:

```protobuf
service ChunkEmbed {
  rpc ChunkEmbedMessage(ChunkEmbedRequest) returns (ChunkEmbedResponse);
  rpc EmbedMarkdown(EmbedMarkdownRequest) returns (EmbedMarkdownResponse);
}
```

Where:

- `ChunkEmbedRequest` contains text chunks to embed
- `ChunkEmbedResponse` returns vector embeddings
- `EmbedMarkdownRequest` contains markdown text to chunk and embed
- `EmbedMarkdownResponse` returns chunked text with corresponding embeddings

### Example Client

Here's an example of how to call the service from a Rust client:

```rust
use chunk_embed::chunk_embed_client::ChunkEmbedClient;
use chunk_embed::{ChunkEmbedRequest, EmbedMarkdownRequest};
use tonic::Request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ChunkEmbedClient::connect("http://[::1]:50051").await?;
    
    // Embed specific chunks
    let request = Request::new(ChunkEmbedRequest {
        chunks: vec![
            "This is the first text to embed".to_string(),
            "And here is another example".to_string(),
        ],
    });
    
    let response = client.chunk_embed_message(request).await?;
    println!("CHUNK RESPONSE: {:?}", response);
    
    // Embed a markdown document
    let markdown_request = Request::new(EmbedMarkdownRequest {
        markdown: "# Example Markdown\n\nThis is a sample markdown document.".to_string(),
    });
    
    let markdown_response = client.embed_markdown(markdown_request).await?;
    println!("MARKDOWN RESPONSE: {:?}", markdown_response);
    
    Ok(())
}
```

## Implementation Details

### HuggingFace Client

The `HuggingFaceClient` is responsible for chunking markdown documents into logical sections using Hugging Face's Llama-3.3-70B-Instruct model:

- **Intelligent Chunking**: Uses the LLM to understand document structure and create semantically meaningful chunks
- **Retry Mechanism**: Implements exponential backoff for API call retries
- **Error Handling**: Robust error handling with descriptive messages

```rust
// Example of markdown chunking
let hf_client = HuggingFaceClient::new();
let chunks = hf_client.chunk_markdown(&client, &api_key, markdown).await?;
```

The chunking process works by:

1. Sending the markdown to Llama-3.3-70B-Instruct with instructions to chunk it into logical sections
2. Processing the model's response to extract chunks separated by "---"
3. Returning a vector of chunked text segments

### FastEmbed Integration

The service uses FastEmbed for generating embeddings from the chunks:

- Efficient vector generation
- Optimized for performance
- Converts embeddings to appropriate formats for client consumption

### Retry with Backoff

The service implements a sophisticated retry mechanism with exponential backoff:

```rust
async fn retry_with_backoff<F, Fut, T>(mut f: F) -> Result<T, anyhow::Error>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, anyhow::Error>>,
{
    // Retry logic with exponential backoff
}
```

This ensures resilience against temporary API failures or network issues.

## Development

### Project Structure

```
.
├── proto/                  # Protocol Buffer definitions
├── src/
│   ├── service/            # gRPC service implementations
│   │   ├── chunk_embed_service.rs  # Main embedding service
│   │   ├── message_service.rs      # Message handling
│   │   └── huggingface.rs          # Hugging Face API client
│   └── main.rs             # Application entry point
├── Cargo.toml              # Rust dependencies
├── build.rs                # Build script for protobuf
└── Dockerfile              # Docker configuration
```

### Testing

The project includes tests for the Hugging Face client:

```rust
#[tokio::test]
async fn test_chunk() -> Result<(), Box<dyn std::error::Error>> {
    // Test markdown chunking functionality
}
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

Built with ❤️ using Rust and gRPC.
