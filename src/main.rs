use std::collections::HashMap;

use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use futures::StreamExt;
use serde::Serialize;

mod cli;

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

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let args = cli::parse_args();
    let listen = format!("{}:{}", args.address, args.port);

    // https://patorjk.com/software/taag/#p=display&f=Delta Corps Priest 1&t=echo
    let logo = "
    ┌─┐┌─┐┬ ┬┌─┐
    ├┤ │  ├─┤│ │
    └─┘└─┘┴ ┴└─┘
    ";

    println!("{}", logo);


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
