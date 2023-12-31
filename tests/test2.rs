use ::autowired::*;
use std::sync::Arc;

fn stateful() -> usize {
  println!("stateful call...");
  0
}

#[autowired(ctx = Ctx, clone)]
struct A {
  provided: AutowiredString,
  shared: Arc<usize>,
  #[inject(stateful())]
  injected: usize,
}

#[derive(Clone)]
struct AutowiredString(Arc<String>);

#[autowired(ctx = Ctx)]
fn provide_string() -> AutowiredString {
  AutowiredString(Arc::new("".into()))
}

#[derive(Context)]
struct Ctx {
  shared: Arc<usize>,
}

#[test]
fn test1() {
  let p = Ctx {
    shared: Arc::new(0),
  }
  .get_provider();

  let a = p.provide::<A>();
  let a2 = p.provide::<A>();

  println!("{}", a.injected);
  assert!(false);
}
