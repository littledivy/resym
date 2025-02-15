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

use pdb_addr2line::pdb;

#[derive(Debug)]
pub enum Error {
  Addr2LineError(pdb_addr2line::Error),
  PdbError(pdb::Error),
  IoError(std::io::Error),
}

impl From<pdb_addr2line::Error> for Error {
  fn from(e: pdb_addr2line::Error) -> Self {
    Error::Addr2LineError(e)
  }
}

impl From<pdb::Error> for Error {
  fn from(e: pdb::Error) -> Self {
    Error::PdbError(e)
  }
}

impl From<std::io::Error> for Error {
  fn from(e: std::io::Error) -> Self {
    Error::IoError(e)
  }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::Addr2LineError(e) => write!(f, "Addr2LineError: {}", e),
      Error::PdbError(e) => write!(f, "PdbError: {}", e),
      Error::IoError(e) => write!(f, "IoError: {}", e),
    }
  }
}
