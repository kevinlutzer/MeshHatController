use std::sync::Arc;

use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use tracing::{error, info};

use meshcore_rs::{
    commands::{CommandHandler, Destination},
    events::Contact,
};

use crate::{
    meshcore_proto::{
        ContactInfo, CreateContactRequest, CreateContactResponse, DeleteContactRequest,
        DeleteContactResponse, SearchContactRequest, SearchContactResponse,
    },
    server::util::timestamp_from_unix,
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
    cmd.add_contact(&contact)
        .await
        .map_err(|e| {
            error!(error = %e, "CreateContact failed");
            Status::internal("Failed to create contact")
        })
        .map(|_| Response::new(CreateContactResponse {}))
}

pub async fn search_contact(
    command: &Arc<Mutex<CommandHandler>>,
    request: Request<SearchContactRequest>,
) -> Result<Response<SearchContactResponse>, Status> {
    let query = request.into_inner().query.to_lowercase();
    info!(query = %query, "SearchContact");

    // Grab the contacts in a narrow scope so we don't hold the lock while processing/filtering them.
    let all_contacts = {
        let cmd = command.lock().await;
        cmd.get_contacts(0)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
    };

    // Grab **all** the contacts and then filter based
    // on the filtering criteria.
    let contacts: Vec<ContactInfo> = all_contacts
        .into_iter()
        .filter(|c| {
            if query.is_empty() {
                return true;
            }
            c.adv_name.to_lowercase().contains(&query) || c.public_key_hex().starts_with(&query)
        })
        .map(|c| {
            let last_advertised_at = if c.last_advert > 0 {
                Some(timestamp_from_unix(c.last_advert))
            } else {
                None
            };

            let last_modified_at = if c.last_modification_timestamp > 0 {
                Some(timestamp_from_unix(c.last_modification_timestamp))
            } else {
                None
            };

            ContactInfo {
                public_key_hex: c.public_key_hex(),
                prefix_hex: c.prefix_hex(),
                name: c.adv_name.clone(),

                // There are only three different types of contacts
                // so this cast is safe
                contact_type: c.contact_type as i32,
                flags: c.flags as u32,
                latitude: c.latitude(),
                longitude: c.longitude(),
                last_advertised_at,
                last_modified_at,
            }
        })
        .collect();

    info!(results = contacts.len(), "SearchContact done");
    Ok(Response::new(SearchContactResponse { contacts }))
}

pub async fn delete_contact(
    command: &Arc<Mutex<CommandHandler>>,
    request: Request<DeleteContactRequest>,
) -> Result<Response<DeleteContactResponse>, Status> {
    let req = request.into_inner();
    info!(pubkey = %req.public_key_hex, "DeleteContact");

    let cmd = command.lock().await;
    cmd.remove_contact(Destination::Hex(req.public_key_hex))
        .await
        .map(|_| Response::new(DeleteContactResponse {}))
        .map_err(|e| {
            error!(error = %e, "DeleteContact failed");
            Status::internal("Failed to delete contact")
        })
}
