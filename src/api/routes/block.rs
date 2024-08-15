use std::error::Error;

use actix_web::web::{Data, Json};
use actix_web::{web::scope, Scope};
use serde::{Deserialize, Serialize};

use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, Responder};
use serde_json::{json, Value};

use crate::api::server::ApiServerData;
use crate::api::util::to_bytes;
use crate::rpc::types::{RpcHandlerResponse, RpcHeader, RPC};

#[derive(Serialize, Deserialize, Debug)]
pub struct GetBlockReq {
    pub height: Option<String>,
    pub hash: Option<String>,
}

#[post("/get")]
pub async fn get_block(
    req: HttpRequest,
    app: Data<ApiServerData>,
    body: Json<GetBlockReq>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let handler = app.rpc_controller.clone();

    let bytes = match to_bytes(&body) {
        Ok(b) => b,
        Err(e) => return Ok(e.respond_to(&req)),
    };

    let rpc = RPC {
        header: RpcHeader::GetBlock,
        payload: bytes,
    };

    let res = handler.handle_client_rpc(&rpc)?;

    let data = match res {
        RpcHandlerResponse::Block(block) => {
            let data = json!({ "block": block });
            json!({ "data": data })
        }
        RpcHandlerResponse::Generic(string) => json!({ "error": string }),
        _ => json!({"error":"incorrect response from RPC handler"}),
    };

    Ok(HttpResponse::Ok().json(data))
}

#[post("/get-header")]
pub async fn get_block_header(
    req: HttpRequest,
    app: Data<ApiServerData>,
    body: Json<GetBlockReq>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let handler = app.rpc_controller.clone();

    let bytes = match to_bytes(&body) {
        Ok(b) => b,
        Err(e) => return Ok(e.respond_to(&req)),
    };

    let rpc = RPC {
        header: RpcHeader::GetBlockHeader,
        payload: bytes,
    };

    let res = handler.handle_client_rpc(&rpc)?;

    let data = match res {
        RpcHandlerResponse::Header(header) => {
            let data = json!({ "header": header });
            json!({ "data": data })
        }
        RpcHandlerResponse::Generic(string) => json!({ "error": string }),
        _ => json!({"error":"incorrect response from RPC handler"}),
    };

    Ok(HttpResponse::Ok().json(data))
}

#[get("/last")]
pub async fn get_last_block(app: Data<ApiServerData>) -> Result<HttpResponse, Box<dyn Error>> {
    let handler = app.rpc_controller.clone();

    let rpc = RPC {
        header: RpcHeader::GetLastBlock,
        payload: vec![],
    };

    let res = handler.handle_client_rpc(&rpc)?;

    let data = match res {
        RpcHandlerResponse::Block(block) => {
            let data = json!({ "block": block });
            json!({ "data": data })
        }
        RpcHandlerResponse::Generic(string) => json!({ "error": string }),
        _ => json!({"error":"incorrect response from RPC handler"}),
    };

    Ok(HttpResponse::Ok().json(data))
}

pub fn register_block_routes() -> Scope {
    scope("/block")
        .service(get_block)
        .service(get_block_header)
        .service(get_last_block)
}