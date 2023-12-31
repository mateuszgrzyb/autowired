use crate::{graph_sorter::GraphSorter, ADepData, DepData, Deps};

pub struct DepsBuilder {
  deps: Deps,
}

impl DepsBuilder {
  pub fn new(deps: Deps) -> Self {
    Self { deps }
  }

  fn _build_sync(&mut self) {
    let dep_data = inventory::iter::<DepData>
      .into_iter()
      .cloned()
      .collect::<Vec<_>>();

    let dep_data = GraphSorter::sort(dep_data);

    for dep in dep_data {
      let dep_type = (dep.type_id)();
      let initialized_dep = (dep.initializer)(&self.deps);
      self.deps.0.insert(dep_type, initialized_dep);
    }
  }

  async fn _build_async(&mut self) {
    let dep_data = inventory::iter::<ADepData>
      .into_iter()
      .cloned()
      .collect::<Vec<_>>();

    let dep_data = GraphSorter::sort(dep_data);

    for dep in dep_data {
      let dep_type = (dep.type_id)();
      let initialized_dep = (dep.initializer)(&self.deps).await;
      self.deps.0.insert(dep_type, initialized_dep);
    }
  }

  fn _get_deps(self) -> Deps {
    self.deps
  }

  pub fn build(mut self) -> Deps {
    self._build_sync();
    self._get_deps()
  }

  pub async fn async_build(mut self) -> Deps {
    self._build_sync();
    self._build_async().await;
    self._get_deps()
  }
}

// pub struct Provider<HL: HList> {
//   deps: Deps,
//   _pd: PhantomData<HL>,
// }
//
// impl<HL: HList> Provider<HL> {
//   pub fn provide<T: Clone + 'static>(&self) -> T {
//     self.deps.get()
//   }
// }
