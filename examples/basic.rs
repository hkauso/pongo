extern crate pongo;

use ::starstraw::{Database as SRDatabase, ServerOptions as SRServerOptions};
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

    set_var("PO_STATIC_DIR", "/static");
    set_var("PO_NESTED", "@pongo");
    set_var("PO_STARSTRAW", "/star");

    // create database
    let database = Database::new(Database::env_options(), ServerOptions::truthy()).await;
    database.init().await;

    let starstraw_database =
        SRDatabase::new(Database::env_options(), SRServerOptions::truthy()).await;
    starstraw_database.init().await;

    // create app
    let app = Router::new()
        .nest("/star", starstraw::routes(starstraw_database.clone()))
        .nest("/@pongo", pongo::dashboard::routes(database.clone()))
        .nest_service("/static", get_service(ServeDir::new("./static")))
        .fallback(pongo::api::not_found);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .unwrap();

    println!("Starting server at http://localhost:{port}!");
    axum::serve(listener, app).await.unwrap();
}
