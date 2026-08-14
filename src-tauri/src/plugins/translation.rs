//! Translation capability. Commands lease this service; they do not read
//! `Session.client` / `Session.keystore`.

use crate::concurrency::GenerationToken;
use crate::db::Database;
use crate::engines::TraditionalEngine;
use crate::error::{ConfigKind, Error};
use crate::providers::ProviderPreset;
use crate::service::{
    self, run_session_with_fallback, PrefetchedSecrets, TranslateInput, TranslateSessionResult,
    Translation,
};
use crate::wire::AppOptions;
use futures::future::BoxFuture;
use linguaray_kernel::{
    ActivationContext, CapabilityPlugin, KernelHandle, PluginDescriptor, PluginError, PluginId,
    ServiceId, ServiceKey,
};
use std::sync::Arc;

pub static TRANSLATION: ServiceKey<TranslationHub> = ServiceKey::new("linguaray.translation");
static PROVIDES: &[ServiceId] = &[ServiceId("linguaray.translation")];
static REQUIRED: &[ServiceId] = &[
    ServiceId("linguaray.http"),
    ServiceId("linguaray.drivers"),
    ServiceId("linguaray.providers"),
];
static OPTIONAL: &[ServiceId] = &[ServiceId("linguaray.secrets")];

pub struct PersistSpec<'a> {
    pub trigger_source: String,
    pub gen: Option<(&'a GenerationToken, u64)>,
}

#[derive(Clone)]
pub struct TranslationHub {
    kernel: KernelHandle,
}

impl TranslationHub {
    fn map_lease(err: linguaray_kernel::LeaseError) -> String {
        match err {
            linguaray_kernel::LeaseError::Unloaded => {
                "HTTP client unavailable: startup build failed (recovery required)".into()
            }
            other => other.to_string(),
        }
    }

    async fn hold_http(
        &self,
    ) -> Result<
        (
            linguaray_kernel::ServiceLease<reqwest::Client>,
            linguaray_kernel::ServiceLease<crate::plugins::drivers::DriverRegistry>,
            reqwest::Client,
        ),
        String,
    > {
        let http = self
            .kernel
            .lease(crate::plugins::http::HTTP)
            .map_err(Self::map_lease)?;
        let drivers = self
            .kernel
            .lease(crate::plugins::drivers::DRIVERS)
            .map_err(Self::map_lease)?;
        let client = http
            .call(|c| {
                let c = c.clone();
                async move { c }
            })
            .await
            .map_err(Self::map_lease)?;
        Ok((http, drivers, client))
    }

    async fn prefetch_key(&self, secret_ref: &str) -> Result<String, String> {
        let secrets = self
            .kernel
            .optional(crate::plugins::secrets::SECRETS)
            .ok_or_else(|| {
                Error::Config(ConfigKind::MissingKey {
                    provider: secret_ref.to_string(),
                })
                .to_string()
            })?;
        let key = secrets
            .call(|ks| {
                let r = ks.get_key(secret_ref);
                async move { r }
            })
            .await
            .map_err(Self::map_lease)?
            .map_err(|e| e.to_string())?;
        key.ok_or_else(|| {
            Error::Config(ConfigKind::MissingKey {
                provider: secret_ref.to_string(),
            })
            .to_string()
        })
    }

    pub async fn translate_preset(
        &self,
        preset: ProviderPreset,
        text: String,
        from: String,
        to: String,
        fallback: Option<Box<dyn TraditionalEngine>>,
    ) -> Result<Translation, String> {
        let (http, drivers, client) = self.hold_http().await?;
        let epoch_h = http.epoch();
        let epoch_d = drivers.epoch();
        let mut secrets = PrefetchedSecrets::default();
        if preset.needs_key {
            secrets.insert(preset.id.clone(), self.prefetch_key(&preset.id).await?);
        }
        let out = http
            .call(|_| async move {
                let store = if secrets.is_empty() {
                    None
                } else {
                    Some(&secrets as &dyn service::SecretStore)
                };
                let input = TranslateInput {
                    text: &text,
                    from: &from,
                    to: &to,
                    options: AppOptions::default(),
                };
                service::translate_with_fallback(&client, store, &preset, input, fallback)
                    .await
                    .map_err(|e| e.to_string())
            })
            .await
            .map_err(Self::map_lease)??;
        if !http.is_live()
            || http.epoch() != epoch_h
            || !drivers.is_live()
            || drivers.epoch() != epoch_d
        {
            return Err("translation cancelled".into());
        }
        Ok(out)
    }

    pub async fn run_session(
        &self,
        db: &Arc<Database>,
        text: &str,
        from: &str,
        to: &str,
        fallback: Option<Arc<dyn TraditionalEngine>>,
        persist: Option<PersistSpec<'_>>,
    ) -> Result<TranslateSessionResult, String> {
        let (http, drivers, client) = self.hold_http().await?;
        let epoch_h = http.epoch();
        let epoch_d = drivers.epoch();
        let started = std::time::Instant::now();
        let mut prefetched = PrefetchedSecrets::default();
        if let Some(secrets) = self.kernel.optional(crate::plugins::secrets::SECRETS) {
            let refs = db
                .with_conn(|conn| crate::db::providers::list(conn))
                .map_err(|e| e.to_string())?
                .into_iter()
                .filter(|p| p.needs_key)
                .map(|p| p.secret_ref)
                .collect::<Vec<_>>();
            for secret_ref in refs {
                match secrets
                    .call(|ks| {
                        let r = ks.get_key(&secret_ref);
                        async move { r }
                    })
                    .await
                {
                    Ok(Ok(Some(key))) => prefetched.insert(secret_ref, key),
                    Ok(Ok(None)) | Ok(Err(_)) | Err(_) => {}
                }
            }
        }
        let db_job = db.clone();
        let text_owned = text.to_string();
        let from_owned = from.to_string();
        let to_owned = to.to_string();
        let result = http
            .call(|_| async move {
                let store = if prefetched.is_empty() {
                    None
                } else {
                    Some(&prefetched as &dyn service::SecretStore)
                };
                run_session_with_fallback(
                    &db_job,
                    &client,
                    store,
                    &text_owned,
                    &from_owned,
                    &to_owned,
                    fallback,
                )
                .await
            })
            .await
            .map_err(Self::map_lease)??;
        if !http.is_live()
            || http.epoch() != epoch_h
            || !drivers.is_live()
            || drivers.epoch() != epoch_d
        {
            return Err("translation cancelled".into());
        }
        if let Some(spec) = persist {
            if let Some((gen, token)) = spec.gen {
                if !gen.is_latest(token) {
                    return Ok(result);
                }
            }
            if let Some(secrets) = self.kernel.optional(crate::plugins::secrets::SECRETS) {
                let elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
                let detected = (!from.is_empty() && from != "auto").then_some(from);
                match secrets
                    .call(|ks| {
                        let persisted = crate::history::persist_translation_session(
                            db,
                            ks,
                            &spec.trigger_source,
                            text,
                            detected,
                            to,
                            &result.outcomes,
                            elapsed_ms,
                        );
                        async move { persisted }
                    })
                    .await
                {
                    Ok(Err(error)) => {
                        log::warn!("encrypted history persistence failed: {error}");
                    }
                    Err(error) => {
                        log::warn!("encrypted history persistence failed: {error}");
                    }
                    Ok(Ok(_)) => {}
                }
            }
        }
        Ok(result)
    }
}

pub struct TranslationPlugin;

impl CapabilityPlugin for TranslationPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: PluginId("translation"),
            required: REQUIRED,
            optional: OPTIONAL,
            provides: PROVIDES,
            manifest: None,
            restart_on_optional_change: false,
        }
    }

    fn config_fingerprint(&self) -> u64 {
        1
    }

    fn activate(&self, ctx: ActivationContext) -> BoxFuture<'_, Result<(), PluginError>> {
        Box::pin(async move {
            let _http = ctx.require(crate::plugins::http::HTTP)?;
            let _drivers = ctx.require(crate::plugins::drivers::DRIVERS)?;
            let _providers = ctx.require(crate::plugins::providers::PROVIDERS)?;
            let _ = ctx.optional(crate::plugins::secrets::SECRETS);
            ctx.stage_provide(
                TRANSLATION,
                Arc::new(TranslationHub {
                    kernel: ctx.handle(),
                }),
            )?;
            Ok(())
        })
    }
}
