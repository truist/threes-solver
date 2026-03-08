use sha2::{Digest, Sha256};
use std::{env, fs, path::PathBuf};

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

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let lock_path = manifest_dir
        .ancestors()
        .map(|p| p.join("Cargo.lock"))
        .find(|p| p.exists());
    if let Some(lock_path) = lock_path {
        if let Ok(bytes) = fs::read(&lock_path) {
            let mut hash = Sha256::new();
            hash.update(&bytes);
            let hex = format!("{:x}", hash.finalize());
            println!("cargo:rustc-env=CARGO_LOCK_SHA256={hex}");
        }
    }

    Ok(())
}
