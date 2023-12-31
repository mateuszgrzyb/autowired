use darling::{FromMeta, Error, ast::NestedMeta};
use proc_macro::TokenStream;
use proc_macro2::Ident;


#[derive(FromMeta)]
pub struct AutowiredArgs {
  #[darling(default)]
  pub clone: bool,
  #[darling(default, rename = "async_")]
  pub asyncness: bool,
  pub ctx: Ident,
}

impl AutowiredArgs {
  pub fn parse(args: TokenStream) -> Result<Self, Error> {
    let args = NestedMeta::parse_meta_list(args.into())?;
    let result = Self::from_list(&args)?;
    Ok(result)
  }
}