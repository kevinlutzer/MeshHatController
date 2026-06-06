use std::sync::Arc;

use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

mod contact;
mod get_name;
mod message;
mod reset;
mod util;

use meshcore_rs::commands::CommandHandler;

use crate::meshcore_proto::{GetNameRequest, GetNameResponse};
use crate::meshcore_proto::{
    ReceiveMessageRequest, ReceiveMessageResponse, ResetRequest, ResetResponse, SendMessageRequest,
    SendMessageResponse, mesh_core_service_server::MeshCoreService as MeshCoreServiceGrpc,
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

    async fn get_name(
        &self,
        _request: Request<GetNameRequest>,
    ) -> Result<Response<GetNameResponse>, Status> {
        get_name::get_name(&self.name).await
    }
}
