use actix_cors::Cors;
use actix_web::dev::Server;
use actix_web::middleware::Logger;
use actix_web::{http::header, web, App, HttpServer, Scope};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex as StdMutex};

use std::thread;
use std::time;

use actix_web::web::Data;
use bytes::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use log::{error, info, warn};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::rpc::controller::RpcController;

use super::router::register_all_routes;

pub type GenericError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, GenericError>;
pub type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

// pub static INDEX: &[u8] = b"<a href=\"test.html\">test.html</a>";
// pub static INTERNAL_SERVER_ERROR: &[u8] = b"Internal Server Error";
// pub static NOTFOUND: &[u8] = b"Not Found";
// pub static POST_DATA: &str = r#"{"original": "data"}"#;
// pub static URL: &str = "http://127.0.0.1:1337/json_api";

pub struct ApiServerConfig {
    api_addr: String,
}

impl Default for ApiServerConfig {
    fn default() -> Self {
        ApiServerConfig {
            api_addr: "0.0.0.0:6000".to_string(),
        }
    }
}

pub struct ApiServerData {
    pub config: ApiServerConfig,
    pub rpc_controller: Arc<RpcController>,
}

pub struct ApiServer {
    // router: Arc<Mutex<HttpRouter>>,
    config: ApiServerConfig,
    data: Data<ApiServerData>,
}

impl ApiServer {
    pub fn new(config: ApiServerConfig, rpc_controller: Arc<RpcController>) -> Self {
        let data = Data::new(ApiServerData {
            config: ApiServerConfig::default(),
            rpc_controller,
        });

        Self { data, config }
    }

    pub async fn start(&self) -> Result<Server> {
        let api_addr = self.config.api_addr.to_string();
        let data = self.data.clone();
        let server = HttpServer::new(move || {
            let cors = Cors::default()
                .allow_any_origin()
                .send_wildcard()
                .allowed_methods(vec!["GET", "POST", "OPTIONS", "DELETE"])
                .allowed_headers(vec![
                    header::CONTENT_TYPE,
                    header::AUTHORIZATION,
                    header::ACCEPT,
                ]);

            App::new()
                .app_data(data.clone())
                .service(register_all_routes())
                .wrap(Logger::default())
                .wrap(cors)
        })
        .bind(api_addr.to_string())?
        .run();
        Ok(server)
    }
}
