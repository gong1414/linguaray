//! Local HTTP API: auth, origin rejection, rate limit, S0 routes.

mod policy;
mod server;

pub use policy::{authorize, reject_origin, RateLimiter};
pub use server::{
    start_listener, ApiHooks, ExternalApiHandle, ExternalApiStatus, DEFAULT_PORT,
};
