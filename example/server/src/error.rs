use axum::{http, response::IntoResponse, Json};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum ApiError {
  #[error("Invalid trace frame string")]
  InvalidFrameString,

  #[error("Resym error: {0}")]
  Resym(#[from] resym::Error),
}

impl IntoResponse for ApiError {
  fn into_response(self) -> axum::response::Response {
    #[derive(Serialize)]
    struct ErrResponse {
      message: String,
    }

    let status = match self {
      Self::InvalidFrameString => http::StatusCode::BAD_REQUEST,
      Self::Resym(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
    };

    let message = format!("{}", self);
    (status, Json(ErrResponse { message })).into_response()
  }
}
