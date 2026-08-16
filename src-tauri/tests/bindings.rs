use std::fs;

const BINDINGS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../src/bridge/bindings.ts");

#[test]
#[ignore = "writes the committed TypeScript bindings"]
fn export_typescript_bindings() {
    linguaray_lib::export_typescript_bindings(BINDINGS_PATH);
}

#[test]
fn generated_bindings_are_current() {
    let dir = tempfile::tempdir().expect("create bindings tempdir");
    let generated_path = dir.path().join("bindings.ts");
    linguaray_lib::export_typescript_bindings(&generated_path);

    let generated = fs::read_to_string(generated_path).expect("read generated bindings");
    let committed = fs::read_to_string(BINDINGS_PATH).expect(
        "read committed bindings; run `pnpm bindings:generate` if the file is missing",
    );
    assert_eq!(
        committed, generated,
        "Tauri bindings are stale; run `pnpm bindings:generate`"
    );
}
