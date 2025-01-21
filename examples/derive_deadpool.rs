use deadpool_redis::{
    // Very important to import inner redis - otherwise macro expansion fails!
    redis,
    redis::{AsyncCommands, ErrorKind, RedisError, RedisResult},
    Config,
    Runtime,
};
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

/// Show a simple async usage of redis_macros traits
/// Just derive the traits and forget them!
#[tokio::main]
async fn main() -> RedisResult<()> {
    // Open new async connection to localhost
    let cfg = Config::from_url("redis://localhost:6379");

    let pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();
    let mut con = pool.get().await.map_err(|_| {
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

    // Set and get back the user in Redis asynchronously, no problem
    let _: () = con.set("user_deadpool", &user).await?;
    let stored_user: User = con.get("user_deadpool").await?;

    // You will get back the same data
    assert_eq!(user, stored_user);

    Ok(())
}

#[test]
fn test_derive_async() {
    assert_eq!(main(), Ok(()));
}
