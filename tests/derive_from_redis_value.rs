use redis::{FromRedisValue, Value};
use redis_macros::FromRedisValue;
use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
enum Address {
    Street(String),
    Road(String),
}

#[derive(Debug, PartialEq, Deserialize, FromRedisValue)]
struct User {
    id: u32,
    name: String,
    addresses: Vec<Address>,
}

#[test]
pub fn it_should_implement_the_from_redis_value_trait() {
    let user = User {
        id: 1,
        name: "Ziggy".to_string(),
        addresses: vec![
            Address::Street("Downing".to_string()),
            Address::Road("Abbey".to_string()),
        ],
    };

    let val = Value::BulkString("{\"id\":1,\"name\":\"Ziggy\",\"addresses\":[{\"Street\":\"Downing\"},{\"Road\":\"Abbey\"}]}".as_bytes().into());
    let result = User::from_redis_value(&val);
    assert_eq!(result, Ok(user));
}

#[test]
pub fn it_should_also_deserialize_if_the_input_is_in_brackets() {
    let user = User {
        id: 1,
        name: "Ziggy".to_string(),
        addresses: vec![
            Address::Street("Downing".to_string()),
            Address::Road("Abbey".to_string()),
        ],
    };

    let val = Value::BulkString("[{\"id\":1,\"name\":\"Ziggy\",\"addresses\":[{\"Street\":\"Downing\"},{\"Road\":\"Abbey\"}]}]".as_bytes().into());
    let result = User::from_redis_value(&val);
    assert_eq!(result, Ok(user));
}

#[test]
pub fn it_should_fail_if_input_is_not_compatible_with_type() {
    let val = Value::BulkString("{}".as_bytes().into());
    let result = User::from_redis_value(&val);
    if let Err(err) = result {
        assert_eq!(err.to_string(), "Response was of incompatible type - TypeError: Response type not deserializable to User with serde_json. (response was bulk-string('\"{}\"'))".to_string());
    } else {
        panic!("Deserialization should fail.");
    }
}

#[test]
pub fn it_should_fail_if_input_is_not_valid_utf8() {
    let val = Value::BulkString(vec![0, 159, 146, 150]); // Some invalid utf8
    let result = User::from_redis_value(&val);
    if let Err(err) = result {
        assert_eq!(err.to_string(), "Response was of incompatible type - TypeError: Response was not valid UTF-8 string. (response was binary-data([0, 159, 146, 150]))".to_string());
    } else {
        panic!("UTF-8 parsing should fail.");
    }
}

#[test]
pub fn it_should_fail_if_input_is_missing() {
    let val = Value::Nil;
    let result = User::from_redis_value(&val);
    if let Err(err) = result {
        assert_eq!(err.to_string(), "Response was of incompatible type - TypeError: Response type was not deserializable to User. (response was nil)".to_string());
    } else {
        panic!("UTF-8 parsing should fail.");
    }
}

#[derive(Debug, PartialEq, Deserialize, FromRedisValue)]
struct Pair<K, V> {
    key: K,
    value: V,
}

#[test]
pub fn it_should_deserialize_struct_with_multiple_generics() {
    let expected = Pair {
        key: 42u32,
        value: "answer".to_string(),
    };
    let json = "{\"key\":42,\"value\":\"answer\"}";
    let val = Value::BulkString(json.as_bytes().into());
    let result = Pair::<u32, String>::from_redis_value(&val);
    assert_eq!(result, Ok(expected));
}