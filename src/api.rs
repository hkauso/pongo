//! Responds to API requests
use crate::database::Database;
use crate::model::PongoError;
use dorsal::DefaultReturn;

use axum::response::IntoResponse;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::cookie::CookieJar;
use serde::{Deserialize, Serialize};

pub fn routes(database: Database) -> Router {
    Router::new()
        // sql
        .route("/sql/:table/fetch", post(fetch_all_request))
        .route("/sql/:table/execute", post(execute_request))
        // redis
        .route("/redis/get", post(redis_get_request))
        .route("/redis/delete", post(redis_delete_request))
        .route("/redis/insert", post(redis_insert_request))
        // auth
        .route("/auth/callback", get(starstraw::api::callback_request))
        .route("/auth/logout", get(starstraw::api::logout_request))
        // ...
        .with_state(database)
}

// TODO: document api methods for admin api

#[derive(Serialize, Deserialize)]
pub struct SQLQueryProps {
    pub query: String,
}

// database operations
pub async fn fetch_all_request(
    jar: CookieJar,
    Path(table): Path<String>,
    State(database): State<Database>,
    Json(req): Json<SQLQueryProps>,
) -> impl IntoResponse {
    // get user from token
    let auth_user = match jar.get("__Secure-Token") {
        Some(c) => match database
            .auth
            .get_profile_by_unhashed(c.value_trimmed().to_string())
            .await
        {
            Ok(ua) => ua,
            Err(_) => {
                return Json(DefaultReturn {
                    success: false,
                    message: PongoError::NotAllowed.to_string(),
                    payload: None,
                });
            }
        },
        None => {
            return Json(DefaultReturn {
                success: false,
                message: PongoError::NotAllowed.to_string(),
                payload: None,
            });
        }
    };

    // check permissions
    if !starstraw::model::SkillManager(auth_user.skills)
        .has_skill(starstraw::model::SkillName::Absolute)
    {
        return Json(DefaultReturn {
            success: false,
            message: PongoError::NotAllowed.to_string(),
            payload: None,
        });
    }

    // ...
    Json(
        match database
            .sql_fetch_all(req.query.replace("$table", &table))
            .await
        {
            Ok(r) => DefaultReturn {
                success: true,
                message: String::new(),
                payload: Some(r),
            },
            Err(e) => DefaultReturn {
                success: false,
                message: e.to_string(),
                payload: None,
            },
        },
    )
}

pub async fn execute_request(
    jar: CookieJar,
    Path(table): Path<String>,
    State(database): State<Database>,
    Json(req): Json<SQLQueryProps>,
) -> impl IntoResponse {
    // get user from token
    let auth_user = match jar.get("__Secure-Token") {
        Some(c) => match database
            .auth
            .get_profile_by_unhashed(c.value_trimmed().to_string())
            .await
        {
            Ok(ua) => ua,
            Err(_) => {
                return Json(DefaultReturn {
                    success: false,
                    message: PongoError::NotAllowed.to_string(),
                    payload: None,
                });
            }
        },
        None => {
            return Json(DefaultReturn {
                success: false,
                message: PongoError::NotAllowed.to_string(),
                payload: None,
            });
        }
    };

    // check permissions
    if !starstraw::model::SkillManager(auth_user.skills)
        .has_skill(starstraw::model::SkillName::Absolute)
    {
        return Json(DefaultReturn {
            success: false,
            message: PongoError::NotAllowed.to_string(),
            payload: None,
        });
    }

    // ...
    Json(
        match database
            .sql_execute(req.query.replace("$table", &table))
            .await
        {
            Ok(r) => DefaultReturn {
                success: true,
                message: String::new(),
                payload: Some(r),
            },
            Err(e) => DefaultReturn {
                success: false,
                message: e.to_string(),
                payload: None,
            },
        },
    )
}

// redis
#[derive(Serialize, Deserialize)]
pub struct RedisQueryProps {
    pub key: String,
}

#[derive(Serialize, Deserialize)]
pub struct RedisInsertProps {
    pub key: String,
    pub value: String,
}

pub async fn redis_get_request(
    jar: CookieJar,
    State(database): State<Database>,
    Json(props): Json<RedisQueryProps>,
) -> impl IntoResponse {
    // get user from token
    let auth_user = match jar.get("__Secure-Token") {
        Some(c) => match database
            .auth
            .get_profile_by_unhashed(c.value_trimmed().to_string())
            .await
        {
            Ok(ua) => ua,
            Err(_) => {
                return Json(DefaultReturn {
                    success: false,
                    message: PongoError::NotAllowed.to_string(),
                    payload: None,
                });
            }
        },
        None => {
            return Json(DefaultReturn {
                success: false,
                message: PongoError::NotAllowed.to_string(),
                payload: None,
            });
        }
    };

    // check permissions
    if !starstraw::model::SkillManager(auth_user.skills)
        .has_skill(starstraw::model::SkillName::Absolute)
    {
        return Json(DefaultReturn {
            success: false,
            message: PongoError::NotAllowed.to_string(),
            payload: None,
        });
    }

    // ...
    Json(match database.base.cachedb.get(props.key).await {
        Some(r) => DefaultReturn {
            success: true,
            message: String::new(),
            payload: Some(r),
        },
        None => DefaultReturn {
            success: false,
            message: PongoError::NotFound.to_string(),
            payload: None,
        },
    })
}

pub async fn redis_delete_request(
    jar: CookieJar,
    State(database): State<Database>,
    Json(props): Json<RedisQueryProps>,
) -> impl IntoResponse {
    // get user from token
    let auth_user = match jar.get("__Secure-Token") {
        Some(c) => match database
            .auth
            .get_profile_by_unhashed(c.value_trimmed().to_string())
            .await
        {
            Ok(ua) => ua,
            Err(_) => {
                return Json(DefaultReturn {
                    success: false,
                    message: PongoError::NotAllowed.to_string(),
                    payload: None,
                });
            }
        },
        None => {
            return Json(DefaultReturn {
                success: false,
                message: PongoError::NotAllowed.to_string(),
                payload: None,
            });
        }
    };

    // check permissions
    if !starstraw::model::SkillManager(auth_user.skills)
        .has_skill(starstraw::model::SkillName::Absolute)
    {
        return Json(DefaultReturn {
            success: false,
            message: PongoError::NotAllowed.to_string(),
            payload: None,
        });
    }

    // ...
    Json(match database.base.cachedb.remove(props.key).await {
        false => DefaultReturn {
            success: true,
            message: String::new(),
            payload: Some(true),
        },
        true => DefaultReturn {
            success: false,
            message: PongoError::NotFound.to_string(),
            payload: None,
        },
    })
}

pub async fn redis_insert_request(
    jar: CookieJar,
    State(database): State<Database>,
    Json(props): Json<RedisInsertProps>,
) -> impl IntoResponse {
    // get user from token
    let auth_user = match jar.get("__Secure-Token") {
        Some(c) => match database
            .auth
            .get_profile_by_unhashed(c.value_trimmed().to_string())
            .await
        {
            Ok(ua) => ua,
            Err(_) => {
                return Json(DefaultReturn {
                    success: false,
                    message: PongoError::NotAllowed.to_string(),
                    payload: None,
                });
            }
        },
        None => {
            return Json(DefaultReturn {
                success: false,
                message: PongoError::NotAllowed.to_string(),
                payload: None,
            });
        }
    };

    // check permissions
    if !starstraw::model::SkillManager(auth_user.skills)
        .has_skill(starstraw::model::SkillName::Absolute)
    {
        return Json(DefaultReturn {
            success: false,
            message: PongoError::NotAllowed.to_string(),
            payload: None,
        });
    }

    // ...
    Json(
        match database.base.cachedb.set(props.key, props.value).await {
            true => DefaultReturn {
                success: true,
                message: String::new(),
                payload: Some(true),
            },
            false => DefaultReturn {
                success: false,
                message: PongoError::NotFound.to_string(),
                payload: None,
            },
        },
    )
}

// general
pub async fn not_found() -> impl IntoResponse {
    Json(DefaultReturn::<u16> {
        success: false,
        message: String::from("Path does not exist"),
        payload: 404,
    })
}
