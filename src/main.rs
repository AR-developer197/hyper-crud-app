mod handlers;

use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use http_body_util::Full;
use hyper::{body::{Bytes, Incoming}, server::conn::http1, Method, Request, Response, StatusCode};
use hyper_util::{rt::TokioIo, service::TowerToHyperService};
use tokio::net::TcpListener;
use tower::{Service, ServiceBuilder};

pub use handlers::{
    create_user,
    get_user,
    modify_user,
    delete_user
};

pub type GenericError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, GenericError>;
pub type Req = Request<Incoming>;
pub type Db =  Arc<HashMap<String, String>>;

#[derive(Clone, Debug)]
struct Logger<S> {
    inner: S
}

impl<S> Logger<S> {
    fn new(inner: S) -> Self {
        Logger {inner}
    }
}

impl<S> Service<Req> for Logger<S> 
    where 
        S: Service<Req> + Clone
{
    type Response = S::Response;

    type Error = S::Error;

    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        println!("method: {}\r\nUri: {}", req.method(), req.uri());
        self.inner.call(req)
    }
}

async fn handle_request(req: Req, db: Db) -> Result<Response<Full<Bytes>>> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/create-user") => create_user(req, db).await,
        (&Method::GET, "/get-user") => get_user(req).await,
        (&Method::DELETE, "/delete-user") => delete_user(req).await,
        (&Method::PUT, "/modify-user") => modify_user(req).await,
        _ => {
            let res= Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from("404 NOT FOUND")))
                .unwrap();
            
            Ok(res)
        }
    }

   
}

async fn graceful_shotdown() {
    tokio::signal::ctrl_c() 
        .await
        .expect("failed to install CTRL+C signal handler");
}

#[tokio::main]
async fn main() -> Result<()>{
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let http = http1::Builder::new();

    let listener = TcpListener::bind(addr).await?;
    let conn = hyper_util::server::graceful::GracefulShutdown::new();

    let mut signal = std::pin::pin!(graceful_shotdown());

    let db = Arc::new(HashMap::new());

    loop {
        tokio::select! {
            Ok((stream, _)) = listener.accept() => {
                let io = TokioIo::new(stream);
                let db = &db;

                let sv = tower::service_fn(move |req| handle_request(req, db.clone()));
                let sv = ServiceBuilder::new().layer_fn(Logger::new).service(sv);
                let sv = TowerToHyperService::new(sv);

                let sv = http.serve_connection(io, sv);

                let fut_conn = conn.watch(sv).await;
                
                tokio::spawn(async move {
                    if let Err(err) = fut_conn {
                        println!("Server error: {}", err)
                    };
                });

            },

            _ = &mut signal => {
                println!("the server has been shut down")
            }
        }
    }
}