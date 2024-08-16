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
use crate::crypto::address::random_sender_receiver;
use crate::crypto::utils::random_hash;
use crate::rpc::types::{RpcHeader, RpcResponse, RPC};

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
        RpcResponse::Transaction(tx) => {
            let tx_json = tx.data_str();
            let data = json!({ "tx": tx_json });
            json!({ "data": data })
        }
        RpcResponse::Generic(string) => json!({ "error": string }),
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

    // TODO: Tx should be completed and signed by client
    let (sender, receiver) = random_sender_receiver();
    let hash = random_hash();
    let new_tx = Transaction::new_transfer(sender, receiver, hash, &bytes)?;

    debug!("NEW TX REQ :{new_tx:?}",);

    let rpc = RPC {
        header: RpcHeader::NewTx,
        payload: new_tx.to_bytes()?,
    };

    let res = handler.handle_client_rpc(&rpc)?;

    let data = match res {
        RpcResponse::Transaction(tx) => {
            let tx_json = tx.data_str();

            let data = json!({ "tx": tx_json });
            json!({ "data": data })
        }
        RpcResponse::Generic(string) => json!({ "error": string }),
        _ => json!({"error":"incorrect response from RPC handler"}),
    };

    Ok(HttpResponse::Ok().json(data))
}

pub fn register_transaction_routes() -> Scope {
    scope("/tx").service(get_tx).service(new_tx)
}
