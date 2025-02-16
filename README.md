[![Crates.io](https://img.shields.io/crates/v/resym.svg)](https://crates.io/crates/resym)

[Documentation](https://docs.rs/resym) | [Example](example/)

# `resym`

Serialize and symbolicate stack traces from remotely located PDB.

```toml
[dependencies]
resym = "0.1"
```

Here's an example:

```rust
// your application

fn set_panic_hook() {
  std::panic::set_hook(Box::new(move |info| {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
      let trace_str = resym::win64::trace();
      println!("Visit to symbolicate: http://<resym_svc>/{}", trace_str);
    }
  }));
}

fn main() {
  set_panic_hook();

  panic!("oh no!");
}
```

```rust
// your symbolification service

// GET /<trace_str>
fn handle_request(mut trace_str: Vec<u8>) -> Result<String> {
  let mut writer = Vec::new();
  let stream = std::fs::File::open("example.pdb")?;

  resym::symbolicate(stream, &mut trace_str, resym::DefaultFormatter::new(&mut writer))?;

  Ok(String::from_utf8(writer)?)
}
```

![image](https://github.com/user-attachments/assets/453a598d-c668-4423-a329-8b4b70c6f4a6)

