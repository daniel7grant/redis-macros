use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Attribute, DeriveInput, Expr, GenericParam};

fn get_serializer(attrs: Vec<Attribute>, default: &str) -> TokenStream2 {
    let default_token = default.parse::<TokenStream2>().unwrap();

    attrs
        .into_iter()
        .find(|attr| attr.path().is_ident("redis_serializer"))
        .and_then(|attr| {
            let Ok(Expr::Path(path)) = attr.parse_args::<Expr>() else {
                return None;
            };

            Some(path.to_token_stream())
        })
        .unwrap_or(default_token)
}

/// Derive macro for the redis crate's [`FromRedisValue`](../redis/trait.FromRedisValue.html) trait to allow parsing Redis responses to this type.
///
/// *NOTE: This trait requires serde's [`Deserialize`](../serde/trait.Deserialize.html) to also be derived (or implemented).*
///
/// Simply use the `#[derive(FromRedisValue, Deserialize)]` before any structs (or serializable elements).
/// This allows, when using Redis commands, to set this as the return type and deserialize from JSON automatically, while reading from Redis.
///
/// ```rust,no_run
/// # use redis::{Client, Commands, RedisResult};
/// use redis_macros::{FromRedisValue};
/// use serde::{Deserialize};
///
/// #[derive(FromRedisValue, Deserialize)]
/// struct User { id: u32 }
///  
/// # fn main () -> redis::RedisResult<()> {
/// # let client = redis::Client::open("redis://localhost:6379/")?;
/// # let mut con = client.get_connection()?;
/// con.set("user", &r#"{ "id": 1 }"#)?;
/// let user: User = con.get("user")?;  // => User { id: 1 }
/// # Ok(())
/// # }
/// ```
///
/// If you want to use a different serde format, for example `serde_yaml`, you can set this with the `redis_serializer` attribute.
/// The only restriction is to have the deserializer implement the `from_str` function.
///
/// ```rust,no_run
/// use redis_macros::{FromRedisValue};
/// use serde::{Deserialize};
///
/// #[derive(FromRedisValue, Deserialize)]
/// #[redis_serializer(serde_yaml)]
/// struct User { id: u32 }
/// ```
///
/// For more information see the isomorphic pair of this trait: [ToRedisArgs].
#[proc_macro_derive(FromRedisValue, attributes(redis_serializer))]
pub fn from_redis_value_macro(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        attrs,
        generics,
        ..
    } = parse_macro_input!(input as DeriveInput);
    let serializer = get_serializer(attrs, "serde_json");
    let ident_str = format!("{}", ident);
    let serializer_str = format!("{}", serializer);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut where_clause_extended = where_clause.cloned();

    // Add serde constraints for each type parameter
    for param in &generics.params {
        if let GenericParam::Type(type_param) = param {
            let ident = &type_param.ident;
            let constraint = syn::parse_quote! { #ident : ::serde::de::DeserializeOwned };

            if let Some(ref mut w) = where_clause_extended {
                w.predicates.push(constraint);
            } else {
                where_clause_extended = Some(syn::parse_quote! { where #constraint });
            }
        }
    }

    let where_with_serialize = where_clause_extended
        .as_ref()
        .map(|w| quote! { #w })
        .unwrap_or(quote! {});

    let failed_parse_error = quote! {
        ::std::result::Result::Err(::std::format!("Response type not deserializable to {} with {}. (response was {:?})", #ident_str, #serializer_str, v).into())
    };

    // If the parsing failed, the issue might simply be that the user is using a RedisJSON command
    // RedisJSON commands wrap the response into square brackets for some godforsaken reason
    // We can try removing the brackets and try the parse again
    let redis_json_hack = quote! {
        let mut ch = s.chars();
        if ch.next() == ::std::option::Option::Some('[') && ch.next_back() == ::std::option::Option::Some(']') {
            if let ::std::result::Result::Ok(s) = #serializer::from_str(ch.as_str()) {
                ::std::result::Result::Ok(s)
            } else {
                ::std::result::Result::Err(::std::format!("Response type not RedisJSON deserializable to {}. (response was {:?})", #ident_str, v).into())
            }
        } else {
            #failed_parse_error
        }
    };

    // The Redis JSON hack only relevant if we are using serde_json
    let failed_parse = if serializer_str == "serde_json" {
        redis_json_hack
    } else {
        failed_parse_error
    };

    quote! {
        impl #impl_generics redis::FromRedisValue for #ident #ty_generics #where_with_serialize {
            fn from_redis_value(v: redis::Value) -> ::std::result::Result<Self, redis::ParsingError> {
                match v {
                    redis::Value::BulkString(ref bytes) => {
                        if let ::std::result::Result::Ok(s) = ::std::str::from_utf8(bytes) {
                            if let ::std::result::Result::Ok(s) = #serializer::from_str(s) {
                                ::std::result::Result::Ok(s)
                            } else {
                                #failed_parse
                            }
                        } else {
                            ::std::result::Result::Err(::std::format!("Response was not valid UTF-8 string. (response was {:?})", v).into())
                        }
                    },
                    _ => ::std::result::Result::Err(::std::format!("Response type was not deserializable to {}. (response was {:?})", #ident_str, v).into()),
                }
            }
        }
    }
    .into()
}

/// Derive macro for the redis crate's [`ToRedisArgs`](../redis/trait.ToRedisArgs.html) trait to allow passing the type to Redis commands.
///
/// *NOTE: This trait requires serde's [`Serialize`](../serde/trait.Serialize.html) to also be derived (or implemented).*
///
/// ***WARNING: This trait panics if the underlying serialization fails.***
///
/// Simply use the `#[derive(ToRedisArgs, Serialize)]` before any structs (or serializable elements).
/// This allows to pass this type to Redis commands like SET. The type will be serialized into JSON automatically while saving to Redis.
///
/// ```rust,no_run
/// # use redis::{Client, Commands, RedisResult};
/// use redis_macros::{ToRedisArgs};
/// use serde::{Serialize};
///
/// #[derive(ToRedisArgs, Serialize)]
/// struct User { id: u32 }
///  
/// # fn main () -> redis::RedisResult<()> {
/// # let client = redis::Client::open("redis://localhost:6379/")?;
/// # let mut con = client.get_connection()?;
/// con.set("user", User { id: 1 })?;
/// let user: String = con.get("user")?;  // => "{ \"id\": 1 }"
/// # Ok(())
/// # }
/// ```
///
/// If you want to use a different serde format, for example `serde_yaml`, you can set this with the `redis_serializer` attribute.
/// The only restriciton is to have the serializer implement the `to_string` function.
///
/// ```rust,no_run
/// # use redis::{Client, Commands, RedisResult};
/// use redis_macros::{ToRedisArgs};
/// use serde::{Serialize};
///
/// #[derive(ToRedisArgs, Serialize)]
/// #[redis_serializer(serde_yaml)]
/// struct User { id: u32 }
/// ```
///
/// For more information see the isomorphic pair of this trait: [FromRedisValue].
#[proc_macro_derive(ToRedisArgs, attributes(redis_serializer))]
pub fn to_redis_args_macro(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        attrs,
        generics,
        ..
    } = parse_macro_input!(input as DeriveInput);
    let serializer = get_serializer(attrs, "serde_json");

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut where_clause_extended = where_clause.cloned();

    // Add serde constraints for each type parameter
    for param in &generics.params {
        if let GenericParam::Type(type_param) = param {
            let ident = &type_param.ident;
            let constraint = syn::parse_quote! { #ident : ::serde::Serialize };

            if let Some(ref mut w) = where_clause_extended {
                w.predicates.push(constraint);
            } else {
                where_clause_extended = Some(syn::parse_quote! { where #constraint });
            }
        }
    }

    let where_with_serialize = where_clause_extended
        .as_ref()
        .map(|w| quote! { #w })
        .unwrap_or(quote! {});

    quote! {
        impl #impl_generics redis::ToRedisArgs for #ident #ty_generics #where_with_serialize {
            fn write_redis_args<W>(&self, out: &mut W)
            where
                W: ?::std::marker::Sized + redis::RedisWrite,
            {
                let buf = #serializer::to_string(&self).unwrap();
                out.write_arg(&buf.as_bytes())
            }
        }

        impl #impl_generics redis::ToSingleRedisArg for #ident #ty_generics #where_with_serialize {}
    }
    .into()
}
