use actix_web::{web::scope, Scope};

use crate::api::routes::block::register_block_routes;

use super::routes::{chain::register_chain_routes, transaction::register_transaction_routes};

pub fn register_all_routes() -> Scope {
    scope("")
        .service(register_block_routes())
        .service(register_transaction_routes())
        .service(register_chain_routes())
}
