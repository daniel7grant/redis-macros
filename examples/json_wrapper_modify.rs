use redis::{Client, ErrorKind, JsonAsyncCommands, RedisError, RedisResult};
use redis_macros::Json;
use serde::{Deserialize, Serialize};

/// Define structs to hold the data
#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Address {
    Street(String),
    Road(String),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    addresses: Vec<Address>,
}

/// This example shows how to use more exotic RedisJSON commands
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

    // Wrap the data in `Json(..)` when passing to and from Redis
    let _: () = con.json_set("user_wrapped_modify", "$", &user).await?;

    // Modify inner values with JSON.SET
    let _: () = con.json_set("user_wrapped_modify", "$.name", &"Bowie")
        .await?;
    let Json(stored_name): Json<String> = con.json_get("user_wrapped_modify", "$.name").await?;
    assert_eq!("Bowie", stored_name);

    // Increment numbers with JSON.NUMINCRBY
    let _: () = con.json_num_incr_by("user_wrapped_modify", "$.id", 1)
        .await?;
    let Json(stored_id): Json<u32> = con.json_get("user_wrapped_modify", "$.id").await?;
    assert_eq!(2, stored_id);

    // Append item to array with JSON.ARR_APPEND
    let _: () = con.json_arr_append(
        "user_wrapped_modify",
        "$.addresses",
        &Address::Street("Oxford".to_string()),
    )
    .await?;
    let Json(stored_addresses): Json<Vec<Address>> =
        con.json_get("user_wrapped_modify", "$.addresses").await?;
    assert_eq!(
        vec![
            Address::Street("Downing".to_string()),
            Address::Road("Abbey".to_string()),
            Address::Street("Oxford".to_string())
        ],
        stored_addresses
    );

    Ok(())
}

#[test]
fn test_json_wrapper_modify() {
    assert_eq!(main(), Ok(()));
}
