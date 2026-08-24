fn main() {
    let interface = std::path::PathBuf::from("src/api.udl");
    uniffi_dart::generate_scaffolding(interface).expect("generate UniFFI Dart scaffolding");
}
