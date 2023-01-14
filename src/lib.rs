#[cfg(feature = "macros")]
extern crate macros;

#[cfg(feature = "json")]
mod json;

#[cfg(feature = "macros")]
pub use macros::{FromRedisValue,ToRedisArgs};
#[cfg(feature = "json")]
pub use json::Json;
