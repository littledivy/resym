use axum::{http, response::IntoResponse, Json};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum ApiError {
  #[error("Invalid trace frame string")]
  InvalidFrameString,

  #[error("PDB error: {0}")]
  PdbError(#[from] resym::pdb_addr2line::Error),
}

impl IntoResponse for ApiError {
  fn into_response(self) -> axum::response::Response {
    #[derive(Serialize)]
    struct ErrResponse {
      message: String,
    }

    let status = match self {
      Self::InvalidFrameString => http::StatusCode::BAD_REQUEST,
      Self::PdbError(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
    };

    let message = format!("{}", self);
    (status, Json(ErrResponse { message })).into_response()
  }
}
