extern crate pongo;

use axum::{routing::get_service, Router};
use pongo::*;
use std::env::{set_var, var};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok(); // load .env

    let port: u16 = match var("PORT") {
        Ok(v) => v.parse::<u16>().unwrap(),
        Err(_) => 8080,
    };

    set_var("STATIC_DIR", "/static");

    // create database
    let database = Database::new(Database::env_options(), ServerOptions::truthy()).await;
    database.init().await;

    // create app
    let app = Router::new()
        .nest("/@pongo", pongo::dashboard::routes(database.clone()))
        .nest_service("/static", get_service(ServeDir::new("./static")))
        .fallback(pongo::api::not_found);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .unwrap();

    println!("Starting server at http://localhost:{port}!");
    axum::serve(listener, app).await.unwrap();
}
