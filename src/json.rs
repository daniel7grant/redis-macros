use redis::{RedisResult, Value};
use serde::de::DeserializeOwned;

#[derive(Debug)]
pub struct Json<T>(pub T);

impl<T> ::redis::FromRedisValue for Json<T>
where
    T: DeserializeOwned,
{
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        match *v {
            Value::Data(ref bytes) => {
                if let Ok(s) = ::std::str::from_utf8(bytes) {
                    let mut ch = s.chars();
                    if ch.next() == Some('[') && ch.next_back() == Some(']') {
                        if let Ok(t) = serde_json::from_str(ch.as_str()) {
                            Ok(Json(t))
                        } else {
                            Err(::redis::RedisError::from((
                                ::redis::ErrorKind::TypeError,
                                "Response was of incompatible type",
                                format!("Response type in JSON was not deserializable. (response was {v:?})"),
                            )))
                        }
                    } else {
                        Err(::redis::RedisError::from((
                            ::redis::ErrorKind::TypeError,
                            "Response was of incompatible type",
                            format!("Response type was not JSON type. (response was {v:?})"),
                        )))
                    }
                } else {
                    Err(::redis::RedisError::from((
                        ::redis::ErrorKind::TypeError,
                        "Response was of incompatible type",
                        format!("Response was not valid UTF-8 string. (response was {v:?})"),
                    )))
                }
            }
            _ => Err(::redis::RedisError::from((
                ::redis::ErrorKind::TypeError,
                "Response was of incompatible type",
                format!("Response type not RedisJSON deserializable. (response was {v:?})"),
            ))),
        }
    }
}
