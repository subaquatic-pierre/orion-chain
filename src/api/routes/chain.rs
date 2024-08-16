use std::error::Error;

use actix_web::web::{Data, Json};
use actix_web::{web::scope, Scope};
use log::debug;

use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, Responder};
use serde_json::{json, Value};

use crate::api::server::ApiServerData;
use crate::rpc::types::{RpcHeader, RpcResponse, RPC};

#[get("/height")]
pub async fn get_chain_height(app: Data<ApiServerData>) -> Result<HttpResponse, Box<dyn Error>> {
    let handler = app.rpc_controller.clone();

    let rpc = RPC {
        header: RpcHeader::GetLastBlock,
        payload: vec![],
    };

    let res = handler.handle_client_rpc(&rpc)?;

    let data = match res {
        RpcResponse::Block(block) => {
            let data = json!({ "height": block.header().height });
            json!({ "data": data })
        }
        RpcResponse::Generic(string) => json!({ "error": string }),
        _ => json!({"error":"incorrect response from RPC handler"}),
    };

    Ok(HttpResponse::Ok().json(data))
}

pub fn register_chain_routes() -> Scope {
    scope("/chain").service(get_chain_height)
}
