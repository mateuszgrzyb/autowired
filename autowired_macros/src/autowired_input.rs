use syn::{Error, ItemStruct, ItemFn, parse::{Parse, ParseStream}, Attribute, Visibility, Token};

pub enum AutowiredInput {
  Struct(ItemStruct),
  Fn(ItemFn),
  AsyncFn(ItemFn),
}

impl Parse for AutowiredInput {
  fn parse(input: ParseStream) -> Result<Self, Error> {
    let attrs = input.call(Attribute::parse_outer)?;
    let vis = input.parse::<Visibility>()?;

    let lh = input.lookahead1();

    let result = if lh.peek(Token![struct]) {
      Self::Struct(ItemStruct {
        attrs,
        vis,
        ..input.parse()?
      })
    } else if lh.peek(Token![fn]) {
      Self::Fn(ItemFn {
        attrs,
        vis,
        ..input.parse()?
      })
    } else if lh.peek(Token![async]) {
      Self::AsyncFn(ItemFn {
        attrs,
        vis,
        ..input.parse()?
      })
    } else {
      return Err(lh.error());
    };

    Ok(result)
  }
}
