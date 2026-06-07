use std::sync::Arc;

use tokio::sync::Mutex;
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;
use tonic::{Request, Response, Status};

mod contact;
mod info;
mod message;
mod reset;
mod util;

use meshcore_rs::commands::CommandHandler;

use crate::meshcore_proto::{
    GetInfoRequest, GetInfoResponse, ReceiveMessageRequest, ReceiveMessageResponse, ResetRequest,
    ResetResponse, SendMessageRequest, SendMessageResponse, WatchMessagesRequest,
    mesh_core_service_server::MeshCoreService as MeshCoreServiceGrpc,
};
use crate::server::message::{receive_message, send_message};

pub struct MeshCoreService {
    commands: Arc<Mutex<CommandHandler>>,
    name: String,
}

impl MeshCoreService {
    pub fn new(commands: &Arc<Mutex<CommandHandler>>, name: &str) -> Self {
        Self {
            commands: commands.clone(),
            name: name.to_string(),
        }
    }
}

#[tonic::async_trait]
impl MeshCoreServiceGrpc for MeshCoreService {
    type WatchMessagesStream = ReceiverStream<Result<ReceiveMessageResponse, Status>>;

    async fn receive_message(
        &self,
        _request: Request<ReceiveMessageRequest>,
    ) -> Result<Response<ReceiveMessageResponse>, Status> {
        receive_message(&self.commands).await
    }

    async fn send_message(
        &self,
        request: Request<SendMessageRequest>,
    ) -> Result<Response<SendMessageResponse>, Status> {
        send_message(&self.commands, request).await
    }

    async fn watch_messages(
        &self,
        request: tonic::Request<WatchMessagesRequest>,
    ) -> Result<tonic::Response<Self::WatchMessagesStream>, tonic::Status> {
        let polling_delay_ms = request.into_inner().polling_delay_ms.unwrap_or(1000);
        let (tx, rx) = tokio::sync::mpsc::channel(128);
        let token = CancellationToken::new();

        message::watch_messages(&self.commands, tx, polling_delay_ms, token.clone()).await;
        // Token is held by the server; when response is dropped, task continues
        // But client disconnect (rx closed) will break the loop
        
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn reset(&self, _: Request<ResetRequest>) -> Result<Response<ResetResponse>, Status> {
        reset::reset().await
    }

    async fn create_contact(
        &self,
        request: Request<crate::meshcore_proto::CreateContactRequest>,
    ) -> Result<Response<crate::meshcore_proto::CreateContactResponse>, Status> {
        contact::create_contact(&self.commands, request).await
    }

    async fn search_contact(
        &self,
        request: Request<crate::meshcore_proto::SearchContactRequest>,
    ) -> Result<Response<crate::meshcore_proto::SearchContactResponse>, Status> {
        contact::search_contact(&self.commands, request).await
    }

    async fn delete_contact(
        &self,
        request: Request<crate::meshcore_proto::DeleteContactRequest>,
    ) -> Result<Response<crate::meshcore_proto::DeleteContactResponse>, Status> {
        contact::delete_contact(&self.commands, request).await
    }

    async fn get_info(
        &self,
        _request: Request<GetInfoRequest>,
    ) -> Result<Response<GetInfoResponse>, Status> {
        info::get_info(&self.name).await
    }
}
