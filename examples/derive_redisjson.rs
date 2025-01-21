use redis::{Client, JsonAsyncCommands, ErrorKind, RedisError, RedisResult};
use redis_macros::{FromRedisValue, ToRedisArgs, Json};
use serde::{Deserialize, Serialize};

/// Define structs to hold the data
/// If we want to JSON.GET the inner struct we have to dderive FromRedisValue
#[derive(Debug, PartialEq, Serialize, Deserialize, FromRedisValue)]
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

/// Show a usage of redis macros with RedisJSON commands
/// You can use RedisJSON paths to extract the inner paths
#[tokio::main]
async fn main() -> RedisResult<()> {
    // Open new connection to localhost
    let client = Client::open("redis://localhost:6379")?;
    let mut con = client.get_multiplexed_async_connection().await.map_err(|_| {
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

    // Set and get the data in Redis with RedisJSON
    let _: () = con.json_set("user_json", "$", &user).await?;
    let stored_user: User = con.json_get("user_json", "$").await?;
    assert_eq!(user, stored_user);

    // Even with inner structs (don't forget to derive FromRedisValue for them)
    let stored_address: Address = con.json_get("user_json", "$.addresses[0]").await?;
    assert_eq!(user.addresses[0], stored_address);

    // However it doesn't work with types that redis overrides (e.g. String, Vec)
    // You have to wrap them in Json instead
    let Json(stored_name): Json<String> = con.json_get("user_json", "$.name").await?;
    assert_eq!(user.name, stored_name);
    let Json(stored_addresses): Json<Vec<Address>> = con.json_get("user_json", "$.addresses").await?;
    assert_eq!(user.addresses, stored_addresses);

    Ok(())
}

#[test]
fn test_derive_redisjson() {
    assert_eq!(main(), Ok(()));
}
