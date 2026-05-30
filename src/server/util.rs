use hex::FromHex;
use tonic::Status;

/// Decode a 64-character hex string into a 32-byte array.
#[allow(clippy::result_large_err)]
pub fn decode_pubkey(hex_str: &str) -> Result<[u8; 32], Status> {
    let bytes = Vec::<u8>::from_hex(hex_str)
        .map_err(|e| Status::invalid_argument(format!("invalid hex pubkey: {e}")))?;

    bytes
        .try_into()
        .map_err(|_| Status::invalid_argument("pubkey must be exactly 32 bytes (64 hex chars)"))
}
