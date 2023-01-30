use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input, token, Attribute, DeriveInput, Result,
};

struct ParseParenthesed {
    _p: token::Paren,
    field: TokenStream2,
}

impl Parse for ParseParenthesed {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(ParseParenthesed {
            _p: parenthesized!(content in input),
            field: content.parse()?,
        })
    }
}

fn get_serializer(attrs: Vec<Attribute>, default: &str) -> TokenStream2 {
    let default_token = default.parse::<TokenStream2>().unwrap();
    attrs
        .into_iter()
        .find(|a| a.path.segments.len() == 1 && a.path.segments[0].ident == "redis_serializer")
        .map(|Attribute { tokens, .. }| {
            let tokens = tokens.into();
            let ParseParenthesed { field, .. } = parse_macro_input!(tokens as ParseParenthesed);
            field.into()
        })
        .unwrap_or(default_token.into())
        .into()
}

#[proc_macro_derive(FromRedisValue, attributes(redis_serializer))]
pub fn from_redis_value_macro(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, attrs, .. } = parse_macro_input!(input as DeriveInput);
    let serializer = get_serializer(attrs, "serde_json");
    let ident_str = format!("{}", ident);
    let serializer_str = format!("{}", serializer);

    let failed_parse_error = quote! {
        Err(::redis::RedisError::from((
            ::redis::ErrorKind::TypeError,
            "Response was of incompatible type",
            format!("Response type not deserializable to {} with {}. (response was {:?})", #ident_str, #serializer_str, v)
        )))
    };

    // If the parsing failed, the issue might simply be that the user is using a RedisJSON command
    // RedisJSON commands wrap the response into square brackets for some godforesaken reason
    // We can try removing the brackets and try the parse again
    let redis_json_hack = quote! {
        let mut ch = s.chars();
        if ch.next() == Some('[') && ch.next_back() == Some(']') {
            if let Ok(s) = ::#serializer::from_str(ch.as_str()) {
                Ok(s)
            } else {
                Err(::redis::RedisError::from((
                ::redis::ErrorKind::TypeError,
                "Response was of incompatible type",
                format!("Response type not RedisJSON deserializable to {}. (response was {:?})", #ident_str, v)
            )))
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
        impl ::redis::FromRedisValue for #ident {
            fn from_redis_value(v: &::redis::Value) -> ::redis::RedisResult<Self> {
                match *v {
                    ::redis::Value::Data(ref bytes) => {
                        if let Ok(s) = ::std::str::from_utf8(bytes) {
                            if let Ok(s) = ::#serializer::from_str(s) {
                                Ok(s)
                            } else {
                                #failed_parse
                            }
                        } else {
                            Err(::redis::RedisError::from((
                                ::redis::ErrorKind::TypeError,
                                "Response was of incompatible type",
                                format!("Response was not valid UTF-8 string. (response was {:?})", v)
                            )))
                        }
                    },
                    _ => Err(::redis::RedisError::from((
                        ::redis::ErrorKind::TypeError,
                        "Response was of incompatible type",
                        format!("Response type was not deserializable to {}. (response was {:?})", #ident_str, v)
                    ))),
                }
            }
        }
    }
    .into()
}

#[proc_macro_derive(ToRedisArgs, attributes(redis_serializer))]
pub fn to_redis_args_macro(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, attrs, .. } = parse_macro_input!(input as DeriveInput);
    let serializer = get_serializer(attrs, "serde_json");

    quote! {
        impl ::redis::ToRedisArgs for #ident {
            fn write_redis_args<W>(&self, out: &mut W)
            where
                W: ?Sized + ::redis::RedisWrite,
            {
                let buf = ::#serializer::to_string(&self).unwrap();
                out.write_arg(&buf.as_bytes())
            }
        }
    }
    .into()
}
