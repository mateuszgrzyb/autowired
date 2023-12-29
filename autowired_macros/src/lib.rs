use std::collections::HashSet;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::{
  parse::Parse, parse_macro_input, Attribute, Fields, FieldsNamed, FieldsUnnamed, FnArg, ItemFn,
  ItemStruct, ReturnType, Signature, Token, Type, Visibility,
};

type Shared = HashSet<Type>;

enum AutowiredInput {
  Struct(ItemStruct),
  Fn(ItemFn),
  AsyncFn(ItemFn),
}

impl Parse for AutowiredInput {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
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

impl ToTokens for AutowiredInput {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    match self {
      Self::Struct(s) => s.to_tokens(tokens),
      Self::Fn(f) => f.to_tokens(tokens),
      Self::AsyncFn(f) => f.to_tokens(tokens),
    }
  }
}

fn deps_get() -> TokenStream2 {
  quote! { deps.get() }
}

impl AutowiredInput {
  fn get_shared(&mut self) -> Shared {
    fn is_shared(a: &Attribute) -> bool {
      let syn::Meta::Path(syn::Path { segments, .. }) = &a.meta else {
        return false;
      };

      let Some(last) = segments.last() else {
        return false;
      };

      last.ident.eq("shared")
    }

    match self {
      Self::Struct(ItemStruct { fields, .. }) => {
        let mut r = HashSet::new();

        for f in fields {
          let l1 = f.attrs.len();
          f.attrs.retain(|a| !is_shared(a));
          let l2 = f.attrs.len();
          if l1 != l2 {
            r.insert(f.ty.clone());
          }
        }

        r
      }
      Self::Fn(ItemFn {
        sig: Signature { inputs, .. },
        ..
      })
      | Self::AsyncFn(ItemFn {
        sig: Signature { inputs, .. },
        ..
      }) => {
        let mut r = HashSet::new();

        for f in inputs.iter_mut().filter_map(|i| {
          if let FnArg::Typed(i) = i {
            Some(i)
          } else {
            None
          }
        }) {
          let l1 = f.attrs.len();
          f.attrs.retain(|a| !is_shared(a));
          let l2 = f.attrs.len();
          if l1 != l2 {
            r.insert(f.ty.as_ref().clone());
          }
        }

        r
      }
    }
  }

  fn typename(&self) -> Result<TokenStream2, String> {
    match self {
      Self::Struct(ItemStruct { ident, .. }) => Ok(quote! { #ident }),
      Self::Fn(ItemFn {
        sig: Signature {
          output: ReturnType::Type(_, t),
          ..
        },
        ..
      }) => Ok(quote! { #t }),
      Self::AsyncFn(ItemFn {
        sig: Signature {
          output: ReturnType::Type(_, t),
          ..
        },
        ..
      }) => Ok(quote! { #t }),
      Self::Fn(_) => Err("typename error".into()),
      Self::AsyncFn(_) => Err("typename error".into()),
    }
  }

  fn children(&self) -> Vec<Type> {
    match self {
      Self::Struct(ItemStruct { fields, .. }) => fields.iter().map(|f| &f.ty).cloned().collect(),
      Self::Fn(ItemFn {
        sig: Signature { inputs, .. },
        ..
      })
      | Self::AsyncFn(ItemFn {
        sig: Signature { inputs, .. },
        ..
      }) => inputs
        .iter()
        .filter_map(|i| {
          if let FnArg::Typed(ti) = i {
            Some(ti)
          } else {
            None
          }
        })
        .map(|ti| &*ti.ty)
        .cloned()
        .collect(),
    }
  }

  fn children_names(&self) -> Vec<String> {
    self
      .children()
      .into_iter()
      .map(|c| quote! { #c }.to_string())
      .collect()
  }

  fn initializer_body(&self) -> TokenStream2 {
    match self {
      Self::Struct(ItemStruct {
        ident,
        fields: Fields::Named(FieldsNamed { named, .. }),
        ..
      }) => {
        let names = named.iter().map(|f| f.ident.as_ref().unwrap());
        let fields = named.iter().map(|_| deps_get());
        quote! {
          Box::new(
            #ident {
              #(#names: #fields),*
            }
          )
        }
      }
      Self::Struct(ItemStruct {
        ident,
        fields: Fields::Unnamed(FieldsUnnamed { unnamed, .. }),
        ..
      }) => {
        let fields = unnamed.iter().map(|_| deps_get());
        quote! {
          Box::new(
            #ident (
              #(#fields),*
            )
          )
        }
      }
      Self::Struct(ItemStruct {
        ident,
        fields: Fields::Unit,
        ..
      }) => {
        quote! {
          Box::new(
            #ident
          )
        }
      }
      Self::Fn(ItemFn {
        sig: Signature { ident, inputs, .. },
        ..
      }) => {
        let args = inputs.iter().map(|_| deps_get());
        quote! {
          Box::new(
            #ident(
              #(#args),*
            )
          )
        }
      }
      Self::AsyncFn(ItemFn {
        sig: Signature { ident, inputs, .. },
        ..
      }) => {
        let args = inputs.iter().map(|_| deps_get());
        quote! {

          async fn _init<
            //R: 'static, 
            R: Send + Sync + 'static, 
            F: ::autowired::Future<Output = R>,
          >(f: F) -> ::autowired::DependencyValue {
            let r = f.await;
            Box::new(r)
          }

          Box::pin(
            _init(
              #ident(
                #(#args),*
              )
            )
          )
        }
      }
    }
  }

  fn initializer_rt(&self) -> TokenStream2 {
    if self.is_async() {
      //quote! { ::autowired::Pin<Box<dyn ::autowired::Future<Output = ::autowired::DependencyValue>>> }
      quote! { ::autowired::Pin<Box<dyn ::autowired::Future<Output = ::autowired::DependencyValue> + Send + Sync>> }
    } else {
      quote! { ::autowired::DependencyValue }
    }
  }

  fn dep_data_type(&self) -> TokenStream2 {
    if self.is_async() {
      quote! { ::autowired::ADepData }
    } else {
      quote! { ::autowired::DepData }
    }
  }

  fn typecheck_children(&self, shared: &Shared) -> Result<TokenStream2, String> {
    let children = self.children().into_iter().filter(|c| !shared.contains(c)).collect::<Vec<_>>();
    let children_idents = 
      children
      .iter()
      .enumerate()
      .map(|(i, _)| format_ident!("MSG_{}", i))
      .collect::<Vec<_>>();
    let children_msgs = children_idents.iter().map(|n| n.to_string() + ", ");

    let type_ = &self.typename()?;
    let name = type_.to_string();

    let check_ident = format_ident!("__AUTOWIRED_{}_check", name);
    let r_ident = format_ident!("__AUTOWIRED_{}_R", name);
    let check_async_ident = format_ident!("__AUTOWIRED_{}_check_async", name);
    let r_async_ident = format_ident!("__AUTOWIRED_{}_R_async", name);

    if children.is_empty() {
      return Ok(quote!());
    }

    let result = quote! {
      #[allow(non_upper_case_globals)]
      const fn #check_async_ident() -> (bool, &'static str) {
        use ::autowired::AsyncAutowiredDep as ADep;
        const CHILDREN_ASYNC: bool = #(::autowired::impls!(#children: ADep))||*;

        match (CHILDREN_ASYNC, ::autowired::impls!(#type_: ::autowired::AsyncAutowiredDep)) {
          (true, false) => (false, "Asyncness error"),
          _ => (true, "")
        }
      }

      #[allow(non_upper_case_globals)]
      const #r_async_ident: (bool, &'static str) = #check_async_ident();
      const _: () = assert!(#r_async_ident.0, "{}", #r_async_ident.1);

      #[allow(non_upper_case_globals)]
      const fn #check_ident() -> (bool, &'static str) {
        use ::autowired::AutowiredDep as Dep;

        #(
          const #children_idents: &'static str = if ::autowired::impls!(#children: Dep) { "" } else { #children_msgs };
        )*

        const MSG: &'static str = ::autowired::concatcp!(#(#children_idents),*);
        /*
        const MSG: &'static str = match str_get!(MSG_, ..MSG_.len() - 2) {
          Some(s) => s,
          None => MSG_,
        };
         */

        (MSG.is_empty(), ::autowired::formatcp!("Types [{}] cannot be autowired", MSG))
      }

      #[allow(non_upper_case_globals)]
      const #r_ident: (bool, &'static str) = #check_ident();
      const _: () = assert!(#r_ident.0, "{}", #r_ident.1);
    };

    Ok(result)
  }

  fn impl_autowired(&self) -> Result<TokenStream2, String> {
    let type_ = self.typename()?;

    let impl_sync = quote! { impl ::autowired::AutowiredDep for #type_ {} };
    let impl_async = quote! { impl ::autowired::AsyncAutowiredDep for #type_ {} };

    let result = if self.is_async() {
      quote! {
        #impl_sync
        #impl_async
      }
    } else {
      quote! {
        #impl_sync
      }
    };

    Ok(result)
  }

  fn is_async(&self) -> bool {
    match self {
      AutowiredInput::Struct(_) | AutowiredInput::Fn(_) => false,
      AutowiredInput::AsyncFn(_) => true,
    }
  }
}

#[proc_macro_attribute]
pub fn autowired(args: TokenStream, input: TokenStream) -> TokenStream {
  let mut input = parse_macro_input!(input as AutowiredInput);

  let type_ = input.typename().unwrap();
  let name = type_.to_string();
  
  let shared = input.get_shared();
  let shared_ids = shared.iter().enumerate().map(|(i, _)| format_ident!("__AUTOWIRED_{}_{}_shared", name, i));
  let shared_strs = shared.iter().map(|t| quote!(#t).to_string());
  
  let children = input.children_names();

  let type_id_name = format_ident!("_AUTOWIRED_{}_type_id", name);

  let initializer_name = format_ident!("_AUTOWIRED_{}_initializer", name);
  let initializer_body = input.initializer_body();
  let initializer_rt = input.initializer_rt();

  let dep_data_type = input.dep_data_type();

  let typecheck_children = input.typecheck_children(&shared).unwrap();

  let impl_autowired = input.impl_autowired().unwrap();

  quote! {
    #input

    #impl_autowired

    /*
    #(
      #[::autowired::distributed_slice(::autowired::SHARED)]
      static #shared_ids: &'static str = #shared_strs;
    )*
     */

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
