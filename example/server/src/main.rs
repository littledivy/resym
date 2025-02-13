// Copyright 2025 Divy Srivastava <dj.srivastava23@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use axum::{extract::Path, routing::get, Router};
use resym::symbolize;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod error;

use error::ApiError;

// GET /{version}/{frame_data}
async fn get_stack_trace(
  Path((_version, address)): Path<(String, String)>,
) -> Result<String, ApiError> {
  let mut input = address.as_bytes().iter().copied();
  let stream = std::fs::File::open("example.pdb").unwrap();

  let mut writer = Vec::new();
  symbolize(stream, &mut input, &mut writer).map_err(ApiError::PdbError)?;

  Ok(String::from_utf8(writer).unwrap())
}

#[tokio::main]
async fn main() {
  dotenv::dotenv().ok();

  tracing_subscriber::registry()
    .with(
      tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(
        |_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into(),
      ),
    )
    .with(tracing_subscriber::fmt::layer())
    .init();

  let app = Router::new()
    .route("/{version}/{frame_data}", get(get_stack_trace))
    .layer(CorsLayer::permissive()); // allow all origins for now

  let listener = TcpListener::bind("0.0.0.0:1234").await.unwrap();
  println!("Listening on {}", listener.local_addr().unwrap());
  axum::serve(listener, app).await.unwrap();
}
