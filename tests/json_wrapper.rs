use redis::{FromRedisValue, Value};
use redis_macros::Json;
use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
enum Address {
    Street(String),
    Road(String),
}

#[derive(Debug, PartialEq, Deserialize)]
struct User {
    id: u32,
    name: String,
    addresses: Vec<Address>,
}

#[test]
pub fn it_should_deserialize_json_results() {
    let user = User {
        id: 1,
        name: "Ziggy".to_string(),
        addresses: vec![
            Address::Street("Downing".to_string()),
            Address::Road("Abbey".to_string()),
        ],
    };

    let val = Value::BulkString("[{\"id\":1,\"name\":\"Ziggy\",\"addresses\":[{\"Street\":\"Downing\"},{\"Road\":\"Abbey\"}]}]".as_bytes().into());
    let result = Json::<User>::from_redis_value(&val);
    if let Ok(Json(parsed_user)) = result {
        assert_eq!(parsed_user, user);
    } else {
        panic!("JSON parsing should succeed.");
    }
}

#[test]
pub fn it_should_also_deserialize_json_wrappable_arguments() {
    let addresses = vec![
        Address::Street("Downing".to_string()),
        Address::Road("Abbey".to_string()),
    ];

    let val = Value::BulkString(
        "[[{\"Street\":\"Downing\"},{\"Road\":\"Abbey\"}]]"
            .as_bytes()
            .into(),
    );
    // This would fail without the JSON wrapper
    let result = Json::<Vec<Address>>::from_redis_value(&val);
    if let Ok(Json(parsed_addresses)) = result {
        assert_eq!(parsed_addresses, addresses);
    } else {
        panic!("JSON parsing should succeed.");
    }
}

#[test]
pub fn it_should_fail_if_the_result_is_not_redis_json() {
    // RedisJSON responses should have wrapping brackets (i.e. [{...}])
    let val = Value::BulkString("{\"id\":1,\"name\":\"Ziggy\",\"addresses\":[{\"Street\":\"Downing\"},{\"Road\":\"Abbey\"}]}".as_bytes().into());
    let result = Json::<User>::from_redis_value(&val);
    if let Err(err) = result {
        assert_eq!(err.to_string(), "Response was of incompatible type - TypeError: Response type was not JSON type. (response was bulk-string('\"{\\\"id\\\":1,\\\"name\\\":\\\"Ziggy\\\",\\\"addresses\\\":[{\\\"Street\\\":\\\"Downing\\\"},{\\\"Road\\\":\\\"Abbey\\\"}]}\"'))".to_string());
    } else {
        panic!("RedisJSON unwrapping should fail.");
    }
}

#[test]
pub fn it_should_fail_if_input_is_not_compatible_with_type() {
    let val = Value::BulkString("[{}]".as_bytes().into());
    let result = Json::<User>::from_redis_value(&val);
    if let Err(err) = result {
        assert_eq!(err.to_string(), "Response was of incompatible type - TypeError: Response type in JSON was not deserializable. (response was bulk-string('\"[{}]\"'))".to_string());
    } else {
        panic!("Deserialization should fail.");
    }
}

#[test]
pub fn it_should_fail_if_input_is_not_valid_utf8() {
    let val = Value::BulkString(vec![0, 159, 146, 150]); // Some invalid utf8
    let result = Json::<User>::from_redis_value(&val);
    if let Err(err) = result {
        assert_eq!(err.to_string(), "Response was of incompatible type - TypeError: Response was not valid UTF-8 string. (response was binary-data([0, 159, 146, 150]))".to_string());
    } else {
        panic!("UTF-8 parsing should fail.");
    }
}

#[test]
pub fn it_should_fail_if_input_is_missing() {
    let val = Value::Nil;
    let result = Json::<User>::from_redis_value(&val);
    if let Err(err) = result {
        assert_eq!(err.to_string(), "Response was of incompatible type - TypeError: Response type not RedisJSON deserializable. (response was nil)".to_string());
    } else {
        panic!("Value Nil should fail.");
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct Pair<K, V> {
    key: K,
    value: V,
}

#[test]
pub fn it_should_deserialize_json_wrapper_with_multiple_generics() {
    let expected = Pair {
        key: 10u16,
        value: "ok".to_string(),
    };
    let val = Value::BulkString(
        "[{\"key\":10,\"value\":\"ok\"}]".as_bytes().into(),
    );
    let result = Json::<Pair<u16, String>>::from_redis_value(&val);
    if let Ok(Json(parsed)) = result {
        assert_eq!(parsed, expected);
    } else {
        panic!("Generic JSON deserialization should succeed.");
    }
}

#[test]
pub fn it_should_fail_json_wrapper_with_mismatched_types() {
    // key is a string and value is a number, should fail deserializing Pair<u8, String>
    let val = Value::BulkString(
        "[{\"key\":\"bad\",\"value\":123}]".as_bytes().into(),
    );
    let result = Json::<Pair<u8, String>>::from_redis_value(&val);
    assert!(result.is_err());
}