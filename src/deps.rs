use std::any::{type_name, TypeId};

use crate::DependencyMap;

#[derive(Default)]
// pub struct Deps(pub HashMap<TypeId, Box<dyn Any>>);
pub struct Deps(pub DependencyMap);

impl Deps {
  //pub fn insert<T: 'static>(&mut self, k: TypeId, t: T) {
  //    self.0.insert(k, Box::new(t));
  //}

  pub fn get<T: Clone + 'static>(&self) -> T {
    let t = TypeId::of::<T>();
    self
      .0
      .get(&t)
      .unwrap_or_else(|| panic!("get error: {}", type_name::<T>()))
      .downcast_ref::<T>()
      .unwrap_or_else(|| panic!("downcast error: {}", type_name::<T>()))
      .clone()
  }
}
