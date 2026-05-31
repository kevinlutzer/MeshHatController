use anyhow::Context;
use clap::{Parser, Subcommand};

use hyper_util::rt::TokioIo;
use tokio::net::UnixStream;
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;

use crate::meshcore_proto::{
    HealthcheckRequest, ResetRequest, mesh_core_service_client::MeshCoreServiceClient,
};

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
    /// Health check the device
    Healthcheck {},
}

async fn build_channel() -> anyhow::Result<tonic::transport::Channel> {
    Endpoint::try_from("http://[::]:50051")?
        .connect_with_connector(service_fn(|_: Uri| async {
            let path =
                "/Users/kevinlutzer/Projects/MeshHatController/meshhat-controller/server/meshcore.sock";

            // Connect to a Uds socket
            Ok::<_, std::io::Error>(TokioIo::new(UnixStream::connect(path).await?))
        }))
        .await
        .with_context(|| "Failed to create the connector channel")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let channel = build_channel().await?;
    let mut client = MeshCoreServiceClient::new(channel);

    match cli.command {
        Commands::Reset {} => {
            let _ = client.reset(ResetRequest {}).await?;

            println!("Successfully reset the device");
        }
        Commands::Healthcheck {} => {
            let response = client.healthcheck(HealthcheckRequest {}).await?;

            println!(
                "Health check passed with device {}",
                response.into_inner().device_name
            );
        }
    }

    Ok(())
}
