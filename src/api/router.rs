use std::net::SocketAddr;
use std::sync::Arc;

use bytes::{Buf, Bytes};
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{body::Incoming as IncomingBody, header, Method, Request, Response, StatusCode};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, MutexGuard};

use crate::core::blockchain::Blockchain;
use crate::network::node::ChainNode;
use crate::network::transport::{ArcMut, LocalTransport};

pub type GenericError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, GenericError>;
pub type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

pub static INDEX: &[u8] = b"<a href=\"test.html\">test.html</a>";
pub static INTERNAL_SERVER_ERROR: &[u8] = b"Internal Server Error";
pub static NOTFOUND: &[u8] = b"Not Found";
pub static POST_DATA: &str = r#"{"original": "data"}"#;
pub static URL: &str = "http://127.0.0.1:1337/json_api";

// async fn client_request_response() -> Result<Response<BoxBody>> {
//     let req = Request::builder()
//         .method(Method::POST)
//         .uri(URL)
//         .header(header::CONTENT_TYPE, "application/json")
//         .body(Full::new(Bytes::from(POST_DATA)))
//         .unwrap();

//     let host = req.uri().host().expect("uri has no host");
//     let port = req.uri().port_u16().expect("uri has no port");
//     let stream = TcpStream::connect(format!("{}:{}", host, port)).await?;
//     let io = TokioIo::new(stream);

//     let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

//     tokio::task::spawn(async move {
//         if let Err(err) = conn.await {
//             println!("Connection error: {:?}", err);
//         }
//     });

//     let web_res = sender.send_request(req).await?;

//     let res_body = web_res.into_body().boxed();

//     Ok(Response::new(res_body))
// }

pub async fn api_post_response(
    chain: &Blockchain,
    req: Request<IncomingBody>,
) -> Result<Response<BoxBody>> {
    // Aggregate the body...
    let whole_body = req.collect().await?.aggregate();
    // Decode as JSON...
    let mut data: serde_json::Value = serde_json::from_reader(whole_body.reader())?;
    // Change the JSON...

    let block = chain.get_header_cloned(0).unwrap();
    let block_str = format!("{:?}", block);

    data["test"] = serde_json::Value::from("test_value");
    data["block"] = serde_json::Value::from(block_str);
    // And respond with the new JSON.
    let json = serde_json::to_string(&data)?;
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(full(json))?;
    Ok(response)
}

pub async fn api_get_response() -> Result<Response<BoxBody>> {
    let data = vec!["foo", "bar"];
    let res = match serde_json::to_string(&data) {
        Ok(json) => Response::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .body(full(json))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(full(INTERNAL_SERVER_ERROR))
            .unwrap(),
    };
    Ok(res)
}

pub struct Router {
    node: Arc<Mutex<ChainNode<LocalTransport>>>,
}

impl Router {
    pub fn new(node: Arc<Mutex<ChainNode<LocalTransport>>>) -> Self {
        Self { node }
    }

    pub async fn route_handler(&self, req: Request<IncomingBody>) -> Result<Response<BoxBody>> {
        let chain = &self.node.lock().await.chain;
        match (req.method(), req.uri().path()) {
            (&Method::GET, "/") | (&Method::GET, "/index.html") => Ok(Response::new(full(INDEX))),
            // (&Method::GET, "/test.html") => client_request_response().await,
            (&Method::POST, "/json_api") => api_post_response(chain, req).await,
            (&Method::GET, "/json_api") => api_get_response().await,
            _ => {
                // Return 404 not found response.
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(full(NOTFOUND))
                    .unwrap())
            }
        }
    }
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}
