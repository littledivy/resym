use std::io::BufReader;

use axum::{
  extract::{Path, State},
  http::StatusCode,
  response::{IntoResponse, Response},
  routing::{get, post},
  Json, Router,
};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod error;

use error::ApiError;

struct FrameData<'s> {
  addrs: &'s [u32],
}

use pdb_addr2line::pdb;

fn look_up_addresses<'s, S: pdb::Source<'s> + Send + 's>(
  stream: S,
  addresses: &[u32],
) -> std::result::Result<(), pdb_addr2line::Error> {
  let pdb = pdb::PDB::open(stream)?;
  let context_data = pdb_addr2line::ContextPdbData::try_from_pdb(pdb)?;
  let context = context_data.make_context()?;

  for address in addresses {
    if let Some(procedure_frames) = context.find_frames(*address)? {
      eprintln!(
        "0x{:x} - {} frames:",
        address,
        procedure_frames.frames.len()
      );
      for frame in procedure_frames.frames {
        let line_str = frame.line.map(|l| format!("{}", l));
        eprintln!(
          "     {} at {}:{}",
          frame.function.as_deref().unwrap_or("<unknown>"),
          frame.file.as_deref().unwrap_or("??"),
          line_str.as_deref().unwrap_or("??"),
        )
      }
    } else {
      eprintln!("{:x} - no frames found", address);
    }
  }
  Ok(())
}

// GET /{version}/{frame_data}
async fn get_stack_trace(
  Path(version): Path<String>,
  Path(address): Path<String>,
) -> Result<Json<Vec<String>>, ApiError> {
  let mut input = address.as_bytes().iter().copied();
  let frame_data =
    vlq::decode(&mut input).map_err(|_| ApiError::InvalidFrameString)?;

  look_up_addresses(stream, &[frame_data as _]).map_err(ApiError::PdbError)?;

  todo!()
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
