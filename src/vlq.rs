pub(crate) fn vlq_decode(
  input: &mut impl Iterator<Item = u8>,
) -> std::result::Result<u64, std::io::Error> {
  let mut result = 0;
  let mut shift = 0;
  loop {
    let byte = input.next().ok_or(std::io::ErrorKind::UnexpectedEof)?;
    let value = match byte {
      b'A'..=b'Z' => byte - b'A',
      b'a'..=b'z' => byte - b'a' + 26,
      b'0'..=b'9' => byte - b'0' + 52,
      b'-' => 62,
      b'_' => 63,
      _ => return Err(std::io::ErrorKind::InvalidData.into()),
    };
    result |= (value as u64 & 31) << shift;
    if value & 32 == 0 {
      break;
    }
    shift += 5;
  }
  Ok(result)
}

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub(crate) fn vlq_encode(value: i32, writer: &mut Vec<u8>) {
  const VLQ_MAX_IN_BYTES: usize = 7;

  const BASE64_URL: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

  let mut vlq: u32 = if value >= 0 {
    (value as u32) << 1
  } else {
    ((-value as u32) << 1) | 1
  };

  for i in 0..VLQ_MAX_IN_BYTES {
    let mut digit = vlq & 31;
    vlq >>= 5;

    if vlq != 0 {
      digit |= 32;
    }

    writer.push(BASE64_URL[digit as usize]);

    if vlq == 0 {
      return;
    }
  }
}
