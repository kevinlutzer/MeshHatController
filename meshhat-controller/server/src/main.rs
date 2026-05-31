mod app_env;
mod server;
mod meshcore_proto {
    tonic::include_proto!("meshcore");
}

use tokio::{net::UnixListener, signal};
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::Server;

use tracing::{error, info, instrument::WithSubscriber};

use meshcore_rs::MeshCore;

use app_env::{
    get_baud_rate, get_serial_port, get_socket_path, load_or_create_env_file, setup_tracing,
};
use meshcore_proto::mesh_core_service_server::MeshCoreServiceServer;
use server::MeshCoreService;

async fn shutdown_signal() {
    signal::ctrl_c()
        .await
        .expect("Failed to install Ctrl+C handler");
    println!("\nCtrl+C received — shutting down...");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_or_create_env_file().await?;

    setup_tracing().await;

    let port = get_serial_port();
    let baud_rate = get_baud_rate();
    let socket_path = get_socket_path();
    info!(
        "Starting the service with serial port = {}, baud rate = {}, socket path = {}",
        port,
        baud_rate,
        socket_path.display()
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

    // info!("Connected to MeshCore device {}", self_info.name);

    let service = MeshCoreService::new(commands);

    // info!(%socket_path.display(), "gRPC server listening");

    let listener = UnixListener::bind(&socket_path)?;
    let incoming = UnixListenerStream::new(listener);

    Server::builder()
        .add_service(MeshCoreServiceServer::new(service))
        .serve_with_incoming_shutdown(incoming, shutdown_signal())
        .with_current_subscriber()
        .await?;

    tokio::fs::remove_file(socket_path).await?;

    info!("Service shutdown complete");

    Ok(())
}
