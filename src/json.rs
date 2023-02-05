use redis::{RedisResult, Value};
use serde::de::DeserializeOwned;

/// Json struct is a wrapper to handle the return types from the RedisJSON commands.
/// 
/// RedisJSON usually returns values in square brackets, which you usually had to handle manually:
/// 
/// ```rust,no_run
/// # use redis::{Client, JsonCommands, RedisResult};
/// # use redis_macros::{FromRedisValue, ToRedisArgs, Json};
/// # use serde::{Deserialize, Serialize};
/// # #[derive(Serialize, Deserialize)]
/// # struct User { id: u32 }
/// # 
/// # fn main () -> redis::RedisResult<()> {
/// # let client = redis::Client::open("redis://localhost:6379/")?;
/// # let mut con = client.get_connection()?;
/// # con.json_set("user", "$", &r#"{ "id": 1 }"#)?;
/// // You have to manually deserialize this and pull from the Vec
/// let user_id: String = con.json_get("user", "$.id")?;  // => "[1]"
/// # Ok(())
/// # }
/// ```
/// 
/// Instead, `Json` implements the `FromRedisValue` trait, removes the square brackets and deserializes from JSON.
/// For this your type don't even have to implement `FromRedisValue`, it only requires to be serde `Deserialize`-able.
/// 
/// ```rust,no_run
/// # use redis::{Client, JsonCommands, RedisResult};
/// # use redis_macros::Json;
/// # use serde::{Deserialize, Serialize};
/// #[derive(Serialize, Deserialize)]
/// struct User { id: u32 }
///  
/// # fn main () -> redis::RedisResult<()> {
/// # let client = redis::Client::open("redis://localhost:6379/")?;
/// # let mut con = client.get_connection()?;
/// # con.json_set("user", "$", &r#"{ "id": 1 }"#)?;
/// let Json(user_id): Json<u32> = con.json_get("user", "$.id")?;  // => 1
/// let Json(user): Json<User> = con.json_get("user", "$")?;  // => User { id: 1 }
/// # Ok(())
/// # }
/// ``` 
/// 
/// This command is designed to use RedisJSON commands. You could probably use this type
/// to parse normal command outputs, but it removes the first and last character
/// so it is not recommended.
///  
#[derive(Debug)]
pub struct Json<T>(
    /// The inner type to deserialize
    pub T
);

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
