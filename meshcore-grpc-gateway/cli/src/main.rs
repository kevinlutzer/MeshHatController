use crate::meshcore_proto::{
    CreateContactRequest, DeleteContactRequest, GetNameRequest, ReceiveMessageRequest,
    ResetRequest, SearchContactRequest, SendMessageRequest,
    mesh_core_service_client::MeshCoreServiceClient, send_message_request::Destination,
};
use clap::{Parser, Subcommand};
use env::get_client_uri_str;

mod meshcore_proto {
    tonic::include_proto!("meshcore");
}

#[derive(Parser)]
#[command(name = "meshat-controller", version, about, long_about = None)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Resets the device
    Reset {},

    /// Prints the name of the device
    GetName {},

    /// Creates a contact
    CreateContact {
        public_key_hex: String,
        name: String,
        contact_type: u32,
        flags: u32,
        latitude: f64,
        longitude: f64,
    },

    /// Deletes a contact with a specific hash
    DeleteContact { public_key_hex: String },

    /// Search for a specific contact. This will match based on name or public key hex.
    SearchContact {
        /// Criteria to search on
        query: String,

        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },

    /// Send a message to a contact or channel
    SendMessage {
        /// Message text to send
        text: String,

        /// Recipient's full 64-char hex public key (mutually exclusive with --channel-index)
        #[arg(long, conflicts_with = "channel_index")]
        contact_pubkey_hex: Option<String>,

        /// Channel index to send to (mutually exclusive with --contact-pubkey-hex)
        #[arg(long, conflicts_with = "contact_pubkey_hex")]
        channel_index: Option<u32>,

        /// Optional unix timestamp in seconds (defaults to server time if omitted)
        #[arg(long)]
        timestamp: Option<u32>,
    },

    /// Poll the device for the next queued incoming message
    ReceiveMessage {
        /// Output result as JSON
        #[arg(long)]
        json: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let addr_str = get_client_uri_str();
    let mut client: MeshCoreServiceClient<tonic::transport::Channel> =
        MeshCoreServiceClient::connect(addr_str).await?;

    match cli.command {
        Commands::Reset {} => {
            let _ = client.reset(ResetRequest {}).await?;
            println!("Successfully reset the device");
        }

        Commands::GetName {} => {
            let response = client.get_name(GetNameRequest {}).await?;
            println!("Device name: {}", response.into_inner().name);
        }

        Commands::SearchContact { query, json } => {
            let contacts = client
                .search_contact(SearchContactRequest { query })
                .await?
                .into_inner()
                .contacts;

            if json {
                // Build a serde_json array from the contact fields directly.
                let arr: serde_json::Value = contacts
                    .iter()
                    .map(|c| {
                        serde_json::json!({
                            "public_key_hex": c.public_key_hex,
                            "name":           c.name,
                            "contact_type":   c.contact_type,
                            "flags":          c.flags,
                            "latitude":       c.latitude,
                            "longitude":      c.longitude,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&arr)?);
            } else {
                for c in &contacts {
                    println!(
                        "{} | {} | type={} flags={} ({}, {})",
                        c.public_key_hex, c.name, c.contact_type, c.flags, c.latitude, c.longitude
                    );
                }
            }
        }

        Commands::CreateContact {
            public_key_hex,
            name,
            contact_type,
            flags,
            latitude,
            longitude,
        } => {
            client
                .create_contact(CreateContactRequest {
                    public_key_hex,
                    name,
                    contact_type,
                    flags,
                    latitude,
                    longitude,
                })
                .await?;
            println!("Successfully created contact");
        }

        Commands::DeleteContact { public_key_hex } => {
            client
                .delete_contact(DeleteContactRequest { public_key_hex })
                .await?;
            println!("Successfully deleted contact");
        }

        Commands::SendMessage {
            text,
            contact_pubkey_hex,
            channel_index,
            timestamp,
        } => {
            let destination = match (contact_pubkey_hex, channel_index) {
                (Some(pubkey), None) => Destination::ContactPubkeyHex(pubkey),
                (None, Some(idx)) => Destination::ChannelIndex(idx),
                _ => anyhow::bail!(
                    "Exactly one of --contact-pubkey-hex or --channel-index must be provided"
                ),
            };

            client
                .send_message(SendMessageRequest {
                    destination: Some(destination),
                    text,
                    timestamp,
                })
                .await?;
            println!("Message sent successfully");
        }

        Commands::ReceiveMessage { json } => {
            let msg = client
                .receive_message(ReceiveMessageRequest {})
                .await?
                .into_inner();

            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "has_message":    msg.has_message,
                        "is_channel_msg": msg.is_channel_msg,
                        "sender_hex":     msg.sender_hex,
                        "channel_index":  msg.channel_index,
                        "text":           msg.text,
                    }))?
                );
            } else if !msg.has_message {
                println!("No messages queued");
            } else if msg.is_channel_msg {
                println!("[Channel {}] {}", msg.channel_index, msg.text);
            } else {
                println!("[Contact {}] {}", msg.sender_hex, msg.text);
            }
        }
    }

    Ok(())
}
