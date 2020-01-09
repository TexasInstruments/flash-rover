// Copyright (c) 2019 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

fn main() {
    if cfg!(target_os = "windows") {
        //println!(r"cargo:rustc-link-search=C:\Program Files\Java\jdk1.8.0_202\lib");
    }
}
