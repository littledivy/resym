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

use axum::{extract::Path, response::Html, routing::get, Router};
use resym::{symbolicate, Formatter};
use std::io::Write;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod error;

use error::ApiError;

struct HtmlFormatter<'a> {
  writer: &'a mut Vec<u8>,
}

impl<'a> HtmlFormatter<'a> {
  fn new(writer: &'a mut Vec<u8>) -> Self {
    Self { writer }
  }
}

impl Formatter for HtmlFormatter<'_> {
  fn write_frames(
    &mut self,
    _: u32,
    frame: &resym::pdb_addr2line::FunctionFrames,
  ) {
    for frame in &frame.frames {
      let source_str =
        maybe_link_source(frame.file.as_deref().unwrap_or("??"), frame.line);
      let _ = writeln!(
        self.writer,
        "     <li>{} at {}</li>",
        frame.function.as_deref().unwrap_or("<unknown>"),
        source_str,
      );
    }
  }
}

fn maybe_link_source(file: &str, line: Option<u32>) -> String {
  let file = file.replace("\\", "/");
  let line_str = line.map(|l| l.to_string()).unwrap_or("??".to_string());

  // rustc
  if file.starts_with("/rustc/") {
    let mut parts = file.splitn(4, '/');
    let _ = parts.next();
    let _ = parts.next();
    let commit_hash = parts.next().unwrap_or("??");
    let actual_path = parts.next().unwrap_or("??");

    return format!(
        "<a target='_blank' href='https://github.com/rust-lang/rust/tree/{}/{}#L{}'>{}</a>",
        commit_hash, actual_path, line_str, actual_path
      );
  }

  let mut parts = file.split('/');
  // deno
  while let Some(part) = parts.next() {
    if part == "deno" {
      let actual_path = parts.collect::<Vec<_>>().join("/");

      return format!(
        "<a target='_blank' href='https://github.com/denoland/deno/blob/main/{}#L{}'>{}</a>",
        actual_path, line_str, actual_path
      );
    }
  }

  format!("{}:{}", file, line_str,)
}

// http://localhost:1234/0.0.0/uhvCs8Z220xrB-zzxrB-3ixrBs4xxrBoh-4zBqvzB2ujB8tgBiiuByguvrB0_tBy4zBwplmzBut0L4y_uB

// GET /{version}/{frame_data}
async fn get_stack_trace(
  Path((_version, address)): Path<(String, String)>,
) -> Result<Html<String>, ApiError> {
  let mut input = address.as_bytes().iter().copied();
  let stream = std::fs::File::open("example.pdb").unwrap();

  let mut writer = Vec::new();
  symbolicate(stream, &mut input, HtmlFormatter::new(&mut writer))
    .map_err(ApiError::Resym)?;

  Ok(Html(String::from_utf8(writer).unwrap()))
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
