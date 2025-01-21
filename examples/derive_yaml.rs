use redis::{AsyncCommands, Client, ErrorKind, RedisError, RedisResult};
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

/// Derive the traits and set the `redis_serializer` attrribute
#[derive(Debug, PartialEq, Serialize, Deserialize, FromRedisValue, ToRedisArgs)]
#[redis_serializer(serde_yaml)]
struct User {
    id: u32,
    name: String,
    addresses: Vec<Address>,
}

/// This example shows how to use different serializer, in this example serde_yaml
#[tokio::main]
async fn main() -> RedisResult<()> {
    // Open new async connection to localhost
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

    // Set and get back the user in YAML format, no problem
    let _: () = con.set("user_yaml", &user).await?;
    let stored_user: User = con.get("user_yaml").await?;
    assert_eq!(user, stored_user);

    // If we get this out in string, it will be YAML
    let stored_yaml: String = con.get("user_yaml").await?;
    assert_eq!(
        "id: 1
name: Ziggy
addresses:
- !Street Downing
- !Road Abbey
",
        stored_yaml
    );

    Ok(())
}

#[test]
fn test_derive_yaml() {
    assert_eq!(main(), Ok(()));
}
