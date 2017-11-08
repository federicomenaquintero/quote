use tokens::*;
use to_tokens::ToTokens;

pub trait SpanlessEq {
    fn spanless_eq(a: &Self, b: &Self) -> bool;
}

pub fn spanless_eq<T>(a: &T, b: &T) -> bool
    where T: ToTokens
{
    let mut ta = Tokens::new();
    a.to_tokens(&mut ta);

    let mut tb = Tokens::new();
    b.to_tokens(&mut tb);

    SpanlessEq::spanless_eq(&ta, &tb)
}
