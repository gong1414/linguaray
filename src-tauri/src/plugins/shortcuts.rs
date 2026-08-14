//! Shortcuts capability: one `replace_all` effect, host captured at construct.

use crate::on_hotkey;
use crate::on_input_hotkey;
use crate::shortcuts::{Registrar as ShortcutRegistrar, ShortcutAction, ShortcutController};
use futures::future::BoxFuture;
use linguaray_kernel::{
    ActivationContext, CapabilityPlugin, EffectDisposer, PluginDescriptor, PluginError, PluginId,
};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// OS adapter. Inverse-rollback on failure stays here, not in the kernel.
pub struct TauriShortcutRegistrar {
    app: tauri::AppHandle,
    current: parking_lot::Mutex<Vec<(ShortcutAction, String)>>,
}

impl TauriShortcutRegistrar {
    pub fn new(app: tauri::AppHandle) -> Self {
        Self {
            app,
            current: parking_lot::Mutex::new(Vec::new()),
        }
    }

    fn register_binding(&self, action: ShortcutAction, combo: &str) -> Result<(), String> {
        let action_for_handler = action;
        self.app
            .global_shortcut()
            .on_shortcut(combo, move |app, shortcut, event| {
                if app
                    .try_state::<Arc<ShortcutController>>()
                    .is_some_and(|state| state.is_recording())
                {
                    return;
                }
                match action_for_handler {
                    ShortcutAction::Selection => on_hotkey(app, shortcut, event),
                    ShortcutAction::Input => on_input_hotkey(app, shortcut, event),
                    ShortcutAction::Clipboard if event.state == ShortcutState::Pressed => {
                        let _ = app.emit("tray-action", "translate-clipboard");
                    }
                    ShortcutAction::Clipboard | ShortcutAction::Ocr => {}
                }
            })
            .map_err(|error| error.to_string())
    }

    fn unregister_bindings(&self, bindings: &[(ShortcutAction, String)]) -> Result<(), String> {
        for (_, combo) in bindings {
            self.app
                .global_shortcut()
                .unregister(combo.as_str())
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }
}

impl ShortcutRegistrar for TauriShortcutRegistrar {
    fn replace_all(&self, shortcuts: &[(ShortcutAction, String)]) -> Result<(), String> {
        let mut current = self.current.lock();
        let previous = current.clone();
        self.unregister_bindings(&previous)?;

        let mut registered = Vec::new();
        for (action, combo) in shortcuts {
            if let Err(operation) = self.register_binding(*action, combo) {
                let _ = self.unregister_bindings(&registered);
                let mut rollback_errors = Vec::new();
                for (old_action, old_combo) in &previous {
                    if let Err(error) = self.register_binding(*old_action, old_combo) {
                        rollback_errors.push(error);
                    }
                }
                if rollback_errors.is_empty() {
                    *current = previous;
                    return Err(operation);
                }
                current.clear();
                return Err(format!(
                    "{operation}; rollback failed: {}",
                    rollback_errors.join("; ")
                ));
            }
            registered.push((*action, combo.clone()));
        }
        *current = registered;
        Ok(())
    }
}

pub struct ShortcutsPlugin {
    host: Arc<dyn ShortcutRegistrar>,
    controller: Arc<ShortcutController>,
}

impl ShortcutsPlugin {
    pub fn new(host: Arc<dyn ShortcutRegistrar>, controller: Arc<ShortcutController>) -> Self {
        Self { host, controller }
    }
}

impl CapabilityPlugin for ShortcutsPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: PluginId("shortcuts"),
            required: &[],
            optional: &[],
            provides: &[],
            manifest: None,
            restart_on_optional_change: false,
        }
    }

    fn config_fingerprint(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        if let Ok(bindings) = self.controller.registrable_now() {
            for (action, combo) in bindings {
                action.as_str().hash(&mut hasher);
                combo.hash(&mut hasher);
            }
        }
        hasher.finish()
    }

    fn activate(&self, ctx: ActivationContext) -> BoxFuture<'_, Result<(), PluginError>> {
        let host = self.host.clone();
        let controller = self.controller.clone();
        Box::pin(async move {
            let set = controller
                .registrable_now()
                .map_err(|error| PluginError::Failed(error.to_string()))?;
            ctx.install_effect("shortcuts.replace_all", move || {
                let host = host.clone();
                let controller = controller.clone();
                async move {
                    if let Err(error) = host.replace_all(&set) {
                        controller.set_registration_error(Some(error.clone()));
                        return Err(PluginError::Failed(error));
                    }
                    controller.set_registration_error(None);
                    let host = host.clone();
                    Ok(EffectDisposer::from_fn(move || {
                        let _ = host.replace_all(&[]);
                    }))
                }
            })
            .await
        })
    }
}
