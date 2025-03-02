//! Binary to test panics

fn foo() {
  bar()
}

fn bar() {
  panic!("I just couldn't anymore..");
}

fn main() {
  println!("Hello i will panic");
  foo();
}
