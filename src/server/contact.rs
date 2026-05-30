use std::sync::Arc;

use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use tracing::{error, info};

use meshcore_rs::{
    commands::{CommandHandler, Destination},
    events::Contact,
};

use crate::meshcore_proto::{
    ContactInfo, CreateContactRequest, CreateContactResponse, DeleteContactRequest,
    DeleteContactResponse, SearchContactRequest, SearchContactResponse,
};

pub async fn create_contact(
    command: &Arc<Mutex<CommandHandler>>,
    request: Request<CreateContactRequest>,
) -> Result<Response<CreateContactResponse>, Status> {
    let req = request.into_inner();
    info!(name = %req.name, "CreateContact");

    let public_key = crate::server::util::decode_pubkey(&req.public_key_hex)?;

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

    let cmd = command.lock().await;
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

pub async fn search_contact(
    command: &Arc<Mutex<CommandHandler>>,
    request: Request<SearchContactRequest>,
) -> Result<Response<SearchContactResponse>, Status> {
    let query = request.into_inner().query.to_lowercase();
    info!(query = %query, "SearchContact");

    let cmd = command.lock().await;
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
pub async fn delete_contact(
    command: &Arc<Mutex<CommandHandler>>,
    request: Request<DeleteContactRequest>,
) -> Result<Response<DeleteContactResponse>, Status> {
    let req = request.into_inner();
    info!(pubkey = %req.public_key_hex, "DeleteContact");

    let cmd = command.lock().await;
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
