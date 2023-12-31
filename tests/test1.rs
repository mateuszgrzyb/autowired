use std::sync::Arc;

use autowired::{Context, TypeId};

#[derive(Debug)]
#[::autowired::autowired(ctx = ProviderContext, clone)]
struct A {
  b: B,
  c: C,
}

#[derive(Debug, Clone)]
struct B {
  c: C,
}

#[::autowired::autowired(ctx = ProviderContext)]
fn b(c: C) -> B {
  B { c }
}

#[derive(Debug, Clone)]
struct C {}

#[::autowired::autowired(ctx = ProviderContext)]
fn c() -> C {
  C {}
}

#[derive(Debug, Clone)]
struct AsyncParent {
  a: A,
}

#[::autowired::autowired(ctx = ProviderContext)]
async fn async_parent(a: A) -> AsyncParent {
  AsyncParent { a }
}

#[derive(Context)]
struct ProviderContext {
  //counter: Arc<usize>,
}

#[test]
fn test_1() {
  let p = ProviderContext {
    //counter: Arc::new(0),
  }
  .get_provider();

  //let parent = p.provide::<AsyncParent>();
  let a = p.provide::<A>();

  println!("{:?}", a);
}

#[tokio::test]
async fn test_2() {
  let p = ProviderContext {
    //counter: Arc::new(0),
  }
  .get_async_provider()
  .await;

  let parent = p.provide::<AsyncParent>();
  let a = p.provide::<A>();

  println!("{:?}", parent);
}
