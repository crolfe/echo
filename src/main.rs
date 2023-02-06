use std::collections::HashMap;

use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use futures::StreamExt;
use serde::Serialize;

#[derive(Serialize)]
struct ResponseBody<'a> {
    headers: HashMap<&'a str, &'a str>,
    method: &'a str,
    query: &'a str,
    client_ip: String,
}

#[derive(Serialize)]
struct PostResponseBody<'a> {
    headers: HashMap<&'a str, &'a str>,
    method: &'a str,
    query: &'a str,
    client_ip: String,
    data: &'a str,
}

fn copy_headers<'a>(req: &'a HttpRequest) -> HashMap<&'a str, &'a str> {
    let mut headers = HashMap::new();
    for (header, value) in req.headers().iter() {
        headers.insert(header.as_str(), value.to_str().unwrap());
    }
    headers
}

fn copy_client_ip<'a>(req: &'a HttpRequest) -> String {
    match req.peer_addr() {
        Some(val) => val.ip().to_string(),
        None => String::from("unknown-ip"),
    }
}

async fn echo_get(req: HttpRequest) -> impl Responder {
    let headers = copy_headers(&req);

    HttpResponse::Ok().json(ResponseBody {
        method: req.method().as_str(),
        headers: headers,
        query: req.query_string(),
        client_ip: copy_client_ip(&req),
    })
}

async fn echo_with_body(req: HttpRequest, mut body: web::Payload) -> Result<HttpResponse, Error> {
    let mut bytes = web::BytesMut::new();
    while let Some(item) = body.next().await {
        bytes.extend_from_slice(&item?);
    }

    let headers = copy_headers(&req);

    Ok(HttpResponse::Ok().json(PostResponseBody {
        method: req.method().as_str(),
        headers: headers,
        query: req.query_string(),
        client_ip: copy_client_ip(&req),
        data: std::str::from_utf8(&bytes).unwrap(),
    }))
}

fn get_env_var(key: String, default: String) -> String {

    let env_value = match std::env::var(key) {
        Ok(val) => val,
        Err(_) => default
    };

    return env_value
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // https://patorjk.com/software/taag/#p=display&f=Delta Corps Priest 1&t=echo
    let logo = "
    ┌─┐┌─┐┬ ┬┌─┐
    ├┤ │  ├─┤│ │
    └─┘└─┘┴ ┴└─┘
    ";

    println!("{}", logo);

    let listen_port = get_env_var(String::from("ECHO_PORT"), String::from("8080"));
    let listen_addr = get_env_var(String::from("ECHO_LISTEN_ADDR"), String::from("localhost"));

    let listen = format!("{}:{}", listen_addr, listen_port);
    println!("Running on http://{}", listen);

    HttpServer::new(|| {
        App::new()
            .route("/get", web::get().to(echo_get))
            .route("/put", web::put().to(echo_with_body))
            .route("/patch", web::patch().to(echo_with_body))
            .route("/post", web::post().to(echo_with_body))
    })
    .bind(listen)?
    .run()
    .await
}
