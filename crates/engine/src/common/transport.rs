use std::sync::{OnceLock, RwLock};

use reqwest::{Client, ClientBuilder, NoProxy, RequestBuilder, Response};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum NetworkProxyMode {
    #[default]
    System,
    Direct,
    Custom,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NetworkProxyConfig {
    pub mode: NetworkProxyMode,
    pub url: String,
    pub bypass: String,
}

static NETWORK_PROXY: OnceLock<RwLock<NetworkProxyConfig>> = OnceLock::new();

fn network_proxy() -> &'static RwLock<NetworkProxyConfig> {
    NETWORK_PROXY.get_or_init(|| RwLock::new(NetworkProxyConfig::default()))
}

/// Applies one process-wide desktop proxy policy. LinguaRay owns a single
/// runtime in production, and every provider client is rebuilt when this
/// setting changes.
pub fn configure_network_proxy(config: NetworkProxyConfig) -> Result<(), String> {
    build_client_from_config(&config, Client::builder())?;
    *network_proxy()
        .write()
        .map_err(|error| format!("proxy settings lock poisoned: {error}"))? = config;
    Ok(())
}

pub fn current_network_proxy() -> Result<NetworkProxyConfig, String> {
    network_proxy()
        .read()
        .map(|config| config.clone())
        .map_err(|error| format!("proxy settings lock poisoned: {error}"))
}

pub fn build_http_client() -> Result<Client, String> {
    build_http_client_with(Client::builder())
}

pub fn build_http_client_with(builder: ClientBuilder) -> Result<Client, String> {
    let config = network_proxy()
        .read()
        .map_err(|error| format!("proxy settings lock poisoned: {error}"))?
        .clone();
    build_client_from_config(&config, builder)
}

fn build_client_from_config(
    config: &NetworkProxyConfig,
    builder: ClientBuilder,
) -> Result<Client, String> {
    let builder = match config.mode {
        NetworkProxyMode::System => builder,
        NetworkProxyMode::Direct => builder.no_proxy(),
        NetworkProxyMode::Custom => {
            let url = config.url.trim();
            if url.is_empty() {
                return Err("custom proxy URL is required".to_owned());
            }
            let parsed =
                reqwest::Url::parse(url).map_err(|error| format!("invalid proxy URL: {error}"))?;
            if parsed.scheme() != "http" && parsed.scheme() != "https" {
                return Err("proxy URL must use http or https".to_owned());
            }
            if parsed.host_str().is_none() || parsed.port_or_known_default().is_none() {
                return Err("proxy URL must include a host and port".to_owned());
            }
            if !parsed.username().is_empty() || parsed.password().is_some() {
                return Err("proxy credentials are not stored in normal settings".to_owned());
            }
            let proxy = reqwest::Proxy::all(parsed)
                .map_err(|error| format!("invalid proxy configuration: {error}"))?
                .no_proxy(NoProxy::from_string(config.bypass.trim()));
            builder.proxy(proxy)
        }
    };
    builder
        .build()
        .map_err(|error| format!("failed to build HTTP client: {error}"))
}

/// Small HTTP transport shared by traditional provider adapters.
#[derive(Debug, Clone)]
pub struct HttpClient {
    endpoint: String,
    inner: Client,
}

#[allow(dead_code)]
impl HttpClient {
    pub fn new(endpoint: impl Into<String>, inner: Client) -> Self {
        Self {
            endpoint: endpoint.into().trim_end_matches('/').to_owned(),
            inner,
        }
    }

    pub fn proxy_aware(endpoint: impl Into<String>) -> Result<Self, String> {
        Ok(Self::new(endpoint, build_http_client()?))
    }

    pub fn client(&self) -> &Client {
        &self.inner
    }

    pub fn base_url(&self) -> &str {
        &self.endpoint
    }

    pub fn join_url(&self, path: &str) -> String {
        let suffix = if path.starts_with('/') {
            path.to_owned()
        } else {
            format!("/{path}")
        };
        format!("{}{suffix}", self.endpoint)
    }

    pub fn post(&self, path: &str) -> RequestBuilder {
        self.inner.post(self.join_url(path))
    }

    pub fn get(&self, path: &str) -> RequestBuilder {
        self.inner.get(self.join_url(path))
    }

    pub async fn execute(&self, request: RequestBuilder) -> reqwest::Result<Response> {
        request.send().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_direct_and_http_proxy_modes() {
        build_client_from_config(
            &NetworkProxyConfig {
                mode: NetworkProxyMode::Direct,
                ..NetworkProxyConfig::default()
            },
            Client::builder(),
        )
        .expect("direct client");
        build_client_from_config(
            &NetworkProxyConfig {
                mode: NetworkProxyMode::Custom,
                url: "http://127.0.0.1:7890".to_owned(),
                bypass: "localhost,127.0.0.1".to_owned(),
            },
            Client::builder(),
        )
        .expect("proxy client");
    }

    #[test]
    fn rejects_proxy_secrets_and_non_http_schemes() {
        for url in [
            "socks5://127.0.0.1:1080",
            "http://user:secret@localhost:7890",
        ] {
            let result = build_client_from_config(
                &NetworkProxyConfig {
                    mode: NetworkProxyMode::Custom,
                    url: url.to_owned(),
                    bypass: String::new(),
                },
                Client::builder(),
            );
            assert!(result.is_err());
        }
    }
}
