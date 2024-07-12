//! Starstraw integration and pages
use askama_axum::Template;
use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use starstraw::{
    model::{Profile, SkillManager, SkillName},
    Database,
};

pub fn routes(database: starstraw::Database) -> Router {
    Router::new()
        .route("/spirit/login", get(login_request))
        .route("/spirit/register", get(register_request))
        .route("/spirit/view/:username", get(spirit_view_request))
        .nest_service("/api", starstraw::api::routes(database.clone()))
        .with_state(database)
}

pub fn serialize<T: Serialize>(input: T) -> String {
    serde_json::to_string::<T>(&input).unwrap()
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

#[derive(Template)]
#[template(path = "profile.html")]
struct ProfileTemplate {
    profile: Profile,
    god_mode_allowed: bool,
}

pub async fn spirit_view_request(
    jar: CookieJar,
    Path(username): Path<String>,
    State(database): State<Database>,
) -> impl IntoResponse {
    // get user from token
    let auth_user = match jar.get("__Secure-Token") {
        Some(c) => match database
            .get_profile_by_unhashed(c.value_trimmed().to_string())
            .await
        {
            Ok(ua) => Some(SkillManager(ua.skills)),
            Err(_) => None,
        },
        None => None,
    };

    // return
    match database.get_profile_by_username(username).await {
        Ok(ua) => Html(
            ProfileTemplate {
                profile: ua,
                god_mode_allowed: match auth_user {
                    Some(sm) => sm.get_stats().title == SkillName::God,
                    None => false,
                },
            }
            .render()
            .unwrap(),
        ),
        Err(e) => Html(e.to_string()),
    }
}
