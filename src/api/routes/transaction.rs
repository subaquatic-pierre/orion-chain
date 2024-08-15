use std::error::Error;

use actix_web::web::{Data, Json};
use actix_web::{web::scope, Scope};
use log::debug;
use serde::{Deserialize, Serialize};

use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, Responder};
use serde_json::{json, Value};

use crate::api::server::ApiServerData;
use crate::api::util::to_bytes;
use crate::core::encoding::ByteEncoding;
use crate::core::transaction::Transaction;
use crate::rpc::types::{RpcHandlerResponse, RpcHeader, RPC};

#[derive(Serialize, Deserialize)]
pub struct GetTxReq {
    pub hash: String,
}

#[post("/get")]
pub async fn get_tx(
    req: HttpRequest,
    app: Data<ApiServerData>,
    body: Json<GetTxReq>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let handler = app.rpc_controller.clone();

    let bytes = match to_bytes(&body) {
        Ok(b) => b,
        Err(e) => return Ok(e.respond_to(&req)),
    };

    let rpc = RPC {
        header: RpcHeader::GetTx,
        payload: bytes,
    };

    let res = handler.handle_client_rpc(&rpc)?;

    let data = match res {
        RpcHandlerResponse::Transaction(tx) => {
            let data = json!({ "tx": tx });
            json!({ "data": data })
        }
        RpcHandlerResponse::Generic(string) => json!({ "error": string }),
        _ => json!({"error":"incorrect response from RPC handler"}),
    };

    Ok(HttpResponse::Ok().json(data))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewTxReq {
    pub value: String,
}

#[post("/new")]
pub async fn new_tx(
    req: HttpRequest,
    app: Data<ApiServerData>,
    body: Json<NewTxReq>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let handler = app.rpc_controller.clone();

    let bytes = match to_bytes(&body) {
        Ok(b) => b,
        Err(e) => return Ok(e.respond_to(&req)),
    };

    let new_tx = Transaction::new(&bytes)?;

    debug!("NEW TX REQ :{new_tx:?}",);

    let rpc = RPC {
        header: RpcHeader::NewTx,
        payload: new_tx.to_bytes()?,
    };

    let res = handler.handle_client_rpc(&rpc)?;

    let data = match res {
        RpcHandlerResponse::Transaction(tx) => {
            let data = json!({ "tx": tx });
            json!({ "data": data })
        }
        RpcHandlerResponse::Generic(string) => json!({ "error": string }),
        _ => json!({"error":"incorrect response from RPC handler"}),
    };

    Ok(HttpResponse::Ok().json(data))
}

pub fn register_transaction_routes() -> Scope {
    scope("/tx").service(get_tx).service(new_tx)
}
