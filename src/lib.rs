pub use std::{
  any::{Any, TypeId},
  collections::HashMap,
  future::Future,
  hash::Hash,
  pin::Pin,
};

pub use const_format::{concatcp, formatcp};
pub use impls::impls;

pub use deps::Deps;
pub use linkme::distributed_slice;
pub use provider_builder::{Provider, ProviderBuilder};

pub use autowired_macros::autowired;
pub use inventory::submit;

mod deps;
mod graph_sorter;
mod provider;
mod provider_builder;

pub trait AutowiredDep {}
pub trait AsyncAutowiredDep: AutowiredDep {}

// pub type DependencyValue = Box<dyn Any>;
pub type DependencyValue = Box<dyn Any + Send + Sync>;
pub type DependencyMap = HashMap<TypeId, DependencyValue>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DepData {
  pub name: &'static str,
  pub children: &'static [&'static str],
  pub type_id: fn() -> TypeId,
  pub initializer: fn(&Deps) -> DependencyValue,
}

inventory::collect!(DepData);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ADepData {
  pub name: &'static str,
  pub children: &'static [&'static str],
  pub type_id: fn() -> TypeId,
  // pub initializer: fn(&Deps) -> Pin<Box<dyn Future<Output = Box<dyn Any>>>>,
  pub initializer: fn(&Deps) -> Pin<Box<dyn Future<Output = DependencyValue> + Send + Sync>>,
}

inventory::collect!(ADepData);

trait IDepData: Eq + Hash + Clone {
  fn name(&self) -> &'static str;
  fn children(&self) -> &'static [&'static str];
}

impl IDepData for DepData {
  fn name(&self) -> &'static str {
    self.name
  }

  fn children(&self) -> &'static [&'static str] {
    self.children
  }
}

impl IDepData for ADepData {
  fn name(&self) -> &'static str {
    self.name
  }

  fn children(&self) -> &'static [&'static str] {
    self.children
  }
}
