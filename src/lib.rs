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

pub use pdb_addr2line;

mod vlq;

pub fn symbolize<
  's,
  S: pdb::Source<'s> + Send + 's,
  B: Iterator<Item = u8>,
>(
  stream: S,
  input: &mut B,
  writer: &mut impl std::io::Write,
) -> std::result::Result<(), pdb_addr2line::Error> {
  let pdb = pdb::PDB::open(stream)?;
  let context_data = pdb_addr2line::ContextPdbData::try_from_pdb(pdb)?;
  let context = context_data.make_context()?;

  while let Ok(address) = vlq::vlq_decode(input) {
    if let Some(procedure_frames) = context.find_frames(address as _)? {
      let _ = writeln!(
        writer,
        "0x{:x} - {} frames:",
        address,
        procedure_frames.frames.len()
      );
      for frame in procedure_frames.frames {
        let line_str = frame.line.map(|l| format!("{}", l));
        let _ = writeln!(
          writer,
          "     {} at {}:{}",
          frame.function.as_deref().unwrap_or("<unknown>"),
          frame.file.as_deref().unwrap_or("??"),
          line_str.as_deref().unwrap_or("??"),
        );
      }
    } else {
      let _ = writeln!(writer, "{:x} - no frames found", address);
    }
  }

  Ok(())
}

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub mod win64 {
  use crate::vlq::vlq_encode;

  type WORD = u16;
  type DWORD = u32;
  type DWORDLONG = u64;
  type HMODULE = *mut u8;
  type BOOL = i32;

  #[repr(C)]
  struct M128A {
    low: u64,
    high: i64,
  }

  #[repr(C, align(16))]
  struct CONTEXT {
    P1Home: DWORDLONG,
    P2Home: DWORDLONG,
    P3Home: DWORDLONG,
    P4Home: DWORDLONG,
    P5Home: DWORDLONG,
    P6Home: DWORDLONG,

    ContextFlags: DWORD,
    MxCsr: DWORD,

    SegCs: WORD,
    SegDs: WORD,
    SegEs: WORD,
    SegFs: WORD,
    SegGs: WORD,
    SegSs: WORD,
    EFlags: DWORD,

    Dr0: DWORDLONG,
    Dr1: DWORDLONG,
    Dr2: DWORDLONG,
    Dr3: DWORDLONG,
    Dr6: DWORDLONG,
    Dr7: DWORDLONG,

    Rax: DWORDLONG,
    Rcx: DWORDLONG,
    Rdx: DWORDLONG,
    Rbx: DWORDLONG,
    Rsp: DWORDLONG,
    Rbp: DWORDLONG,
    Rsi: DWORDLONG,
    Rdi: DWORDLONG,
    R8: DWORDLONG,
    R9: DWORDLONG,
    R10: DWORDLONG,
    R11: DWORDLONG,
    R12: DWORDLONG,
    R13: DWORDLONG,
    R14: DWORDLONG,
    R15: DWORDLONG,

    Rip: DWORDLONG,

    FltSave: [u8; 512],

    VectorRegister: [M128A; 26],
    VectorControl: DWORDLONG,

    DebugControl: DWORDLONG,
    LastBranchToRip: DWORDLONG,
    LastBranchFromRip: DWORDLONG,
    LastExceptionToRip: DWORDLONG,
    LastExceptionFromRip: DWORDLONG,
  }
  extern "system" {
    fn GetModuleHandleExW(
      dwFlags: DWORD,
      name: *const u8,
      handle: *mut HMODULE,
    ) -> BOOL;
    fn RtlCaptureContext(r: *mut CONTEXT);
    fn RtlLookupFunctionEntry(
      ip: DWORDLONG,
      base: *mut DWORDLONG,
      hstable: *mut (),
    ) -> *mut ();
    fn RtlVirtualUnwind(
      ty: u32,
      base: DWORDLONG,
      ip: DWORDLONG,
      entry: *mut (),
      r: *mut CONTEXT,
      hnd_data: *mut *mut (),
      est_frame: *mut DWORDLONG,
      ctx_ptrs: *mut (),
    ) -> *mut ();
  }

  pub fn trace() -> String {
    let mut encoded = Vec::new();

    unsafe {
      let mut context = core::mem::zeroed::<CONTEXT>();
      RtlCaptureContext(&mut context);

      loop {
        let ip = context.Rip;
        let mut base = 0;
        let fn_entry =
          RtlLookupFunctionEntry(ip, &mut base, std::ptr::null_mut());
        if fn_entry.is_null() {
          break;
        }

        let addr = ip as usize;
        let mut handle = std::ptr::null_mut();
        const GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS: u32 = 0x4;
        GetModuleHandleExW(
          GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
          addr as _,
          &mut handle,
        );

        let addr = addr - handle as usize;
        vlq_encode(addr as i32, &mut encoded);

        let mut hnd_data = 0usize;
        let mut est_frame = 0;
        RtlVirtualUnwind(
          0,
          base,
          ip,
          fn_entry,
          &mut context,
          std::ptr::addr_of_mut!(hnd_data) as _,
          &mut est_frame,
          std::ptr::null_mut(),
        );
        if context.Rip == 0 {
          break;
        }
      }
    }

    // Safety: `encoded` is guaranteed to be valid UTF-8
    unsafe { String::from_utf8_unchecked(encoded) }
  }
}
