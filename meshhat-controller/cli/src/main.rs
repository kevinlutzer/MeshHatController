use anyhow::Context;
use clap::{Parser, Subcommand};

use hyper_util::rt::TokioIo;
use tokio::net::UnixStream;
use tonic::transport::{Endpoint, Uri};
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

async fn build_channel() -> anyhow::Result<tonic::transport::Channel> {
    Endpoint::try_from("http://[::]:50051")?
        .connect_with_connector(service_fn(|_: Uri| async {
            let path =
                "/Users/kevinlutzer/Projects/MeshHatController/meshhat-controller/meshcore.sock";

            // Connect to a Uds socket
            Ok::<_, std::io::Error>(TokioIo::new(UnixStream::connect(path).await?))
        }))
        .await
        .with_context(|| "Failed to create the connector channel")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Reset {} => {
            let channel = build_channel().await?;
            let mut client = MeshCoreServiceClient::new(channel);

            let _ = client.reset(ResetRequest {}).await?;

            println!("Successfully reset the device");
        }
    }

    Ok(())
}
