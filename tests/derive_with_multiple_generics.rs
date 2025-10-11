use redis::{FromRedisValue, ToRedisArgs, Value};
use redis_macros::{FromRedisValue, Json, ToRedisArgs};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, FromRedisValue, ToRedisArgs)]
struct Pair<K, V> {
    key: K,
    value: V,
}

#[test]
pub fn it_should_deserialize_struct_with_multiple_generics() {
    let pair = Pair {
        key: 42u32,
        value: "answer".to_string(),
    };
    let bytes = pair.to_redis_args();
    let json = "{\"key\":42,\"value\":\"answer\"}".as_bytes();
    assert_eq!(bytes[0], json);

    let val = Value::BulkString(json.into());
    let result = Pair::<u32, String>::from_redis_value(val);
    assert_eq!(result, Ok(pair));
}

#[derive(Debug, PartialEq, Deserialize)]
struct PairWithoutTrait<K, V> {
    key: K,
    value: V,
}

#[test]
pub fn it_should_deserialize_json_wrapper_with_multiple_generics() {
    let expected = PairWithoutTrait {
        key: 10u16,
        value: "ok".to_string(),
    };
    let val = Value::BulkString("[{\"key\":10,\"value\":\"ok\"}]".as_bytes().into());
    let result = Json::<PairWithoutTrait<u16, String>>::from_redis_value(val);
    if let Ok(Json(parsed)) = result {
        assert_eq!(parsed, expected);
    } else {
        panic!("Generic JSON deserialization should succeed.");
    }
}
