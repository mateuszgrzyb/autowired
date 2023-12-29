use std::{any::TypeId, collections::HashMap, marker::PhantomData};

use frunk::{
  hlist::{HFoldLeftable, HList},
  Func, Poly,
};

use crate::{graph_sorter::GraphSorter, ADepData, DepData, DependencyMap, DependencyValue, Deps};

pub struct ProviderBuilder<HL: HList> {
  deps: Deps,
  _pd: PhantomData<HL>,
}

pub struct F;

impl<A> Func<(DependencyMap, A)> for F
where
  // A: 'static
  A: Send + Sync + 'static,
{
  type Output = DependencyMap;

  #[inline]
  fn call((mut deps, a): (HashMap<TypeId, DependencyValue>, A)) -> Self::Output {
    let t = TypeId::of::<A>();
    deps.insert(t, Box::new(a) as DependencyValue);
    deps
  }
}

impl<HL> ProviderBuilder<HL>
where
  HL: HList + HFoldLeftable<Poly<F>, DependencyMap, Output = DependencyMap>,
{
  pub fn new(lh: HL) -> Self {
    let deps = Deps(lh.foldl(Poly(F), HashMap::new()));

    Self {
      deps,
      _pd: PhantomData,
    }
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

  fn _get_provider(self) -> Provider<HL> {
    Provider {
      deps: self.deps,
      _pd: PhantomData,
    }
  }

  pub fn build(mut self) -> Provider<HL> {
    self._build_sync();
    self._get_provider()
  }

  pub async fn async_build(mut self) -> Provider<HL> {
    self._build_sync();
    self._build_async().await;
    self._get_provider()
  }
}

pub struct Provider<HL: HList> {
  deps: Deps,
  _pd: PhantomData<HL>,
}

impl<HL: HList> Provider<HL> {
  pub fn provide<T: Clone + 'static>(&self) -> T {
    self.deps.get()
  }
}
