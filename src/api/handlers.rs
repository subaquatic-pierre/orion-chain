use hyper::{body::Incoming as IncomingBody, Request, Response, StatusCode};
use serde_json::json;

use super::{
    types::{ArcRcpHandler, BoxBody, GenericReq, GetBlockReq, GetTxReq, NewTxReq, Result},
    util::{json_response, parse_body, to_bytes},
};
use crate::{
    core::hasher::Hasher,
    network::rpc::{RpcHandlerResponse, RpcHeader, RPC},
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

    let res = handler.lock().unwrap().handle_rpc(&rpc, None)?;

    let data = match res {
        RpcHandlerResponse::Header(header) => {
            let data = json!({
                "hash":header.hash().to_string(),
                "prev_hash":header.prev_hash().to_string()
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

    let res = handler.lock().unwrap().handle_rpc(&rpc, None)?;

    let data = match res {
        RpcHandlerResponse::Block(block) => {
            let data = json!({
                "hash": block.hash().to_string(),
            });
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

    let res = handler.lock().unwrap().handle_rpc(&rpc, None)?;

    let data = match res {
        RpcHandlerResponse::Transaction(tx) => {
            let data = json!({
                "hash": tx.hash.to_string(),
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

    let data = data.unwrap();

    let rpc = RPC {
        header: RpcHeader::NewTx,
        payload: to_bytes(&data)?,
    };

    let res = handler.lock().unwrap().handle_rpc(&rpc, None)?;

    let data = match res {
        RpcHandlerResponse::Transaction(tx) => {
            let data = json!({
                "hash": tx.hash.to_string(),
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

    let res = handler.lock().unwrap().handle_rpc(&rpc, None)?;

    let data = match res {
        RpcHandlerResponse::Transaction(tx) => {
            let data = json!({
                "hash": tx.hash.to_string(),
            });
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

    let res = handler.lock().unwrap().handle_rpc(&rpc, None)?;

    let data = match res {
        RpcHandlerResponse::Transaction(tx) => {
            let data = json!({
                "hash": tx.hash.to_string(),
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
