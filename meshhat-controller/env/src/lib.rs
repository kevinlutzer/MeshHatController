use anyhow::Context;
use dotenvy::from_filename;
use std::{
    env::{current_dir, var},
    net::SocketAddr,
    path::PathBuf,
};
use tokio::{fs::File, io::AsyncWriteExt};
use tracing_subscriber::{EnvFilter, fmt};

/// The name of the environment file that contains the service settings
const ENV_FILE_NAME: &str = "settings.ini";
/// Snap specific directory that the service will have access too
const SNAP_COMMON: &str = "SNAP_COMMON";
/// Default environment settings. These can be manipulated by editing the settings.ini
const DEFAULT_SETTINGS: &str = "GRPC_CLIENT_ADDR=https://127.0.0.1:50051\nMESHCORE_BAUD_RATE=115200\nGRPC_LISTEN_ADDR=[::]:50051\nMESHCORE_SERIAL_PORT=/dev/ttyAMA0\n";

/// Gets the settings directory for the service.
/// - For a snap this is the $SNAP_COMMON
/// - For local development or running the binary directly, this is the current working directory.
fn get_working_dir() -> anyhow::Result<PathBuf> {
    match var(SNAP_COMMON).map(PathBuf::from) {
        Ok(dir) => Ok(dir),
        Err(_) => current_dir().with_context(|| "Failed to get the current directory"),
    }
}

/// Loads the settings.ini file **or** creates it if it doesn't exist. This is used to load environment variables from the file for local development,
/// and also to create the file with default values when running in a snap.
pub async fn load_or_create_env_file() -> anyhow::Result<()> {
    let settings_dir = get_working_dir()?;
    let env_file_path = settings_dir.join(ENV_FILE_NAME);

    if !env_file_path.exists() {
        let mut file = File::create(&env_file_path)
            .await
            .with_context(|| "Failed to create settings.ini file")?;
        file.write_all(DEFAULT_SETTINGS.as_bytes())
            .await
            .with_context(|| "Failed to write default settings.ini content")?;
    }

    from_filename(&env_file_path)
        .with_context(|| "Failed to load the settings.ini file. It should exist")?;

    Ok(())
}

/// Sets up the tracing subscriber for logging.
/// It uses the RUST_LOG environment variable to determine the log level, defaulting to "info" if not set.
pub async fn setup_tracing() {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
}

/// Returns the serial port or the default port from the settings.ini file
pub fn get_serial_port() -> String {
    var("MESHCORE_SERIAL_PORT").unwrap_or_else(|_| "/dev/cu.usbmodem1101".to_string())
}

/// Returns the baud rate or the default baud rate from the settings.ini file
/// Defaults to 115200 if not set or if the value cannot be parsed as a u32
/// This is used to configure the serial connection to the MeshCore device.
pub fn get_baud_rate() -> u32 {
    var("MESHCORE_BAUD_RATE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(115_200)
}

pub fn get_client_uri_str() -> String {
    var("GRPC_CLIENT_ADDR").unwrap_or_else(|_| "https://127.0.0.1:50051".to_string())
}

pub fn get_addr_str() -> String {
    var("GRPC_LISTEN_ADDR").unwrap_or_else(|_| "[::]:50051".to_string())
}

pub fn get_addr() -> anyhow::Result<SocketAddr> {
    let addr_str = get_addr_str();
    addr_str
        .parse::<std::net::SocketAddr>()
        .with_context(|| format!("Failed to parse GRPC_LISTEN_ADDR: {}", addr_str))
}
