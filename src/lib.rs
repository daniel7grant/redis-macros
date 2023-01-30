#[cfg(feature = "macros")]
extern crate redis_macros_derive;

#[cfg(feature = "json")]
mod json;

#[cfg(feature = "macros")]
pub use redis_macros_derive::{FromRedisValue,ToRedisArgs};
#[cfg(feature = "json")]
pub use json::Json;
