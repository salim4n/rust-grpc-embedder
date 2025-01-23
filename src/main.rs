mod service;
use std::sync::Arc;
use tonic::{transport::Server, Request, Response, Status};
use langchain_rust::embedding::{Embedder, FastEmbed};
use crate::service::chunk_embed_service::{chunk_embed, ChunkEmbedService};

//TODO: Finir la parralelisation de l'embedding pour  les performances
//TODO: Un système Agentic => un agent qui recherche les mots clé selon un sujet donné par l'user
//TODO: Un système Agentic => pour trouver le meilleur subreddit selon les mots clés de l'agent précedents
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;
    let fast_embed = Arc::new(FastEmbed::try_new()?);

    // Utilisation du constructeur new()
    let embed_service = ChunkEmbedService::new(fast_embed);

    println!("Server listening on {}", addr);


    Server::builder()
        .add_service(chunk_embed::chunk_embed_server::ChunkEmbedServer::new(embed_service))
        .serve(addr)
        .await?;

    Ok(())
}
