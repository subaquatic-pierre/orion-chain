use serde::Serialize;
use std::result::Result as StdResult;

use super::error::ApiError;

pub fn to_bytes<T>(data: &T) -> StdResult<Vec<u8>, ApiError>
where
    T: ?Sized + Serialize,
{
    match bincode::serialize(data) {
        Ok(b) => Ok(b),
        Err(e) => Err(ApiError::new(&e.to_string(), 403)),
    }
}
