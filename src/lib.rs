use std::{any::Any, collections::HashMap, hash::Hash};

pub use std::{any::TypeId, future::Future, pin::Pin};

pub use crate::deps::Deps;
use crate::deps_builder::DepsBuilder;
pub use crate::provider::{Context, Provider};

pub use async_trait::async_trait;
pub use autowired_macros::{autowired, Context};
pub use const_format::{concatcp, formatcp};
pub use impls::impls;
pub use inventory::submit;

mod deps;
mod deps_builder;
mod graph_sorter;
mod provider;

pub trait Dep<T>: Clone {}
pub trait SharedDep<T>: Dep<T> {}
pub trait AutowiredDep: Clone {}
pub trait AsyncAutowiredDep: Clone {}
/*
pub trait Dep<T>: Clone {}
pub trait SharedDep<T>: Dep<T> {}
pub trait AutowiredDep<T>: Dep<T> {}
pub trait AsyncAutowiredDep<T>: AutowiredDep<T> {}
 */

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
  pub initializer: fn(&Deps) -> Pin<Box<dyn Future<Output = DependencyValue> + Send + '_>>,
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
