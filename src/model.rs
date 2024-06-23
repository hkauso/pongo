use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use dorsal::DefaultReturn;
use serde::{Deserialize, Serialize};

/// Basic serialized content storage for extra features that don't need their own table
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Document<T, M> {
    // identifiers
    pub id: String,
    pub namespace: String,
    // document;
    pub content: T,
    pub timestamp: u128,
    pub metadata: M,
}

// props
#[derive(Serialize, Deserialize, Debug)]
pub struct DocumentCreate<T, M> {
    pub namespace: String,
    pub content: T,
    pub metadata: M,
}

/// General API errors
pub enum PongoError {
    NotAllowed,
    ValueError,
    NotFound,
    Other,
}

impl PongoError {
    pub fn to_string(&self) -> String {
        use PongoError::*;
        match self {
            NotAllowed => String::from("You are not allowed to access this resource."),
            ValueError => String::from("One of the field values given is invalid."),
            NotFound => String::from("No asset with this ID could be found."),
            _ => String::from("An unspecified error has occured"),
        }
    }
}

impl IntoResponse for PongoError {
    fn into_response(self) -> Response {
        use crate::model::PongoError::*;
        match self {
            NotAllowed => (
                StatusCode::UNAUTHORIZED,
                Json(DefaultReturn::<u16> {
                    success: false,
                    message: self.to_string(),
                    payload: 401,
                }),
            )
                .into_response(),
            NotFound => (
                StatusCode::NOT_FOUND,
                Json(DefaultReturn::<u16> {
                    success: false,
                    message: self.to_string(),
                    payload: 404,
                }),
            )
                .into_response(),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DefaultReturn::<u16> {
                    success: false,
                    message: self.to_string(),
                    payload: 500,
                }),
            )
                .into_response(),
        }
    }
}
