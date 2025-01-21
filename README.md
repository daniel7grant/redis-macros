# redis-macros

Simple macros and wrappers to [redis-rs](https://github.com/redis-rs/redis-rs/) to automatically serialize and deserialize structs with serde.

## Installation

To install it, simply add the package `redis-macros`. This package is a helper for `redis` and uses `serde` and `serde_json` (or [any other serializer](#using-other-serializer-eg-serde-yaml)), so add these too to the dependencies.

```toml
[dependencies]
redis-macros = "0.5.0"
redis = { version = "0.28" }
serde = { version = "1.0", features = ["derive"] }
serde_json = { version = "1.0" }
```

## Basic usage

### Simple usage

The simplest way to start is to derive `Serialize`, `Deserialize`, `FromRedisValue`, `ToRedisArgs` for any kind of struct... and that's it! You can now get and set these values with regular redis commands:

```rust
use redis::{Client, Commands, RedisResult};
use redis_macros::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
enum Address {
    Street(String),
    Road(String),
}

// Derive the necessary traits
#[derive(Serialize, Deserialize, FromRedisValue, ToRedisArgs)]
struct User {
    id: u32,
    name: String,
    addresses: Vec<Address>,
}

fn main () -> redis::RedisResult<()> {
    let client = redis::Client::open("redis://localhost:6379/")?;
    let mut con = client.get_connection()?;

    let user = User {
        id: 1,
        name: "Ziggy".to_string(),
        addresses: vec![
            Address::Street("Downing".to_string()),
            Address::Road("Abbey".to_string()),
        ],
    };

    // Just use it as you would a primitive
    con.set("user", user)?;
    // user and stored_user will be the same
    let stored_user: User = con.get("user")?;
}
```

For more information, see the [Basic](./examples/derive_basic.rs) or [Async](./examples/derive_async.rs) examples.

### Usage with RedisJSON

You can even use it with RedisJSON, to extract separate parts of the object.

```rust
// Use `JsonCommands`
use redis::{Client, JsonCommands, RedisResult};

// Derive FromRedisValue, ToRedisArgs to the inner struct
#[derive(Serialize, Deserialize, FromRedisValue, ToRedisArgs)]
enum Address { /* ... */ }

// Simple usage is equivalent to set-get
con.json_set("user", "$", &user)?;
let stored_user: User = con.json_get("user", "$")?;

// But you can get deep values - don't forget to derive traits for these too!
let stored_address: Address = con.json_get("user", "$.addresses[0]")?;
```

For more information, see the [RedisJSON](./examples/derive_redisjson.rs) example.

One issue you might be facing is that `redis` already has overrides for some types, for example Vec, String and most primitives. For this you have to use the [Json wrapper](#json-wrapper-with-redisjson).

```rust
// This WON'T work
let stored_addresses: Vec<Address> = con.json_get("user", "$.addresses")?;
```

### Json wrapper with RedisJSON

To deserialize Vecs and primitive types when using RedisJSON, you cannot use the regular types, because these are non-compatible with RedisJSON. However `redis-macros` exports a useful wrapper struct: `Json`. When using RedisJSON, you can wrap your non-structs return values into this:

```rust
use redis_macros::Json;

// Return type can be wrapped into Json
let Json(stored_name): Json<String> = con.json_get("user", "$.name")?;

// It works with Vecs as well
let Json(stored_addresses): Json<Vec<Address>> = con.json_get("user", "$.addresses")?;
// ...now stored_addresses will be equal to user.addresses
```

If you only use RedisJSON, you can even do away with deriving `FromRedisValue` and `ToRedisArgs`, and use `Json` everywhere.

```rust
#[derive(Serialize, Deserialize)]
struct User { /* ... */ }

// This works with simple redis-rs
con.json_set("user", "$", &user)?;
// ...and you can get back with Json wrapper
let Json(stored_user): Json<User> = con.json_get("user", "$")?;
```

For more information, see the [Json Wrapper](./examples/json_wrapper_basic.rs) and [Json Wrapper Advanced](./examples/json_wrapper_modify.rs) examples.

### Using other serializer (e.g. serde-yaml)

In case you want to use another serializer, for example `serde_yaml`, you can install it and use the derives, the same way you would. The only difference should be adding an attribute `redis_serializer` under the derive, with the library you want to serialize with. You can use any Serde serializer as long as they support `from_str` and `to_string` methods. For the full list, see: [Serde data formats](https://serde.rs/#data-formats).

```rust
#[derive(Debug, PartialEq, Serialize, Deserialize, FromRedisValue, ToRedisArgs)]
#[redis_serializer(serde_yaml)]
struct User { /* ... */ }
```

For more information, see the [YAML](./examples/derive_yaml.rs) example.

### Using deadpool-redis or other crates

You can still use the macros if you are using a crate that reexports the `redis` traits, for example [deadpool-redis](https://github.com/bikeshedder/deadpool). The only change you have to make is to `use` the reexported `redis` package explicitly:

```rust
// In the case of deadpool-redis, bring the reexported crate into scope
use deadpool_redis::redis;

// Or if you are importing multiple things from redis, use redis::self
use deadpool_redis::{redis::{self, AsyncCommands}, Config, Runtime};
```

For more information, see the [deadpool-redis](./examples/derive_deadpool.rs) example.

## Testing

You can run the unit tests on the code with `cargo test`:

```sh
cargo test
```

For integration testing, you can run the examples. You will need a RedisJSON compatible redis-server on port 6379, [redis-stack docker image](https://hub.docker.com/r/redis/redis-stack) is recommended:

```sh
docker run -d --rm -p 6379:6379 --name redis docker.io/redis/redis-stack
cargo test --examples
# cleanup the container
docker stop redis
```

## Coverage

For coverage, you can use `grcov`. Simply install `llvm-tools-preview` and `grcov` if you don't have it already:

```sh
rustup component add llvm-tools-preview
cargo install grcov
```

You have to export a few flags to make it work properly:

```sh
export RUSTFLAGS='-Cinstrument-coverage'
export LLVM_PROFILE_FILE='.coverage/cargo-test-%p-%m.profraw'
```

And finally, run the tests and generate the output:

```sh
cargo test
cargo test --examples
grcov .coverage/ -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/
```

Now you can open `./target/debug/coverage/index.html`, and view it in the browser to see the coverage.
