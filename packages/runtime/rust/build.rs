fn main() {
    let interface = camino::Utf8Path::new("src/api.udl");
    uniffi_dart::generate_scaffolding(interface).expect("generate UniFFI Dart scaffolding");
}
