//! Simple macros and wrappers to [redis-rs](https://github.com/redis-rs/redis-rs/)
//! to automatically serialize and deserialize structs with serde.
//!
//! ## Simple usage
//!
//! The simplest way to start is to derive `Serialize`, `Deserialize`,
//! [`FromRedisValue`], [`ToRedisArgs`] for any kind of struct... and that's it!
//! You can now get and set these values with regular redis commands:
//!
//! ```rust,no_run
//! use redis::{Client, Commands, RedisResult};
//! use redis_macros_derive::{FromRedisValue, ToRedisArgs};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize)]
//! enum Address {
//!     Street(String),
//!     Road(String),
//! }
//!
//! // Derive the necessary traits
//! #[derive(Serialize, Deserialize, FromRedisValue, ToRedisArgs)]
//! struct User {
//!     id: u32,
//!     name: String,
//!     addresses: Vec<Address>,
//! }
//!
//! fn main () -> redis::RedisResult<()> {
//!     let client = redis::Client::open("redis://localhost:6379/")?;
//!     let mut con = client.get_connection()?;
//!
//!     let user = User {
//!         id: 1,
//!         name: "Ziggy".to_string(),
//!         addresses: vec![
//!             Address::Street("Downing".to_string()),
//!             Address::Road("Abbey".to_string()),
//!         ],
//!     };
//!
//!     // Just use it as you would a primitive
//!     let _: () = con.set("user", user)?;
//!     // user and stored_user will be the same
//!     let stored_user: User = con.get("user")?;
//!     # Ok(())
//! }
//! ```
//!
//! ## Usage with RedisJSON
//!
//! You can even use it with RedisJSON, to extract separate parts of the object.
//!
//! ```rust,no_run
//! # use redis::{Client, RedisResult};
//! use redis::JsonCommands;
//! # use redis_macros_derive::{FromRedisValue, ToRedisArgs};
//! # use serde::{Deserialize, Serialize};
//!
//! // Derive FromRedisValue, ToRedisArgs to the inner struct
//! #[derive(Serialize, Deserialize, FromRedisValue, ToRedisArgs)]
//! enum Address { Street(String), Road(String) }
//! # #[derive(Serialize, Deserialize, FromRedisValue, ToRedisArgs)]
//! # struct User { id: u32, name: String, addresses: Vec<Address> }
//!
//! # fn main () -> redis::RedisResult<()> {
//! # let client = redis::Client::open("redis://localhost:6379/")?;
//! # let mut con = client.get_connection()?;
//! # let user = User { id: 1, name: "Ziggy".to_string(), addresses: vec![ Address::Street("Downing".to_string()), Address::Road("Abbey".to_string()) ] };
//! // Simple usage is equivalent to set-get
//! let _: () = con.json_set("user", "$", &user)?;
//! let stored_user: User = con.json_get("user", "$")?;
//!
//! // But you can get deep values - don't forget to derive traits for these too!
//! let stored_address: Address = con.json_get("user", "$.addresses[0]")?;
//! # Ok(())
//! # }
//! ```
//!
//! One issue you might be facing is that `redis` already has overrides for some types,
//! for example Vec, String and most primitives. For this you have to use the [Json wrapper](#json-wrapper-with-redisjson).
//!
//! ```rust,no_run
//! # use redis::{Client, JsonCommands, RedisResult};
//! # use redis_macros_derive::{FromRedisValue, ToRedisArgs};
//! # use serde::{Deserialize, Serialize};
//! # #[derive(Serialize, Deserialize, FromRedisValue, ToRedisArgs)]
//! # enum Address { Street(String), Road(String) }
//! # #[derive(Serialize, Deserialize, FromRedisValue, ToRedisArgs)]
//! # struct User { id: u32, name: String, addresses: Vec<Address> }
//! # fn main () -> redis::RedisResult<()> {
//! # let client = redis::Client::open("redis://localhost:6379/")?;
//! # let mut con = client.get_connection()?;
//! # let user = User { id: 1, name: "Ziggy".to_string(), addresses: vec![ Address::Street("Downing".to_string()), Address::Road("Abbey".to_string()) ] };
//! # let _: () = con.json_set("user", "$", &user)?;
//! // This WON'T work
//! let stored_addresses: Vec<Address> = con.json_get("user", "$.addresses")?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Json wrapper with RedisJSON
//!
//! To deserialize Vecs and primitive types when using RedisJSON, you cannot use the regular types,
//! because these are non-compatible with RedisJSON. However `redis-macros` exports a useful wrapper
//! struct: [`Json`]. When using RedisJSON, you can wrap your non-structs return values into this:
//!
//! ```rust,no_run
//! use redis_macros::Json;
//! # use redis::{Client, JsonCommands, RedisResult};
//! # use redis_macros_derive::{FromRedisValue, ToRedisArgs};
//! # use serde::{Deserialize, Serialize};
//! # #[derive(Serialize, Deserialize, FromRedisValue, ToRedisArgs)]
//! # enum Address { Street(String), Road(String) }
//! # #[derive(Serialize, Deserialize, FromRedisValue, ToRedisArgs)]
//! # struct User { id: u32, name: String, addresses: Vec<Address> }
//! # fn main () -> redis::RedisResult<()> {
//! # let client = redis::Client::open("redis://localhost:6379/")?;
//! # let mut con = client.get_connection()?;
//! # let user = User { id: 1, name: "Ziggy".to_string(), addresses: vec![ Address::Street("Downing".to_string()), Address::Road("Abbey".to_string()) ] };
//! # let _: () = con.json_set("user", "$", &user)?;

//! // Return type can be wrapped into Json
//! let Json(stored_name): Json<String> = con.json_get("user", "$.name")?;
//!
//! // It works with Vecs as well
//! let Json(stored_addresses): Json<Vec<Address>> = con.json_get("user", "$.addresses")?;
//! // ...now stored_addresses will be equal to user.addresses
//! # Ok(())
//! # }
//! ```
//!
//! If you only use RedisJSON, you can even do away with deriving `FromRedisValue` and `ToRedisArgs`, and use `Json` everywhere.
//!
//! ```rust,no_run
//! # use redis::{Client, JsonCommands, RedisResult};
//! # use redis_macros::Json;
//! # use redis_macros_derive::{FromRedisValue, ToRedisArgs};
//! # use serde::{Deserialize, Serialize};
//! # #[derive(Serialize, Deserialize, FromRedisValue, ToRedisArgs)]
//! # enum Address { Street(String), Road(String) }
//! #[derive(Serialize, Deserialize)]
//! struct User { /* .. */ }
//! # fn main () -> redis::RedisResult<()> {
//! # let client = redis::Client::open("redis://localhost:6379/")?;
//! # let mut con = client.get_connection()?;
//! # let user = User {};
//! # let _: () = con.json_set("user", "$", &user)?;
//!
//! // This works with simple redis-rs
//! let _: () = con.json_set("user", "$", &user)?;
//! // ...and you can get back with Json wrapper
//! let Json(stored_user): Json<User> = con.json_get("user", "$")?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Using other serializer (e.g. serde-yaml)
//!
//! In case you want to use another serializer, for example `serde_yaml`, you can install it and use the derives,
//! the same way you would. The only difference should be adding an attribute `redis_serializer` under the derive,
//! with the library you want to serialize with. You can use any Serde serializer as long as they support
//! `from_str` and `to_string` methods. For the full list, see: [Serde data formats](https://serde.rs/#data-formats).
//!
//! ```rust,no_run
//! # use redis::{Client, JsonCommands, RedisResult};
//! # use redis_macros_derive::{FromRedisValue, ToRedisArgs};
//! # use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, PartialEq, Serialize, Deserialize, FromRedisValue, ToRedisArgs)]
//! #[redis_serializer(serde_yaml)]
//! struct User { /* ... */ }
//! ```

#[cfg(feature = "macros")]
extern crate redis_macros_derive;

#[cfg(feature = "json")]
mod json;

#[cfg(feature = "json")]
pub use json::Json;

/// Derive macro for the redis crate's [`FromRedisValue`](../redis/trait.FromRedisValue.html) trait to allow parsing Redis responses to this type.
/// 
/// For more information see the `redis_macros_derive` crate: [`FromRedisValue`](../redis_macros_derive/derive.FromRedisValue.html)
#[cfg(feature = "macros")]
pub use redis_macros_derive::FromRedisValue;

/// Derive macro for the redis crate's [`ToRedisArgs`](../redis/trait.ToRedisArgs.html) trait to allow passing the type to Redis commands.
/// 
/// For more information see the `redis_macros_derive` crate: [`ToRedisArgs`](../redis_macros_derive/derive.FromRedisValue.html)
#[cfg(feature = "macros")]
pub use redis_macros_derive::ToRedisArgs;
