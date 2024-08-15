use hyper::{body::Incoming as IncomingBody, Request, Response, StatusCode};
use log::debug;
use serde_json::json;

use super::{
    types::{ArcRcpHandler, BoxBody, GenericReq, GetBlockReq, GetTxReq, NewTxReq, Result},
    util::{json_response, parse_body, to_bytes},
};
use crate::api::types::{BlockJson, TxsJson};
use crate::core::{
    encoding::{ByteEncoding, JsonEncoding},
    transaction::Transaction,
};
use crate::rpc::{
    controller::RpcController,
    types::{RpcHandlerResponse, RpcHeader, RPC},
};
pub async fn get_block_header(
    handler: &ArcRcpHandler,
    req: Request<IncomingBody>,
) -> Result<Response<BoxBody>> {
    let data = parse_body::<GetBlockReq>(req).await;

    if data.is_err() {
        return json_response(
            StatusCode::EXPECTATION_FAILED,
            &json!({"error": "invalid input fields"}),
        )
        .await;
    }

    let data = data.unwrap();

    let rpc = RPC {
        header: RpcHeader::GetBlockHeader,
        payload: to_bytes(&data)?,
    };

    let res = handler.lock().unwrap().handle_client_rpc(&rpc)?;

    let data = match res {
        RpcHandlerResponse::Header(header) => {
            let data = json!({
                "header": header
            });
            json!({ "data": data })
        }
        RpcHandlerResponse::Generic(string) => json!({ "error": string }),
        _ => json!({"error":"incorrect response from RPC handler"}),
    };

    json_response(StatusCode::OK, &data).await
}

pub async fn get_block(
    handler: &ArcRcpHandler,
    req: Request<IncomingBody>,
) -> Result<Response<BoxBody>> {
    let data = parse_body::<GetBlockReq>(req).await;

    if data.is_err() {
        return json_response(
            StatusCode::EXPECTATION_FAILED,
            &json!({"error": "invalid input fields"}),
        )
        .await;
    }

    let data = data.unwrap();

    let rpc = RPC {
        header: RpcHeader::GetBlock,
        payload: to_bytes(&data)?,
    };

    let res = handler.lock().unwrap().handle_client_rpc(&rpc)?;

    let data = match res {
        RpcHandlerResponse::Block(block) => {
            let data = json!({ "block": block });
            json!({ "data": data })
        }
        RpcHandlerResponse::Generic(string) => json!({ "error": string }),
        _ => json!({"error":"incorrect response from RPC handler"}),
    };

    json_response(StatusCode::OK, &data).await
}

pub async fn get_tx(
    handler: &ArcRcpHandler,
    req: Request<IncomingBody>,
) -> Result<Response<BoxBody>> {
    let data = parse_body::<GetTxReq>(req).await;

    if data.is_err() {
        return json_response(
            StatusCode::EXPECTATION_FAILED,
            &json!({"error": "invalid input fields"}),
        )
        .await;
    }

    let data = data.unwrap();

    let rpc = RPC {
        header: RpcHeader::GetTx,
        payload: to_bytes(&data)?,
    };

    let res = handler.lock().unwrap().handle_client_rpc(&rpc)?;

    let data = match res {
        RpcHandlerResponse::Transaction(tx) => {
            let data = json!({
                "tx": tx,
            });
            json!({ "data": data })
        }
        RpcHandlerResponse::Generic(string) => json!({ "error": string }),
        _ => json!({"error":"incorrect response from RPC handler"}),
    };

    json_response(StatusCode::OK, &data).await
}

pub async fn new_tx(
    handler: &ArcRcpHandler,
    req: Request<IncomingBody>,
) -> Result<Response<BoxBody>> {
    let data = parse_body::<NewTxReq>(req).await;

    if data.is_err() {
        return json_response(
            StatusCode::EXPECTATION_FAILED,
            &json!({"error": "invalid input fields"}),
        )
        .await;
    }

    let byte_data = to_bytes(&data.unwrap())?;

    let new_tx = Transaction::new(&byte_data)?;

    debug!("NEW TX REQ :{new_tx:?}",);

    let rpc = RPC {
        header: RpcHeader::NewTx,
        payload: new_tx.to_bytes()?,
    };

    let res = handler.lock().unwrap().handle_client_rpc(&rpc)?;

    let data = match res {
        RpcHandlerResponse::Transaction(tx) => {
            let data = json!({
                "tx": tx,
            });
            json!({ "data": data })
        }
        RpcHandlerResponse::Generic(string) => json!({ "error": string }),
        _ => json!({"error":"incorrect response from RPC handler"}),
    };

    json_response(StatusCode::OK, &data).await
}

pub async fn get_last_block(
    handler: &ArcRcpHandler,
    req: Request<IncomingBody>,
) -> Result<Response<BoxBody>> {
    let data = parse_body::<GenericReq>(req).await;

    if data.is_err() {
        return json_response(
            StatusCode::EXPECTATION_FAILED,
            &json!({"error": "invalid input fields"}),
        )
        .await;
    }

    let data = data.unwrap();

    let rpc = RPC {
        header: RpcHeader::GetLastBlock,
        payload: to_bytes(&data)?,
    };

    let res = handler.lock().unwrap().handle_client_rpc(&rpc)?;

    let data = match res {
        RpcHandlerResponse::Block(block) => {
            let data = block.to_json()?;
            json!({ "data": data })
        }
        RpcHandlerResponse::Generic(string) => json!({ "error": string }),
        _ => json!({"error":"incorrect response from RPC handler"}),
    };

    json_response(StatusCode::OK, &data).await
}

pub async fn get_chain_height(
    handler: &ArcRcpHandler,
    req: Request<IncomingBody>,
) -> Result<Response<BoxBody>> {
    let data = parse_body::<GenericReq>(req).await;

    if data.is_err() {
        return json_response(
            StatusCode::EXPECTATION_FAILED,
            &json!({"error": "invalid input fields"}),
        )
        .await;
    }

    let data = data.unwrap();

    let rpc = RPC {
        header: RpcHeader::GetChainHeight,
        payload: to_bytes(&data)?,
    };

    let res = handler.lock().unwrap().handle_client_rpc(&rpc)?;

    let data = match res {
        RpcHandlerResponse::Transaction(tx) => {
            let data = json!({
                "hash": tx.hash().to_string(),
            });
            json!({ "data": data })
        }
        RpcHandlerResponse::Generic(string) => json!({ "error": string }),
        _ => json!({"error":"incorrect response from RPC handler"}),
    };

    json_response(StatusCode::OK, &data).await
}

pub async fn not_found() -> Result<Response<BoxBody>> {
    let data = json!({ "error": "not found" });
    // Return 404 not found response.
    json_response(StatusCode::NOT_FOUND, &data).await
}
