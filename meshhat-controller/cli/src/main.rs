use std::path::PathBuf;

use clap::{Parser, Subcommand};

use tokio::net::UnixStream;
use tonic::transport::{Channel, Endpoint};
use tower::service_fn;

use crate::meshcore_proto::{ResetRequest, mesh_core_service_client::MeshCoreServiceClient};

mod meshcore_proto {
    tonic::include_proto!("meshcore");
}

/// My awesome CLI tool
#[derive(Parser)]
#[command(name = "meshat-controller", version, about, long_about = None)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Resets the device
    Reset {},
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let endpoint = Endpoint::try_from("http://[::]:50051")?;

    let channel = endpoint
        .connect_with_connector(service_fn(move |_| {
            async move {
                let stream = UnixStream::connect("/Users/kevinlutzer/Projects/MeshHatController/meshhat-controller/meshcore.sock").await?;
                Ok::<_, std::io::Error>(hyper_util::rt::TokioIo::new(stream))
            }
        }))
        .await?;

    let mut client = MeshCoreServiceClient::new(channel);

    let response = client
        .reset(ResetRequest {})
        .await?;

    println!("Response: {:?}", response.into_inner());

    Ok(())
}