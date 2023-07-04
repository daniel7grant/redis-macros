use redis::{Client, Commands, ErrorKind, RedisError, RedisResult};
use redis_macros::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

/// Define structs to hold the data
#[derive(Debug, PartialEq, Serialize, Deserialize, FromRedisValue, ToRedisArgs)]
struct Container<T> {
    inner: T,
}

/// You can use generics with the traits
fn main() -> RedisResult<()> {
    // Open new connection to localhost
    let client = Client::open("redis://localhost:6379")?;
    let mut con = client.get_connection().map_err(|_| {
        RedisError::from((
            ErrorKind::InvalidClientConfig,
            "Cannot connect to localhost:6379. Try starting a redis-server process or container.",
        ))
    })?;

    // Define the data you want to store in Redis.
    let container = Container {
        inner: "contained",
    };

    // Set and get back the container in Redis, no problem
    con.set("container", &container)?;
    let stored_container: Container<&str> = con.get("container")?;

    // You will get back the same data
    assert_eq!(container, stored_container);

    Ok(())
}

#[test]
fn test_derive_basic() {
    assert_eq!(main(), Ok(()));
}
