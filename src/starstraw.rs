//! Starstraw integration and pages
use askama_axum::Template;
use axum::{
    extract::Query,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};

pub fn routes(database: starstraw::Database) -> Router {
    Router::new()
        .route("/spirit/login", get(login_request))
        .route("/spirit/register", get(register_request))
        .nest_service("/api", starstraw::api::routes(database.clone()))
}

#[derive(Serialize, Deserialize)]
pub struct AuthQueryProps {
    pub callback: String,
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    callback: String,
}

pub async fn login_request(Query(props): Query<AuthQueryProps>) -> impl IntoResponse {
    Html(
        LoginTemplate {
            callback: props.callback,
        }
        .render()
        .unwrap(),
    )
}

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate {
    callback: String,
}

pub async fn register_request(Query(props): Query<AuthQueryProps>) -> impl IntoResponse {
    Html(
        RegisterTemplate {
            callback: props.callback,
        }
        .render()
        .unwrap(),
    )
}
