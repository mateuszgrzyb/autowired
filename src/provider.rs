use std::marker::PhantomData;

use async_trait::async_trait;

use crate::{Dep, Deps, DepsBuilder};

#[async_trait]
pub trait Context {
  fn get_initial_deps(&self) -> Deps;

  fn get_provider(&self) -> Provider<Self> {
    let builder = DepsBuilder::new(self.get_initial_deps());
    let deps = builder.build();
    Provider {
      deps,
      _pd: PhantomData,
    }
  }

  async fn get_async_provider(&self) -> Provider<Self> {
    let builder = DepsBuilder::new(self.get_initial_deps());
    let deps = builder.async_build().await;
    Provider {
      deps,
      _pd: PhantomData,
    }
  }
}

pub struct Provider<C: Context + ?Sized> {
  deps: Deps,
  _pd: PhantomData<C>,
}

impl<C: Context> Provider<C> {
  pub fn provide<T: Clone + 'static>(&self) -> T {
    self.deps.get()
  }
}
