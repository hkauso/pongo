use askama_axum::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use axum_extra::extract::cookie::CookieJar;

use crate::Database;
use dorsal::db::special::auth_db::{FullUser, UserMetadata};

pub fn routes(database: Database) -> Router {
    Router::new()
        .route("/", get(homepage_request))
        .route("/:table", get(table_view_request))
        .nest_service("/api", crate::api::routes(database.clone()))
        // ...
        .with_state(database)
}

#[derive(Template)]
#[template(path = "homepage.html")]
struct HomepageTemplate {
    me: FullUser<UserMetadata>,
    auth_state: bool,
}

#[derive(Template)]
#[template(path = "auth.html")]
struct AuthPickerTemplate {
    auth_state: bool,
}

pub async fn homepage_request(
    jar: CookieJar,
    State(database): State<Database>,
) -> impl IntoResponse {
    // get user from token
    let auth_user = match jar.get("__Secure-Token") {
        Some(c) => match database
            .auth
            .get_user_by_unhashed(c.value_trimmed().to_string())
            .await
        {
            Ok(ua) => ua,
            Err(_) => {
                return Html(AuthPickerTemplate { auth_state: false }.render().unwrap());
            }
        },
        None => {
            return Html(AuthPickerTemplate { auth_state: false }.render().unwrap());
        }
    };

    // check permissions
    if !auth_user
        .level
        .permissions
        .contains(&"StaffDashboard".to_string())
    {
        return Html(AuthPickerTemplate { auth_state: false }.render().unwrap());
    }

    // ...
    Html(
        HomepageTemplate {
            me: auth_user,
            auth_state: true,
        }
        .render()
        .unwrap(),
    )
}

#[derive(Template)]
#[template(path = "table.html")]
struct TableViewTemplate {
    me: FullUser<UserMetadata>,
    auth_state: bool,
    name: String,
}

pub async fn table_view_request(
    jar: CookieJar,
    Path(name): Path<String>,
    State(database): State<Database>,
) -> impl IntoResponse {
    // get user from token
    let auth_user = match jar.get("__Secure-Token") {
        Some(c) => match database
            .auth
            .get_user_by_unhashed(c.value_trimmed().to_string())
            .await
        {
            Ok(ua) => ua,
            Err(_) => {
                return Html(AuthPickerTemplate { auth_state: false }.render().unwrap());
            }
        },
        None => {
            return Html(AuthPickerTemplate { auth_state: false }.render().unwrap());
        }
    };

    // check permissions
    if !auth_user
        .level
        .permissions
        .contains(&"StaffDashboard".to_string())
    {
        return Html(AuthPickerTemplate { auth_state: false }.render().unwrap());
    }

    // ...
    Html(
        TableViewTemplate {
            me: auth_user,
            auth_state: true,
            name,
        }
        .render()
        .unwrap(),
    )
}
