use std::convert::Infallible;
use tokio::sync::Mutex;

use hyper::{Body, Request, Response, Server};

use routerify::prelude::*;
use routerify::{RequestInfo, Router, RouterService};

struct ServerState {
    pub tx: tokio::sync::broadcast::Sender<u32>,
}

pub async fn await_handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    println!("inside handle...");

    let state: &Mutex<ServerState> = req.data().unwrap();

    let mut r = state.lock().await.tx.subscribe();

    drop(state);

    println!("Preparing to recv...");

    let value = r.recv().await.unwrap();

    Ok(Response::builder()
        .header("content-type", "text/plain")
        .body(Body::from(String::from(format!(
            "i waited... and then was notified of the value {}.",
            value
        ))))
        .unwrap())
}

pub async fn notify_handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let state: &Mutex<ServerState> = req.data().unwrap();

    println!("Preparing to send...");

    state.lock().await.tx.send(32);

    Ok(Response::builder()
        .header("content-type", "text/plain")
        .body(Body::from(String::from("Notify all the things!")))
        .unwrap())
}

#[tokio::main]
async fn main() {
    // And a MakeService to handle each connection...
    let (tx, _) = tokio::sync::broadcast::channel(16);
    let state = Mutex::new(ServerState { tx: tx });

    let service: RouterService<Body, Infallible> = RouterService::new(
        Router::builder()
            .data(state)
            .get("/await", await_handle)
            .get("/notify", notify_handle)
            .build()
            .unwrap(),
    )
    .unwrap();

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));

    let server = Server::bind(&addr).serve(service);

    if let Err(err) = server.await {
        eprintln!("Server Error: {}", err);
    }
}
