use std::any::{type_name, TypeId};

use crate::DependencyMap;

#[derive(Default)]
pub struct Deps(pub DependencyMap);

impl Deps {
  pub fn get<T: Clone + 'static>(&self) -> T {
    let t = TypeId::of::<T>();

    let v = self
      .0
      .get(&t)
      .unwrap_or_else(|| panic!("get error: {}", type_name::<T>()));

    if let Some(v) = v.downcast_ref::<T>() {
      v.clone()
    } else if let Some(v) = v.downcast_ref::<Box<T>>() {
      *v.clone()
    } else {
      panic!("downcast error: {}", type_name::<T>())
    }
  }
}
