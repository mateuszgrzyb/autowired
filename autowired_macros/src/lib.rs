use autowired_data::AutowiredData;
use autowired_input::AutowiredInput;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{
  parse::Parse, ItemStruct, Token, Type, punctuated::Punctuated, token::Comma, bracketed, parse_macro_input,
};

mod autowired_input;
mod autowired_args;
mod autowired_data;

use autowired_args::AutowiredArgs;


#[proc_macro_attribute]
pub fn autowired(args: TokenStream, input: TokenStream) -> TokenStream {
  let args = match AutowiredArgs::parse(args) {
      Ok(args) => args,
      Err(e) => return e.write_errors().into(),
  };

  let i = parse_macro_input!(input as AutowiredInput);

  let input = AutowiredData::new(args, i);

  let type_ = input.typename().unwrap();
  let name = type_.to_string().split_whitespace().collect::<String>();
  let ident_name = uuid::Uuid::new_v4().as_simple().to_string();
  
  let children = input.children_names();

  let type_id_name = format_ident!("_AUTOWIRED_{}_type_id", ident_name);

  let initializer_name = format_ident!("_AUTOWIRED_{}_initializer", ident_name);
  let initializer_body = input.initializer_body();
  let initializer_rt = input.initializer_rt();

  let dep_data_type = input.dep_data_type();

  let typecheck_children = input.typecheck_children().unwrap();

  let impl_autowired = input.impl_autowired().unwrap();
  let impl_clone = if input.args.clone {
    quote!(#[derive(Clone)])
  } else {
    quote!()
  };

  quote! {
    #impl_clone
    #input

    #impl_autowired

    #[allow(non_upper_case_globals)]
    fn #type_id_name() -> ::autowired::TypeId {
      ::autowired::TypeId::of::<#type_>()
    }

    #[allow(non_upper_case_globals)]
    fn #initializer_name(deps: &::autowired::Deps) -> #initializer_rt {
      #initializer_body
    }

    ::autowired::submit! {
      #dep_data_type {
        name: #name,
        children: &[#(#children),*],
        type_id: #type_id_name,
        initializer: #initializer_name,
      }
    }

    #typecheck_children

  }
  .into()
}

struct ProviderInput {
  p: Ident,
  name: Ident,
  types: Punctuated<Type, Comma>,
}

impl Parse for ProviderInput {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let p = input.parse()?;
    input.parse::<Token![~]>()?;
    let name = input.parse()?;
    
    let content;
    bracketed!(content in input);
    
    let types = content.parse_terminated(Type::parse, Comma)?;
    Ok(Self { p, name, types })
  }
}

#[proc_macro_derive(Context)]
pub fn derive_context(input: TokenStream) -> TokenStream {
  let ItemStruct { ident, fields, .. } = parse_macro_input!(input);
  let types = fields.iter().map(|f| &f.ty).collect::<Vec<_>>();
  let names = fields.iter().enumerate().map(|(i, f)| match &f.ident {
    Some(i) => quote!(#i),
    None => quote!(#i),
  });

  quote!{
    #[::autowired::async_trait]
    impl ::autowired::Context for #ident {
      fn get_initial_deps(&self) -> ::autowired::Deps {
        let mut deps: ::autowired::DependencyMap = Default::default();
        #(
          {
            let t = ::autowired::TypeId::of::<#types>();
            deps.insert(t, Box::new(self.#names.clone()) as ::autowired::DependencyValue);
          }
        )*
        ::autowired::Deps(deps)
      }

    }
    
    #(
    // impl ::autowired::Dep<#ident> for #types {}
    // impl ::autowired::SharedDep<#ident> for #types {}
    impl ::autowired::Dep<#ident> for #types {}
    impl ::autowired::SharedDep<#ident> for #types {}
    )*
  }.into()
}