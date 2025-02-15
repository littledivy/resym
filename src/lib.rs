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

use std::io::ErrorKind;

use pdb_addr2line::pdb;

pub use error::Error;
pub use format::DefaultFormatter;
pub use format::Formatter;
pub use pdb_addr2line;

mod error;
mod format;
mod vlq;
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub mod win64;

pub fn symbolicate<
  's,
  S: pdb::Source<'s> + Send + 's,
  B: Iterator<Item = u8>,
  F: Formatter,
>(
  stream: S,
  input: &mut B,
  mut fmt: F,
) -> std::result::Result<(), Error> {
  let pdb = pdb::PDB::open(stream)?;
  let context_data = pdb_addr2line::ContextPdbData::try_from_pdb(pdb)?;
  let context = context_data.make_context()?;

  loop {
    match vlq::vlq_decode(input) {
      Ok(address) => {
        if let Some(procedure_frames) = context.find_frames(address as _)? {
          fmt.write_frames(address as _, &procedure_frames);
        }
      }
      Err(e) => {
        if e.kind() == ErrorKind::UnexpectedEof {
          break;
        }
        return Err(e.into());
      }
    }
  }

  Ok(())
}
