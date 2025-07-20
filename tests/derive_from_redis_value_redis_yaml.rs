use redis::{FromRedisValue, Value};
use redis_macros::FromRedisValue;
use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
enum Address {
    Street(String),
    Road(String),
}

#[derive(Debug, PartialEq, Deserialize, FromRedisValue)]
#[redis_serializer(serde_yaml)]
struct User {
    id: u32,
    name: String,
    addresses: Vec<Address>,
}

#[test]
pub fn it_should_implement_the_from_redis_value_trait_with_redis_yaml() {
    let user = User {
        id: 1,
        name: "Ziggy".to_string(),
        addresses: vec![
            Address::Street("Downing".to_string()),
            Address::Road("Abbey".to_string()),
        ],
    };

    let val = Value::BulkString(
        "id: 1
name: Ziggy
addresses:
- !Street Downing
- !Road Abbey
"
        .as_bytes()
        .into(),
    );
    let result = User::from_redis_value(&val);
    assert_eq!(result, Ok(user));
}
