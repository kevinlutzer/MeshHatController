use std::sync::Arc;

use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use tracing::{error, info};

use meshcore_rs::{
    commands::{CommandHandler, Destination},
    events::{Contact, EventPayload, EventType},
};

use hex::FromHex;

use crate::meshcore_proto::{
    ContactInfo, CreateContactRequest, CreateContactResponse, DeleteContactRequest,
    DeleteContactResponse, HealthcheckRequest, HealthcheckResponse, ReceiveMessageRequest,
    ReceiveMessageResponse, ResetRequest, ResetResponse, SearchContactRequest,
    SearchContactResponse, SendMessageRequest, SendMessageResponse,
    mesh_core_service_server::MeshCoreService,
    send_message_request::Destination as ProtoDestination,
};

/// Decode a 64-character hex string into a 32-byte array.
#[allow(clippy::result_large_err)]
fn decode_pubkey(hex_str: &str) -> Result<[u8; 32], Status> {
    let bytes = Vec::<u8>::from_hex(hex_str)
        .map_err(|e| Status::invalid_argument(format!("invalid hex pubkey: {e}")))?;

    bytes
        .try_into()
        .map_err(|_| Status::invalid_argument("pubkey must be exactly 32 bytes (64 hex chars)"))
}

pub struct MeshCoreServiceImpl {
    commands: Arc<Mutex<CommandHandler>>,
}

impl MeshCoreServiceImpl {
    pub fn new(commands: &Arc<Mutex<CommandHandler>>) -> Self {
        Self {
            commands: commands.clone(),
        }
    }
}

#[tonic::async_trait]
impl MeshCoreService for MeshCoreServiceImpl {
    // ── SendMessage ───────────────────────────────────────────────────────────
    async fn send_message(
        &self,
        request: Request<SendMessageRequest>,
    ) -> Result<Response<SendMessageResponse>, Status> {
        let req = request.into_inner();
        let text = &req.text;
        let timestamp = req.timestamp;

        let cmd = self.commands.lock().await;
        let result = match req.destination {
            Some(ProtoDestination::ContactPubkeyHex(ref hex)) => cmd
                .send_msg(Destination::Hex(hex.to_string()), text, timestamp)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string()),
            Some(ProtoDestination::ChannelIndex(idx)) => cmd
                .send_channel_msg(idx as u8, text, timestamp)
                .await
                .map_err(|e| e.to_string()),
            None => {
                return Err(Status::invalid_argument(
                    "destination must be set (contact_pubkey_hex or channel_index)",
                ));
            }
        };

        drop(cmd);

        match result {
            Ok(()) => Ok(Response::new(SendMessageResponse {
                success: true,
                error: String::new(),
            })),
            Err(e) => {
                error!(error = %e, "SendMessage failed");
                Ok(Response::new(SendMessageResponse {
                    success: false,
                    error: e,
                }))
            }
        }
    }

    async fn receive_message(
        &self,
        _request: Request<ReceiveMessageRequest>,
    ) -> Result<Response<ReceiveMessageResponse>, Status> {
        let cmd = self.commands.lock().await;
        let event_opt = cmd
            .get_msg()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let Some(event) = event_opt else {
            return Ok(Response::new(ReceiveMessageResponse {
                has_message: false,
                ..Default::default()
            }));
        };

        let resp = match event.event_type {
            EventType::ContactMsgRecv => {
                if let EventPayload::ContactMessage(msg) = event.payload {
                    info!(sender = %hex::encode(msg.sender_prefix), "Received contact messasge");
                    ReceiveMessageResponse {
                        has_message: true,
                        is_channel_msg: false,
                        sender_hex: hex::encode(msg.sender_prefix),
                        channel_index: 0,
                        text: msg.text,
                    }
                } else {
                    return Err(Status::internal("ContactMsgRecv event missing payload"));
                }
            }
            EventType::ChannelMsgRecv => {
                if let EventPayload::ChannelMessage(msg) = event.payload {
                    info!(channel = msg.channel_idx, "Received channel messasge");
                    ReceiveMessageResponse {
                        has_message: true,
                        is_channel_msg: true,
                        sender_hex: String::new(),
                        channel_index: msg.channel_idx as u32,
                        text: msg.text,
                    }
                } else {
                    return Err(Status::internal("ChannelMsgRecv event missing payload"));
                }
            }
            other => {
                return Err(Status::internal(format!(
                    "unexpected event type from get_msg: {other:?}"
                )));
            }
        };

        Ok(Response::new(resp))
    }

    async fn reset(&self, _: Request<ResetRequest>) -> Result<Response<ResetResponse>, Status> {
        Ok(Response::new(ResetResponse {}))
    }

    async fn create_contact(
        &self,
        request: Request<CreateContactRequest>,
    ) -> Result<Response<CreateContactResponse>, Status> {
        let req = request.into_inner();
        info!(name = %req.name, "CreateContact");

        let public_key = decode_pubkey(&req.public_key_hex)?;

        // Convert decimal degrees → microdegrees (i32) as the library expects.
        let adv_lat = (req.latitude * 1_000_000.0) as i32;
        let adv_lon = (req.longitude * 1_000_000.0) as i32;

        let contact = Contact {
            public_key,
            contact_type: req.contact_type as u8,
            flags: req.flags as u8,
            path_len: -1, // flood
            out_path: vec![],
            adv_name: req.name,
            last_advert: 0,
            adv_lat,
            adv_lon,
            last_modification_timestamp: 0,
        };

        let cmd = self.commands.lock().await;
        match cmd.add_contact(&contact).await {
            Ok(()) => Ok(Response::new(CreateContactResponse {
                success: true,
                error: String::new(),
            })),
            Err(e) => {
                error!(error = %e, "CreateContact failed");
                Ok(Response::new(CreateContactResponse {
                    success: false,
                    error: e.to_string(),
                }))
            }
        }
    }

    async fn search_contact(
        &self,
        request: Request<SearchContactRequest>,
    ) -> Result<Response<SearchContactResponse>, Status> {
        let query = request.into_inner().query.to_lowercase();
        info!(query = %query, "SearchContact");

        let cmd = self.commands.lock().await;
        let all_contacts = cmd
            .get_contacts(0)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let contacts: Vec<ContactInfo> = all_contacts
            .into_iter()
            .filter(|c| {
                if query.is_empty() {
                    return true;
                }
                c.adv_name.to_lowercase().contains(&query) || c.public_key_hex().starts_with(&query)
            })
            .map(|c| ContactInfo {
                public_key_hex: c.public_key_hex(),
                prefix_hex: c.prefix_hex(),
                name: c.adv_name.clone(),
                contact_type: c.contact_type as u32,
                flags: c.flags as u32,
                latitude: c.latitude(),
                longitude: c.longitude(),
                last_advert: c.last_advert,
                last_modification_timestamp: c.last_modification_timestamp,
            })
            .collect();

        info!(results = contacts.len(), "SearchContact done");
        Ok(Response::new(SearchContactResponse { contacts }))
    }

    // ── DeleteContact ─────────────────────────────────────────────────────────
    async fn delete_contact(
        &self,
        request: Request<DeleteContactRequest>,
    ) -> Result<Response<DeleteContactResponse>, Status> {
        let req = request.into_inner();
        info!(pubkey = %req.public_key_hex, "DeleteContact");

        let cmd = self.commands.lock().await;
        match cmd
            .remove_contact(Destination::Hex(req.public_key_hex))
            .await
        {
            Ok(()) => Ok(Response::new(DeleteContactResponse {
                success: true,
                error: String::new(),
            })),
            Err(e) => {
                error!(error = %e, "DeleteContact failed");
                Ok(Response::new(DeleteContactResponse {
                    success: false,
                    error: e.to_string(),
                }))
            }
        }
    }

    async fn healthcheck(
        &self,
        _request: Request<HealthcheckRequest>,
    ) -> Result<Response<HealthcheckResponse>, Status> {
        info!("Healthcheck");

        let cmd = self.commands.lock().await;
        match cmd.send_appstart().await {
            Ok(info) => {
                info!(device = %info.name, "Healthcheck OK");
                Ok(Response::new(HealthcheckResponse {
                    ok: true,
                    device_name: info.name,
                    error: String::new(),
                }))
            }
            Err(e) => {
                error!(error = %e, "Healthcheck failed");
                Ok(Response::new(HealthcheckResponse {
                    ok: false,
                    device_name: String::new(),
                    error: e.to_string(),
                }))
            }
        }
    }
}
