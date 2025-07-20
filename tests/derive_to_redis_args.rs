use redis::ToRedisArgs;
use redis_macros::ToRedisArgs;
use serde::Serialize;

#[derive(Debug, Serialize)]
enum Address {
    Street(String),
    Road(String),
}

#[derive(Debug, Serialize, ToRedisArgs)]
struct User {
    id: u32,
    name: String,
    addresses: Vec<Address>,
}

#[test]
pub fn it_should_implement_the_to_redis_args_trait() {
    let user = User {
        id: 1,
        name: "Ziggy".to_string(),
        addresses: vec![
            Address::Street("Downing".to_string()),
            Address::Road("Abbey".to_string()),
        ],
    };

    let bytes = user.to_redis_args();
    assert_eq!(bytes[0], "{\"id\":1,\"name\":\"Ziggy\",\"addresses\":[{\"Street\":\"Downing\"},{\"Road\":\"Abbey\"}]}".as_bytes());
}

#[derive(Debug, Serialize, ToRedisArgs)]
struct Pair<K, V> {
    first: K,
    second: V,
}

#[test]
pub fn it_should_implement_to_redis_args_for_multiple_generics() {
    let pair = Pair {
        first: 100u8,
        second: "data".to_string(),
    };
    let bytes = pair.to_redis_args();
    assert_eq!(
        bytes[0],
        "{\"first\":100,\"second\":\"data\"}".as_bytes()
    );
}