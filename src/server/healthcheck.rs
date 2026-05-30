use std::sync::Arc;

use tokio::sync::Mutex;
use tonic::{Response, Status};
use tracing::{error, info};

use meshcore_rs::commands::CommandHandler;

use crate::meshcore_proto::HealthcheckResponse;

pub async fn healthcheck(
    command: &Arc<Mutex<CommandHandler>>,
) -> Result<Response<HealthcheckResponse>, Status> {
    info!("Health check");

    let cmd = command.lock().await;
    match cmd.send_appstart().await {
        Ok(info) => {
            info!(device = %info.name, "Health check OK");
            Ok(Response::new(HealthcheckResponse {
                ok: true,
                device_name: info.name,
                error: String::new(),
            }))
        }
        Err(e) => {
            error!(error = %e, "Health check failed");
            Ok(Response::new(HealthcheckResponse {
                ok: false,
                device_name: String::new(),
                error: e.to_string(),
            }))
        }
    }
}
