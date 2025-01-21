use redis::{Client, Commands, ErrorKind, RedisError, RedisResult};
use redis_macros::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

/// Define structs to hold the data
/// Children structs don't have to implement FromRedisValue, ToRedisArgs, unless you want to use them as top level
/// They have to implement serde traits though!
#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Address {
    Street(String),
    Road(String),
}

/// Don't forget to implement serde traits and redis traits!
#[derive(Debug, PartialEq, Serialize, Deserialize, FromRedisValue, ToRedisArgs)]
struct User {
    id: u32,
    name: String,
    addresses: Vec<Address>,
}

/// Show a simple usage of redis_macros traits
/// Just derive the traits and forget them!
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
    let user = User {
        id: 1,
        name: "Ziggy".to_string(),
        addresses: vec![
            Address::Street("Downing".to_string()),
            Address::Road("Abbey".to_string()),
        ],
    };

    // Set and get back the user in Redis, no problem
    let _: () = con.set("user", &user)?;
    let stored_user: User = con.get("user")?;

    // You will get back the same data
    assert_eq!(user, stored_user);

    Ok(())
}

#[test]
fn test_derive_basic() {
    assert_eq!(main(), Ok(()));
}
