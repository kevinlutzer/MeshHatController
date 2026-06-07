use std::sync::Arc;

use meshcore_rs::{
    Error, EventPayload, EventType,
    commands::{CommandHandler, Destination},
};

use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use tracing::{debug, error, info};

use crate::meshcore_proto::{
    ChannelMessage, ContactMessage, ReceiveMessageResponse, SendMessageRequest, SendMessageResponse, receive_message_response::Payload, send_message_request::Destination as ProtoDestination
};

pub async fn receive_message(
    commands: &Arc<Mutex<CommandHandler>>,
) -> Result<Response<ReceiveMessageResponse>, Status> {
    let cmd = commands.lock().await;
    let event_opt = cmd
        .get_msg()
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    debug!("get_msg returned event: {:?}", event_opt);

    let Some(event) = event_opt else {
        return Ok(Response::new(ReceiveMessageResponse {
            payload: None,
        }));
    };

    let resp = match event.event_type {
        EventType::ContactMsgRecv => {
            if let EventPayload::ContactMessage(msg) = event.payload {
                info!(sender = %hex::encode(msg.sender_prefix), "Received contact messasge");
                ReceiveMessageResponse {
                    payload: Some(Payload::ContactMessage(ContactMessage {
                        sender_prefix_hex: hex::encode(msg.sender_prefix),
                        text: msg.text,
                    }))
                }
            } else {
                return Err(Status::internal("ContactMsgRecv event missing payload"));
            }
        }
        EventType::ChannelMsgRecv => {
            if let EventPayload::ChannelMessage(msg) = event.payload {
                info!(channel = msg.channel_idx, "Received channel messasge");
                ReceiveMessageResponse {
                    payload: Some(Payload::ChannelMessage(ChannelMessage {
                        channel_index: msg.channel_idx as u32,
                        text: msg.text,
                    }))
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

pub async fn send_message(
    command: &Arc<Mutex<CommandHandler>>,
    request: Request<SendMessageRequest>,
) -> Result<Response<SendMessageResponse>, Status> {
    let req = request.into_inner();
    let text = &req.text;

    // Convert thei 
    let timestamp = if let Some(d) = req.sent_at {
        Some(d.seconds as u32)
    } else {
        None
    };

    let cmd = command.lock().await;
    let result = match req.destination {
        Some(ProtoDestination::ContactPubkeyHex(ref hex)) => {

            let result: Result<(), Error> = cmd
                .send_msg(Destination::Hex(hex.to_string()), text, timestamp)
                .await
                .map(|_| ());
            result
        }
        Some(ProtoDestination::ChannelIndex(idx)) => {
            let result: Result<(), Error> = cmd.send_channel_msg(idx as u8, text, timestamp).await;
            result
        }
        None => {
            return Err(Status::invalid_argument(
                "destination must be set (contact_pubkey_hex or channel_index)",
            ));
        }
    };

    drop(cmd);

    match result {
        Ok(()) => Ok(Response::new(SendMessageResponse {})),
        Err(e) => {
            error!(error = %e, "Send message failed");
            Ok(Response::new(SendMessageResponse {}))
        }
    }
}
