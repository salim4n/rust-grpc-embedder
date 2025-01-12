use tonic::{transport::Server,Request, Response, Status};
use tonic::transport::Error;
use message::message_server::{Message, MessageServer};
use message::{MessageRequest, MessageResponse};
pub mod message {
    tonic::include_proto!("message");
}

#[derive(Debug, Default)]
pub struct MyMessageService {}

#[tonic::async_trait]
impl Message for MyMessageService {
    async fn send_message(
        &self,
        request: Request<MessageRequest>,
    ) -> Result<Response<MessageResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = MessageResponse {
            message: format!("Hello {}!", request.into_inner().message).into(),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let addr = "[::1]:50051".parse().unwrap();
    let message_service = MyMessageService::default();
    Server::builder()
        .add_service(MessageServer::new(message_service))
        .serve(addr)
        .await.expect("failed to serve");

    Ok(())
}