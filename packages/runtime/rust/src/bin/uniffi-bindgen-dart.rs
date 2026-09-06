use anyhow::Result;
use camino::Utf8PathBuf;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let usage = "usage: uniffi-bindgen-dart <cdylib> <out_dir>";
    let cdylib = Utf8PathBuf::from(args.next().expect(usage));
    let out_dir = Utf8PathBuf::from(args.next().expect(usage));
    let manifest_dir = Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let udl = manifest_dir.join("src/api.udl");
    let config = manifest_dir.join("uniffi.toml");
    let config_override = config.exists().then(|| config.as_path());

    uniffi_dart::gen::generate_dart_bindings(&udl, config_override, Some(&out_dir), &cdylib, true)?;
    Ok(())
}
