use crate::meshcore_proto::GetInfoResponse;
use tonic::{Response, Status};
use tracing::info;

pub async fn get_info(name: &str) -> Result<Response<GetInfoResponse>, Status> {
    info!("Get Info");
    Ok(Response::new(GetInfoResponse {
        name: name.to_string(),
    }))
}
