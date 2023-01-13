use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(FromRedisValue)]
pub fn from_redis_value_macro(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input as DeriveInput);

    quote! {
        impl ::redis::FromRedisValue for #ident {
            fn from_redis_value(v: &::redis::Value) -> ::redis::RedisResult<Self> {
                match *v {
                    ::redis::Value::Data(ref bytes) => match ::serde_json::from_slice(&bytes) {
                        Ok(t) => Ok(t),
                        Err(_) => Err(::redis::RedisError::from((
                            ::redis::ErrorKind::TypeError,
                            "Response was of incompatible type",
                            format!("Response type not JSON serializable. (response was {:?})", v)
                        )))
                    },
                    _ => Err(::redis::RedisError::from((
                        ::redis::ErrorKind::TypeError,
                        "Response was of incompatible type",
                        format!("Response type not JSON serializable. (response was {:?})", v)
                    ))),
                }
            }
        }
    }
    .into()
}

#[proc_macro_derive(ToRedisArgs)]
pub fn to_redis_args_macro(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input as DeriveInput);

    quote! {
        impl ::redis::ToRedisArgs for #ident {
            fn write_redis_args<W>(&self, out: &mut W)
            where
                W: ?Sized + ::redis::RedisWrite,
            {
                let mut buf = ::serde_json::to_vec(&self).unwrap();
                out.write_arg(&buf)
            }
        }
    }
    .into()
}
