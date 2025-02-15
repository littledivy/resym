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

use pdb_addr2line::FunctionFrames;

pub trait Formatter {
  fn write_frames(&mut self, addr: u32, frame: &FunctionFrames);
}

pub struct DefaultFormatter<'a, W: std::io::Write> {
  writer: &'a mut W,
}

impl<'a, W: std::io::Write> Formatter for DefaultFormatter<'a, W> {
  fn write_frames(&mut self, addr: u32, frame: &FunctionFrames) {
    let _ =
      writeln!(self.writer, "0x{:x} - {} frames:", addr, frame.frames.len());
    for frame in &frame.frames {
      let line_str = frame.line.map(|l| format!("{}", l));
      let _ = writeln!(
        self.writer,
        "     {} at {}:{}",
        frame.function.as_deref().unwrap_or("<unknown>"),
        frame.file.as_deref().unwrap_or("??"),
        line_str.as_deref().unwrap_or("??"),
      );
    }
  }
}

impl<'a, W: std::io::Write> DefaultFormatter<'a, W> {
  pub fn new(writer: &'a mut W) -> Self {
    Self { writer }
  }
}
