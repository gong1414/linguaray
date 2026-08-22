// SPM requires every C target to have at least one source file.
// All actual code lives in the generated header (`include/linguaray_runtimeFFI.h`),
// which is consumed by the Swift binding.
void __linguaray_runtime_ffi_keepalive(void) {}
