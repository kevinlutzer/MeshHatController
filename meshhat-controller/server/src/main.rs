mod server;
mod meshcore_proto {
    tonic::include_proto!("meshcore");
}
use meshcore_proto::mesh_core_service_server::MeshCoreServiceServer;

<<<<<<< HEAD
use tokio::signal;
use tonic::transport::Server;
use tonic_health::{ServingStatus, server::health_reporter};
=======
use tokio::{net::UnixListener, signal};
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::Server;

>>>>>>> main
use tracing::{error, info, instrument::WithSubscriber};

use meshcore_rs::MeshCore;

<<<<<<< HEAD
use env::{get_addr, get_baud_rate, get_serial_port, load_or_create_env_file, setup_tracing};

use server::MeshCoreService;

async fn shutdown_signal() {
    // we have to expect here because the overall function signature requires us to 
    // to **not** return a result
    #[allow(clippy::expect_used)]
    signal::ctrl_c()
        .await
        .expect("Failed to install Ctrl+C handler");

    info!("Ctrl+C received — shutting down...");
=======
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
>>>>>>> main
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    load_or_create_env_file().await?;

    setup_tracing().await;

    let port = get_serial_port();
    let baud_rate = get_baud_rate();
<<<<<<< HEAD

    info!(
        "Starting the service with serial port = {}, baud rate = {}",
        port, baud_rate
=======
    let socket_path = get_socket_path();
    info!(
        "Starting the service with serial port = {}, baud rate = {}, socket path = {}",
        port,
        baud_rate,
        socket_path.display()
>>>>>>> main
    );

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

<<<<<<< HEAD
    info!("Connected to MeshCore device {}", self_info.name);

    let (health_reporter, health_server) = health_reporter();
    health_reporter.set_serving::<MeshCoreServiceServer<MeshCoreService>>().await;
    health_reporter.set_service_status("MeshCoreServiceServer", ServingStatus::Serving).await;

    let service = MeshCoreService::new(commands, &self_info.name);
=======
    // info!("Connected to MeshCore device {}", self_info.name);

    let service = MeshCoreService::new(commands);

    // info!(%socket_path.display(), "gRPC server listening");

    let listener = UnixListener::bind(&socket_path)?;
    let incoming = UnixListenerStream::new(listener);
>>>>>>> main

    let addr = get_addr()?;
    Server::builder()
        .add_service(health_server)
        .add_service(MeshCoreServiceServer::new(service))
<<<<<<< HEAD
        .serve_with_shutdown(addr, shutdown_signal())
        .with_current_subscriber()
        .await?;

=======
        .serve_with_incoming_shutdown(incoming, shutdown_signal())
        .with_current_subscriber()
        .await?;

    tokio::fs::remove_file(socket_path).await?;

>>>>>>> main
    info!("Service shutdown complete");

    Ok(())
}
