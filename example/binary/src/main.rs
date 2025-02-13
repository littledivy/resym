// Copyright 2023 Divy Srivastava <dj.srivastava23@gmail.com>
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

fn main() {
  set_panic_hook();

  panic!("Hello, world!");
}

fn set_panic_hook() {
  let orig_hook = std::panic::take_hook();
  std::panic::set_hook(Box::new(move |info| {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
      let tstr = resym::win64::trace();
      println!("link: http://localhost:1234/0.0.0/{}", tstr);
    }

    orig_hook(info);
  }));
}
