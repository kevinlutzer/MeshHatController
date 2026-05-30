mod app_env;
mod server;
mod meshcore_proto {
    tonic::include_proto!("meshcore");
}

use tonic::transport::Server;
use tracing::{error, info, instrument::WithSubscriber};

use meshcore_rs::MeshCore;

use app_env::{
    get_baud_rate, get_grpc_listen_addr, get_serial_port, load_or_create_env_file, setup_tracing,
};
use meshcore_proto::mesh_core_service_server::MeshCoreServiceServer;
use server::MeshCoreService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_or_create_env_file().await?;

    setup_tracing().await;

    let port = get_serial_port();
    let baud_rate = get_baud_rate();
    let listen_addr = get_grpc_listen_addr();
    info!(
        "Starting the service with serial port = {}, baud rate = {}, gRPC listen address = {}",
        port, baud_rate, listen_addr
    );

    // ── Initialise MeshCore SDK over serial ──────────────────────────────────
    let meshcore = MeshCore::serial(&port, baud_rate).await.map_err(|e| {
        error!(error = %e, "Failed to open serial connection");
        e
    })?;

    let commands = meshcore.commands();

    // Verify connectivity and log the device name.
    let self_info = commands
        .clone()
        .lock()
        .await
        .send_appstart()
        .await
        .map_err(|e| {
            error!(error = %e, "send_appstart failed – is the device connected?");
            e
        })?;

    info!("Connected to MeshCore device {}", self_info.name);

    // ── gRPC server ──────────────────────────────────────────────────────────
    let addr = listen_addr.parse().map_err(|e| {
        error!(addr = %listen_addr, error = ?e, "Invalid listen address");
        Box::<dyn std::error::Error>::from(format!("invalid GRPC_LISTEN_ADDR: {e}"))
    })?;

    let service = MeshCoreService::new(commands);

    info!(%addr, "gRPC server listening");

    Server::builder()
        .add_service(MeshCoreServiceServer::new(service))
        .serve(addr)
        .with_current_subscriber()
        .await?;

    Ok(())
}
