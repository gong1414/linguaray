//! Generic capability supervisor (spec §5).
//!
//! This crate has **no** Tauri / reqwest / rusqlite / vendor strings.
//! Production compose lives in the host (`plugins::builtin_plugins`).
//! There is no `HostEffect` type: plugins capture host in their constructor.

mod context;
mod lease;
mod supervisor;
mod types;

pub use context::{ActivationContext, KernelHandle};
pub use lease::{EffectDisposer, ServiceLease};
pub use supervisor::Supervisor;
pub use types::{
    ActivationEpoch, CancelToken, CapabilityPlugin, ComposeError, DrainConfig, FiberDiagnostic,
    FiberState, LeaseError, PluginDescriptor, PluginError, PluginId, PluginManifest, ServiceId,
    ServiceKey,
};

#[cfg(test)]
mod k0;
