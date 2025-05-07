use std::sync::Arc;
use std::time::Duration;
use dotenvy::dotenv;
use tonic::{Request, Response, Status};
use langchain_rust::embedding::{Embedder, FastEmbed};

pub mod chunk_embed {
    tonic::include_proto!("chunk_embed");
}

use chunk_embed::chunk_embed_server::ChunkEmbed;
use crate::service::chunk_embed_service::chunk_embed::{EmbeddingFromMarkdown, EmbedMarkdownResponse, EmbedMarkdownRequest};
use chunk_embed::{ChunkEmbedRequest, ChunkEmbedResponse};
use serde::{Deserialize, Serialize};
use crate::service::huggingface::HuggingFaceClient;

#[derive(Debug, Serialize, Deserialize)]
struct LogicalChunk {
    chunk_text: String,
    chunk_summary: String,
    chunk_type: String, // "section", "code_example", "theorem", etc.
}
pub struct ChunkEmbedService {
    embedder: Arc<FastEmbed>,
}

impl ChunkEmbedService {
    pub fn new(embedder: Arc<FastEmbed>) -> Self {
        ChunkEmbedService { embedder }
    }
}

// Implémentation du service gRPC
#[tonic::async_trait]
impl ChunkEmbed for ChunkEmbedService {
    async fn chunk_embed_message(
        &self,
        request: Request<ChunkEmbedRequest>, // Requête RPC
    ) -> Result<Response<ChunkEmbedResponse>, Status> {
        let chunks = request.into_inner().chunks;

        // Génération des embeddings avec le modèle
        let embeddings_result: Result<Vec<Vec<f64>>, Status> = self
            .embedder
            .embed_documents(&chunks)
            .await // Await the future to get the result
            .map_err(|e| Status::internal(format!("Embedding error: {}", e)));

        // Gestion des erreurs
        let embeddings = embeddings_result?;

        // Aplatir et convertir les embeddings
        let embeddings: Vec<f32> = embeddings
            .into_iter()
            .flatten()
            .map(|e| e as f32)
            .collect();


        // Construction de la réponse
        let response = ChunkEmbedResponse {
            embeddings,
        };

        Ok(Response::new(response))
    }

    async fn embed_markdown(&self, request: Request<EmbedMarkdownRequest>) -> Result<Response<EmbedMarkdownResponse>, Status> {
        dotenv().ok();
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build();
        let markdown = request.into_inner().markdown;
        let api_key = std::env::var("HF_API_KEY").expect("HF_API_KEY must be set");
        let hf_client = HuggingFaceClient::new();
        let chunks = hf_client.chunk_markdown(&client.unwrap(), &api_key, &markdown).await.unwrap();
        // Génération des embeddings avec le modèle
        let embeddings_result: Result<Vec<Vec<f64>>, Status> = self
            .embedder
            .embed_documents(&chunks)
            .await // Await the future to get the result
            .map_err(|e| Status::internal(format!("Embedding error: {}", e)));

        let result: Vec<EmbeddingFromMarkdown> = match embeddings_result {
            Ok(embeddings) => embeddings
                .iter()
                .enumerate()
                .map(|(index, embedding)| {
                    EmbeddingFromMarkdown {
                        embedding: embedding.iter().map(|&e| e as f32).collect(),
                        chunk: chunks
                            .get(index)
                            .expect("Chunk not found for index")
                            .as_bytes()
                            .to_vec(),
                    }
                })
                .collect(),
            Err(e) => return Err(Status::internal(format!("Embedding error: {}", e))),
        };

        Ok(Response::new(EmbedMarkdownResponse {
            embeddings: result
        }))
    }
}

