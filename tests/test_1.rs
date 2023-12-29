use std::{
  any::{type_name, Any},
  rc::Rc,
  sync::Arc,
};

use autowired::{HashMap, TypeId};
use frunk::hlist;
use frunk::{
  hlist::{HFoldLeftable, HList},
  poly_fn, Func, Poly,
};

#[derive(Debug, Clone)]
#[::autowired::autowired]
struct A {
  b: B,
  c: C,
  #[shared]
  counter: Arc<usize>,
}

//#[::autowired::autowired]
//async fn a(b: B, c: C) -> A {
//  //fn a(b: B, c: C) -> A {
//  A { b, c }
//}

#[derive(Debug, Clone)]
struct B {
  c: C,
}

#[::autowired::autowired]
fn b(c: C) -> B {
  B { c }
}

#[derive(Debug, Clone)]
struct C {}

#[::autowired::autowired]
fn c() -> C {
  C {}
}

#[derive(Debug, Clone)]
struct AsyncParent {
  a: A,
}

//impl ::autowired::AutowiredDep for AsyncParent {}

#[::autowired::autowired]
async fn async_parent(a: A) -> AsyncParent {
  AsyncParent { a }
}

/*
use ::autowired::*;


fn initializer(a: A) -> Pin<Box<dyn Future<Output = Box<dyn Any>>>> {
  async fn _init<R: 'static, F: Future<Output = R>>(f: F) -> Box<dyn Any> {
    let r = f.await;
    Box::new(r)
  }

  Box::pin(_init(async_parent(a)))
}
 */

/*
#[derive(Clone)]
struct A {
  b: B,
  c: C,
  counter: Rc<usize>,
}

#[derive(Clone)]
struct B {
  c: C,
}

#[derive(Clone)]
struct C {

}

#[derive(Clone)]
#[::autowired::autowired]
struct D {
  a: A,
}

fn a_type_id() -> ::autowired::TypeId {
  ::autowired::TypeId::of::<A>()
}

fn initialize_a(deps: &::autowired::Deps) -> Box<dyn ::autowired::Any> {
  let a = A {
    b: deps.get(),
    c: deps.get(),
    counter: deps.get(),
  };

  Box::new(a)
}


::autowired::submit! {
  ::autowired::DepData {
    name: "A",
    children: &["B", "C"],
    type_id: a_type_id,
    initializer: initialize_a,
  }
}

fn create_b(c: C) -> B {
  B {
    c
  }
}

fn initialize_b(deps: &::autowired::Deps) -> Box<dyn ::autowired::Any> {
  let b = create_b(deps.get());

  Box::new(b)
}

fn b_type_id() -> ::autowired::TypeId {
  ::autowired::TypeId::of::<B>()
}

::autowired::submit! {
  ::autowired::DepData {
    name: "B",
    children: &["C"],
    type_id: b_type_id,
    initializer: initialize_b,
  }
}

fn initialize_c(deps: &::autowired::Deps) -> Box<dyn ::autowired::Any> {
  let c = C {
  };

  Box::new(c)
}

fn c_type_id() -> ::autowired::TypeId {
  ::autowired::TypeId::of::<C>()
}

::autowired::submit! {
  ::autowired::DepData {
    name: "C",
    children: &[],
    type_id: c_type_id,
    initializer: initialize_c,
  }
}

 */

struct Context {
  counter: Rc<usize>,
}

#[test]
fn test_1() {
  let p = ::autowired::ProviderBuilder::new(hlist![Arc::new(0usize)]).build();

  //let parent = p.provide::<AsyncParent>();
  let a = p.provide::<A>();

  println!("{:?}", a);
}

struct F;

impl<A: 'static> Func<(HashMap<TypeId, Box<dyn Any>>, A)> for F {
  type Output = HashMap<TypeId, Box<dyn Any>>;

  #[inline]
  fn call((mut deps, a): (HashMap<TypeId, Box<dyn Any>>, A)) -> Self::Output {
    let t = TypeId::of::<A>();
    deps.insert(t, Box::new(a) as Box<dyn Any>);
    deps
  }
}

fn vs<HL>(hl: HL)
where
  HL: HList
    + HFoldLeftable<Poly<F>, HashMap<TypeId, Box<dyn Any>>, Output = HashMap<TypeId, Box<dyn Any>>>,
{
  let a = hl.foldl(Poly(F), HashMap::new());
}

#[test]
fn test_hlist() {
  let h = hlist!["ala", 1, true];
  vs(h);
  let values = h.foldl(Poly(F), HashMap::new());

  println!("{:?}", values);
}

#[tokio::test]
async fn test_2() {
  let p = ::autowired::ProviderBuilder::new(hlist![Arc::new(0usize)])
    .async_build()
    .await;

  let parent = p.provide::<AsyncParent>();
  //let a = p.provide::<A>();

  println!("{:?}", parent);
}
