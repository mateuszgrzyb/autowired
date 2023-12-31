use std::{collections::HashMap, future::IntoFuture};

use crate::{autowired_input::AutowiredInput, autowired_args::AutowiredArgs};
use darling::FromField;
use quote::{quote, format_ident, ToTokens};
use proc_macro2::TokenStream as TokenStream2;
use syn::{ItemStruct, ItemFn, Signature, ReturnType, Type, FnArg, FieldsNamed, Fields, FieldsUnnamed, Expr, Meta, parse::Parse, Attribute};

pub struct AutowiredData {
  pub args: AutowiredArgs,
  pub input: AutowiredInput,
  pub inject: HashMap<usize, Expr>,
}

fn deps_get() -> TokenStream2 {
  quote! { deps.get() }
}

fn detach_attrs(i: usize, attrs: &mut Vec<Attribute>) -> HashMap<usize, Expr> {
  let mut exprs = HashMap::new();
  
  attrs.retain(|a| {

    let Meta::List(a) = &a.meta else {
      return true
    };

    if !a.path.segments.last().is_some_and(|a| a.ident == "inject") {
      return true
    };

    let Some(e) = syn::parse::<Expr>(a.tokens.clone().into()).ok() else {
      return true
    };

    exprs.insert(i, e);

    false
  });

  exprs
}

impl AutowiredData {
  pub fn new(args: AutowiredArgs, mut input: AutowiredInput) -> Self {
    let mut inject = HashMap::new();
    
    match &mut input {
      AutowiredInput::Struct(s) => {
        for (i, f) in s.fields.iter_mut().enumerate() {
          inject.extend(detach_attrs(i, &mut f.attrs))
        }
      },
      AutowiredInput::Fn(f) | AutowiredInput::AsyncFn(f) => {
        for (i, f) in f.sig.inputs.iter_mut().enumerate().filter_map(|(i, a)| if let FnArg::Typed(a) = a { Some((i, a)) } else { None }) {
          inject.extend(detach_attrs(i, &mut f.attrs))
        }
      },
    }

    println!("{:?}", inject);

    Self {
      args, input, inject
    }
  }

  pub fn typename(&self) -> Result<TokenStream2, String> {
    match &self.input {
      AutowiredInput::Struct(ItemStruct { ident, .. }) => Ok(quote! { #ident }),
      AutowiredInput::Fn(ItemFn {
        sig: Signature {
          output: ReturnType::Type(_, t),
          ..
        },
        ..
      }) => Ok(quote! { #t }),
      AutowiredInput::AsyncFn(ItemFn {
        sig: Signature {
          output: ReturnType::Type(_, t),
          ..
        },
        ..
      }) => Ok(quote! { #t }),
      AutowiredInput::Fn(_) => Err("typename error".into()),
      AutowiredInput::AsyncFn(_) => Err("typename error".into()),
    }
  }

  pub fn children(&self) -> Vec<Type> {
    match &self.input {
      AutowiredInput::Struct(s) => s.fields.iter().map(|f| &f.ty).cloned().collect(),
      AutowiredInput::Fn(f) | AutowiredInput::AsyncFn(f) => f
        .sig
        .inputs
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

  pub fn children_names(&self) -> Vec<String> {
    self
      .children()
      .into_iter()
      .map(|c| quote! { #c }.to_string())
      .collect()
  }

  pub fn initializer_body(&self) -> TokenStream2 {
    let body = match &self.input {
      AutowiredInput::Struct(s) => {
        let ident = &s.ident;
        match &s.fields {
          Fields::Named(fields) => {
            let names = fields.named.iter().map(|f| f.ident.as_ref().unwrap());
            let fields = fields.named.iter().enumerate().map(|(i, _)| {
              self.inject.get(&i).map(|e| quote!{#e}).unwrap_or(deps_get())
            });
            quote! { Box::new(#ident { #(#names: #fields),* }) }
          },
          Fields::Unnamed(fields) => {
            let fields = fields.unnamed.iter().enumerate().map(|(i, _)| {
              self.inject.get(&i).map(|e| quote!{#e}).unwrap_or(deps_get())
            });
            quote! { Box::new(#ident(#(#fields),*)) }
          },
          Fields::Unit => quote! { Box::new(#ident) }
        }
      }
      AutowiredInput::Fn(f) | AutowiredInput::AsyncFn(f) => {
        let ident = &f.sig.ident;
        let args = f.sig.inputs.iter().map(|_| deps_get());
        quote! { #ident(#(#args),*) }
      }
    };

    if self.is_async() {
      let body = match self.input {
        AutowiredInput::Struct(_) | AutowiredInput::Fn(_) => quote!{ async { #body } },
        AutowiredInput::AsyncFn(_) => body,
      };
      quote!{
        async fn _init<
          //R: 'static, 
          R: Send + Sync + 'static, 
          F: ::autowired::Future<Output = R>,
        >(f: F) -> ::autowired::DependencyValue {
          Box::new(f.await)
        }

        Box::pin(_init(#body))
      }
    } else {
      quote!{
        Box::new(#body)
      }
    }
  }

  pub fn initializer_rt(&self) -> TokenStream2 {
    if self.is_async() {
      //quote! { ::autowired::Pin<Box<dyn ::autowired::Future<Output = ::autowired::DependencyValue>>> }
      quote! { ::autowired::Pin<Box<dyn ::autowired::Future<Output = ::autowired::DependencyValue> + Send + '_>> }
    } else {
      quote! { ::autowired::DependencyValue }
    }
  }

  pub fn dep_data_type(&self) -> TokenStream2 {
    if self.is_async() {
      quote! { ::autowired::ADepData }
    } else {
      quote! { ::autowired::DepData }
    }
  }

  pub fn typecheck_children(&self) -> Result<TokenStream2, String> {
    let ctx = &self.args.ctx;

    let children = self.children().into_iter().enumerate().filter(|(i, _)| !self.inject.contains_key(i)).map(|(_, e)| e).collect::<Vec<_>>();
    let children_idents = 
      children
      .iter()
      .enumerate()
      .map(|(i, _)| format_ident!("MSG_{}", i))
      .collect::<Vec<_>>();
    let children_msgs = children.iter().map(|t| quote!(#t).to_string().split_whitespace().collect::<String>() + ", ");

    let type_ = &self.typename()?;
    let name = uuid::Uuid::new_v4().as_simple().to_string();
    //let name = type_.to_string();

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
        // const CHILDREN_ASYNC: bool = #(::autowired::impls!(#children: ADep<#ctx>))||*;
        const CHILDREN_ASYNC: bool = #(::autowired::impls!(#children: ::autowired::AsyncAutowiredDep))||*;

        // match (CHILDREN_ASYNC, ::autowired::impls!(#type_: ::autowired::AsyncAutowiredDep<#ctx>)) {
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
        #(
        // const #children_idents: &'static str = if ::autowired::impls!(#children: Dep<#ctx>) { "" } else { #children_msgs };
        const #children_idents: &'static str = if ::autowired::impls!(#children: ::autowired::Dep<#ctx>) { "" } else { #children_msgs };
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

  pub fn impl_autowired(&self) -> Result<TokenStream2, String> {
    let ctx = &self.args.ctx;
    let type_ = self.typename()?;

    // let impl_dep = quote!{ impl ::autowired::Dep<#ctx> for #type_ {} };
    // let impl_sync = quote! { impl ::autowired::AutowiredDep<#ctx> for #type_ {} };
    // let impl_async = quote! { impl ::autowired::AsyncAutowiredDep<#ctx> for #type_ {} };
    let impl_dep = quote!{ impl ::autowired::Dep<#ctx> for #type_ {} };
    let impl_sync = quote! { impl ::autowired::AutowiredDep for #type_ {} };
    let impl_async = quote! { impl ::autowired::AsyncAutowiredDep for #type_ {} };

    let result = if self.is_async() {
      quote! {
        #impl_dep
        #impl_sync
        #impl_async
      }
    } else {
      quote! {
        #impl_dep
        #impl_sync
      }
    };

    Ok(result)
  }

  pub fn is_async(&self) -> bool {
    match (&self.args, &self.input) {
      (AutowiredArgs { asyncness, .. }, AutowiredInput::Struct(_)) => *asyncness,
      (AutowiredArgs { .. }, AutowiredInput::Fn(_)) => false,
      (AutowiredArgs { .. }, AutowiredInput::AsyncFn(_)) => true,
    }
  }
}

impl ToTokens for AutowiredData {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    match &self.input {
      AutowiredInput::Struct(s) => s.to_tokens(tokens),
      AutowiredInput::Fn(f) => f.to_tokens(tokens),
      AutowiredInput::AsyncFn(f) => f.to_tokens(tokens),
    }
  }
}