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

// new test at the end of tests/derive_from_redis_value_redis_yaml.rs

#[derive(Debug, PartialEq, Deserialize, FromRedisValue)]
#[redis_serializer(serde_yaml)]
struct Pair<K, V> {
    key: K,
    value: V,
}

#[test]
pub fn it_should_deserialize_struct_with_multiple_generics_with_yaml() {
    let expected = Pair {
        key: 42u32,
        value: "answer".to_string(),
    };
    let yaml = "key: 42\nvalue: answer\n";
    let val = Value::BulkString(yaml.as_bytes().into());
    let result = Pair::<u32, String>::from_redis_value(&val);
    assert_eq!(result, Ok(expected));
}