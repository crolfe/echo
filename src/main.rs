use std::collections::HashMap;
use std::env;

use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use futures::StreamExt;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
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
async fn echo_post(req: HttpRequest, mut body: web::Payload) -> Result<HttpResponse, Error> {
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
    let dc = match env::var("DC") {
        Ok(val) => val,
        Err(_) => String::from("Unknown"),
    };

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    println!("We are running in the following datacenter: {}", dc);

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(echo_get))
            .route("/", web::post().to(echo_post))
    })
    .bind("0.0.0.0:8080")?
    .bind_openssl("0.0.0.0:8443", builder)?
    .run()
    .await
}
