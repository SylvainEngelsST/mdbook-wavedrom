// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::env;
use std::fs::File;
use std::io::Write;

use xshell::{Shell, cmd};

type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

const LICENSE_HEADER: &str = r#"/* MIT Licensed. Copyright (c) 2011-2026 Aliaksei Chapyzhenka */
/* For license information please see https://github.com/wavedrom/wavedrom/blob/trunk/LICENSE */
"#;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let version = args.next().expect("Need wavedrom.js version");

    let release_url = format!("https://github.com/wavedrom/wavedrom/releases/tag/{version}");
    let asset_url = format!("https://app.unpkg.com/wavedrom@{version}/files/wavedrom.min.js");
    let asset_path = "src/bin/assets/wavedrom.min.js";

    let commit_msg =
        format!("Upgrade to wavedrom {version}\n\nRelease: {release_url}\nAsset URL: {asset_url}");

    let sh = Shell::new()?;

    let mut fp = File::create(asset_path)?;
    let asset_content = cmd!(sh, "curl {asset_url}").read()?;

    write!(fp, "{LICENSE_HEADER}")?;
    write!(fp, "{asset_content}")?;
    drop(fp);

    cmd!(sh, "git add src/bin/assets").run()?;
    cmd!(sh, "git commit -m {commit_msg}").run()?;

    Ok(())
}
