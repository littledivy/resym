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

  while let Ok(address) = vlq::decode(input) {
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
  type WORD = u16;
  type DWORD = u32;
  type DWORDLONG = u64;
  type HMODULE = *mut u8;
  type BOOL = i32;

  #[repr(C)]
  pub struct M128A {
    pub Low: u64,
    pub High: i64,
  }

  #[repr(C, align(16))]
  pub struct CONTEXT {
    pub P1Home: DWORDLONG,
    pub P2Home: DWORDLONG,
    pub P3Home: DWORDLONG,
    pub P4Home: DWORDLONG,
    pub P5Home: DWORDLONG,
    pub P6Home: DWORDLONG,

    pub ContextFlags: DWORD,
    pub MxCsr: DWORD,

    pub SegCs: WORD,
    pub SegDs: WORD,
    pub SegEs: WORD,
    pub SegFs: WORD,
    pub SegGs: WORD,
    pub SegSs: WORD,
    pub EFlags: DWORD,

    pub Dr0: DWORDLONG,
    pub Dr1: DWORDLONG,
    pub Dr2: DWORDLONG,
    pub Dr3: DWORDLONG,
    pub Dr6: DWORDLONG,
    pub Dr7: DWORDLONG,

    pub Rax: DWORDLONG,
    pub Rcx: DWORDLONG,
    pub Rdx: DWORDLONG,
    pub Rbx: DWORDLONG,
    pub Rsp: DWORDLONG,
    pub Rbp: DWORDLONG,
    pub Rsi: DWORDLONG,
    pub Rdi: DWORDLONG,
    pub R8: DWORDLONG,
    pub R9: DWORDLONG,
    pub R10: DWORDLONG,
    pub R11: DWORDLONG,
    pub R12: DWORDLONG,
    pub R13: DWORDLONG,
    pub R14: DWORDLONG,
    pub R15: DWORDLONG,

    pub Rip: DWORDLONG,

    pub FltSave: [u8; 512],

    pub VectorRegister: [M128A; 26],
    pub VectorControl: DWORDLONG,

    pub DebugControl: DWORDLONG,
    pub LastBranchToRip: DWORDLONG,
    pub LastBranchFromRip: DWORDLONG,
    pub LastExceptionToRip: DWORDLONG,
    pub LastExceptionFromRip: DWORDLONG,
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
    let mut frame_addrs = vec![];
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

        frame_addrs.push(ip as usize);

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

    let mut encoded = Vec::new();
    for addr in frame_addrs {
      let mut handle = std::ptr::null_mut();
      const GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS: u32 = 0x4;
      unsafe {
        GetModuleHandleExW(
          GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
          addr as _,
          &mut handle,
        );
      }

      let addr = addr - handle as usize;
      vlq::encode(addr as i64, &mut encoded).unwrap();
    }

    let mut b64 = String::from_utf8(encoded).unwrap();
    b64
  }
}
