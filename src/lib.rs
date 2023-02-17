#[cfg(feature = "server")]
#[macro_use]
extern crate rocket;

#[cfg(feature = "frontend")]
#[macro_use]
extern crate rocket_include_tera;

#[cfg(feature = "api")]
pub mod api;
#[cfg(feature = "client")]
pub mod client;
pub mod data;
#[cfg(feature = "render")]
pub mod render;
#[cfg(feature = "server")]
pub mod server;
