use bincode::config::BigEndian;
use bincode::Options;
use bytes::Buf;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{body::Incoming as IncomingBody, header, Request, Response, StatusCode};
use serde::Serialize;
use std::result::Result as StdResult;

use crate::core::error::CoreError;

use super::types::{BoxBody, Result};

// use super::types::BoxBody;

pub fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

pub async fn parse_body<T>(req: Request<IncomingBody>) -> Result<T>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let whole_body = req.collect().await?.aggregate();
    let data: T = serde_json::from_reader(whole_body.reader())?;

    Ok(data)
}

pub async fn json_response<T>(code: StatusCode, data: &T) -> Result<Response<BoxBody>>
where
    T: ?Sized + Serialize,
{
    let json = serde_json::to_string(&data)?;
    let response = Response::builder()
        .status(code)
        .header(header::CONTENT_TYPE, "application/json")
        .body(full(json))?;
    Ok(response)
}

pub fn to_bytes<T>(data: &T) -> StdResult<Vec<u8>, CoreError>
where
    T: ?Sized + Serialize,
{
    let encoder = bincode::DefaultOptions::new().with_big_endian();

    match encoder.serialize(data) {
        Ok(b) => Ok(b),
        Err(e) => Err(CoreError::Parsing(e.to_string())),
    }
}
