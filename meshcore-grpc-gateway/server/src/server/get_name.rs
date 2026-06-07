use crate::meshcore_proto::GetNameResponse;
use tonic::{Response, Status};
use tracing::info;

pub async fn get_name(name: &str) -> Result<Response<GetNameResponse>, Status> {
    info!("Get Name");
    Ok(Response::new(GetNameResponse {
        name: name.to_string(),
    }))
}
