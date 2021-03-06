use super::Tokens;

use std::borrow::Cow;

use proc_macro2::{TokenNode, Literal, Spacing, Delimiter, Term, TokenTree, Span};

fn tt(kind: TokenNode) -> TokenTree {
    TokenTree {
        span: Span::default(),
        kind: kind,
    }
}

/// Types that can be interpolated inside a `quote!(...)` invocation.
pub trait ToTokens {
    /// Write `self` to the given `Tokens`.
    ///
    /// Example implementation for a struct representing Rust paths like
    /// `std::cmp::PartialEq`:
    ///
    /// ```ignore
    /// pub struct Path {
    ///     pub global: bool,
    ///     pub segments: Vec<PathSegment>,
    /// }
    ///
    /// impl ToTokens for Path {
    ///     fn to_tokens(&self, tokens: &mut Tokens) {
    ///         for (i, segment) in self.segments.iter().enumerate() {
    ///             if i > 0 || self.global {
    ///                 tokens.append("::");
    ///             }
    ///             segment.to_tokens(tokens);
    ///         }
    ///     }
    /// }
    /// ```
    fn to_tokens(&self, &mut Tokens);

    /// Convert `self` directly into a `Tokens` object.
    ///
    /// This method is implicitly implemented using `to_tokens`, and acts as a
    /// convenience method for consumers of the `ToTokens` trait.
    fn into_tokens(self) -> Tokens where Self: Sized {
        let mut tokens = Tokens::new();
        self.to_tokens(&mut tokens);
        tokens
    }
}

impl<'a, T: ?Sized + ToTokens> ToTokens for &'a T {
    fn to_tokens(&self, tokens: &mut Tokens) {
        (**self).to_tokens(tokens);
    }
}

impl<'a, T: ?Sized + ToOwned + ToTokens> ToTokens for Cow<'a, T> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        (**self).to_tokens(tokens);
    }
}

impl<T: ?Sized + ToTokens> ToTokens for Box<T> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        (**self).to_tokens(tokens);
    }
}

impl<T: ToTokens> ToTokens for Option<T> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        if let Some(ref t) = *self {
            t.to_tokens(tokens);
        }
    }
}

impl ToTokens for Term {
    fn to_tokens(&self, tokens: &mut Tokens) {
        tokens.append(tt(TokenNode::Term(*self)));
    }
}

impl ToTokens for str {
    fn to_tokens(&self, tokens: &mut Tokens) {
        tokens.append(tt(TokenNode::Literal(Literal::string(self))));
    }
}

impl ToTokens for String {
    fn to_tokens(&self, tokens: &mut Tokens) {
        self.as_str().to_tokens(tokens);
    }
}

macro_rules! primitive {
    ($($t:ident)*) => ($(
        impl ToTokens for $t {
            fn to_tokens(&self, tokens: &mut Tokens) {
                tokens.append(tt(TokenNode::Literal(Literal::$t(*self))));
            }
        }
    )*)
}

primitive! {
    i8 i16 i32 i64 isize
    u8 u16 u32 u64 usize
    f32 f64
}

impl ToTokens for char {
    fn to_tokens(&self, tokens: &mut Tokens) {
        tokens.append(tt(TokenNode::Literal(Literal::character(*self))));
    }
}

impl ToTokens for bool {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let word = if *self {"true"} else {"false"};
        tokens.append(tt(TokenNode::Term(Term::intern(word))));
    }
}

/// Wrap a `&str` so it interpolates as a byte-string: `b"abc"`.
#[derive(Debug)]
pub struct ByteStr<'a>(pub &'a str);

impl<'a> ToTokens for ByteStr<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let lit = Literal::byte_string(self.0.as_bytes());
        tokens.append(tt(TokenNode::Literal(lit)));
    }
}

impl<T: ToTokens> ToTokens for [T] {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let mut sub = Tokens::new();
        for item in self {
            item.to_tokens(&mut sub);
            sub.append(tt(TokenNode::Op(',', Spacing::Alone)));
        }
        tokens.append(tt(TokenNode::Group(Delimiter::Bracket, sub.into())));
    }
}

impl<T: ToTokens> ToTokens for Vec<T> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        self[..].to_tokens(tokens)
    }
}

macro_rules! array_impls {
    ($($N:expr)+) => {
        $(
            impl<T: ToTokens> ToTokens for [T; $N] {
                fn to_tokens(&self, tokens: &mut Tokens) {
                    self[..].to_tokens(tokens)
                }
            }
        )+
    }
}

array_impls! {
     0  1  2  3  4  5  6  7  8  9
    10 11 12 13 14 15 16 17 18 19
    20 21 22 23 24 25 26 27 28 29
    30 31 32
}

macro_rules! tuple_impls {
    ($(
        $Tuple:ident {
            $(($idx:tt) -> $T:ident)*
        }
    )+) => {
        $(
            impl<$($T: ToTokens),*> ToTokens for ($($T,)*) {
                fn to_tokens(&self, tokens: &mut Tokens) {
                    let mut _sub = Tokens::new();
                    $(
                        self.$idx.to_tokens(&mut _sub);
                        _sub.append(tt(TokenNode::Op(',', Spacing::Alone)));
                    )*
                    tokens.append(tt(TokenNode::Group(Delimiter::Parenthesis,
                                                      _sub.into())));
                }
            }
        )+
    }
}

tuple_impls! {
    Tuple0 {}
    Tuple1 {
        (0) -> A
    }
    Tuple2 {
        (0) -> A
        (1) -> B
    }
    Tuple3 {
        (0) -> A
        (1) -> B
        (2) -> C
    }
    Tuple4 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
    }
    Tuple5 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
    }
    Tuple6 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
    }
    Tuple7 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
    }
    Tuple8 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
    }
    Tuple9 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
        (8) -> I
    }
    Tuple10 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
        (8) -> I
        (9) -> J
    }
    Tuple11 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
        (8) -> I
        (9) -> J
        (10) -> K
    }
    Tuple12 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
        (8) -> I
        (9) -> J
        (10) -> K
        (11) -> L
    }
}
