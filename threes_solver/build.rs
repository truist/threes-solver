use sha2::{Digest, Sha256};
use std::fs;

use vergen_gix::{BuildBuilder, Emitter, GixBuilder, RustcBuilder, SysinfoBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let build = BuildBuilder::all_build()?;
    let gitcl = GixBuilder::all_git()?;
    let rustc = RustcBuilder::all_rustc()?;
    let si = SysinfoBuilder::all_sysinfo()?;

    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&gitcl)?
        .add_instructions(&rustc)?
        .add_instructions(&si)?
        .emit()?;

    let cargo_lock = fs::read("../Cargo.lock")?; //.ok();
    let mut hash = Sha256::new();
    hash.update(&cargo_lock);
    let hex = format!("{:x}", hash.finalize());
    println!("cargo:rustc-env=CARGO_LOCK_SHA256={hex}");

    Ok(())
}
