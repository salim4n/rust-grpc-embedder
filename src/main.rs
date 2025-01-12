use std::sync::Arc;
use tonic::{transport::Server, Request, Response, Status};
use langchain_rust::embedding::{Embedder, FastEmbed};

pub mod chunk_embed {
    tonic::include_proto!("chunk_embed");
}

use chunk_embed::chunk_embed_server::{ChunkEmbed, ChunkEmbedServer};
use chunk_embed::{ChunkEmbedRequest, ChunkEmbedResponse};

// Structure du service gRPC
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;
    let fast_embed = Arc::new(FastEmbed::try_new()?);

    let embed_service = ChunkEmbedService {
        embedder: fast_embed,
    };

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(ChunkEmbedServer::new(embed_service))
        .serve(addr)
        .await?;

    Ok(())
}
