use std::sync::Arc;

use anyhow::anyhow;
use meshcore_rs::{
    EventPayload, EventType,
    commands::{CommandHandler, Destination},
};

use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use tracing::{debug, error, info};

use crate::meshcore_proto::{
    ChannelMessage, ContactMessage, ReceiveMessageResponse, SendMessageRequest,
    SendMessageResponse, receive_message_response::Payload,
    send_message_request::Destination as ProtoDestination,
};

pub async fn poll_message(
    commands: &Arc<Mutex<CommandHandler>>,
) -> Result<ReceiveMessageResponse, anyhow::Error> {
    let event_opt = {
        let cmd = commands.lock().await;
        cmd.get_msg().await.map_err(|e| {
            error!(error = %e, "get_msg failed");
            anyhow!("Failed to get message: {e}")
        })?
    };

    debug!("get_msg returned event: {:?}", event_opt);

    let Some(event) = event_opt else {
        return Ok(ReceiveMessageResponse { payload: None });
    };

    match event.event_type {
        EventType::ContactMsgRecv => {
            if let EventPayload::ContactMessage(msg) = event.payload {
                info!(sender = %hex::encode(msg.sender_prefix), "Received contact message");
                Ok(ReceiveMessageResponse {
                    payload: Some(Payload::ContactMessage(ContactMessage {
                        sender_prefix_hex: hex::encode(msg.sender_prefix),
                        text: msg.text,
                    })),
                })
            } else {
                error!("ContactMsgRecv event missing payload");
                Err(anyhow!("ContactMsgRecv event missing payload"))
            }
        }
        EventType::ChannelMsgRecv => {
            if let EventPayload::ChannelMessage(msg) = event.payload {
                info!(channel = msg.channel_idx, "Received channel message");
                Ok(ReceiveMessageResponse {
                    payload: Some(Payload::ChannelMessage(ChannelMessage {
                        channel_index: msg.channel_idx as u32,
                        text: msg.text,
                    })),
                })
            } else {
                error!("ChannelMsgRecv event missing payload");
                Err(anyhow!("ChannelMsgRecv event missing payload"))
            }
        }
        other => {
            error!("Received non-message event: {:?}", other);
            Err(anyhow!("unexpected event type from get_msg: {other:?}"))
        }
    }
}

pub async fn receive_message(
    commands: &Arc<Mutex<CommandHandler>>,
) -> Result<Response<ReceiveMessageResponse>, Status> {
    poll_message(commands)
        .await
        .map(Response::new)
        .map_err(|e| {
            error!(error = %e, "ReceiveMessage failed");
            Status::internal("Failed to receive message")
        })
}

pub async fn send_message(
    command: &Arc<Mutex<CommandHandler>>,
    request: Request<SendMessageRequest>,
) -> Result<Response<SendMessageResponse>, Status> {
    let req = request.into_inner();

    let text = &req.text;
    let timestamp = req.sent_at.map(|d| d.seconds as u32);

    let result = {
        let cmd = command.lock().await;
        match req.destination {
            Some(ProtoDestination::ContactPubkeyHex(ref hex)) => cmd
                .send_msg(Destination::Hex(hex.to_string()), text, timestamp)
                .await
                .map(|_| ()),
            Some(ProtoDestination::ChannelIndex(idx)) => {
                cmd.send_channel_msg(idx as u8, text, timestamp).await
            }
            None => {
                return Err(Status::invalid_argument(
                    "destination must be set (contact_pubkey_hex or channel_index)",
                ));
            }
        }
    };

    if let Err(ref e) = result {
        error!(error = %e, "Send message failed");
        Err(Status::internal("Failed to send message "))
    } else {
        Ok(Response::new(SendMessageResponse {}))
    }
}
