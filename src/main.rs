mod error;
mod schema;

#[macro_use]
extern crate juniper;
use juniper::{EmptyMutation, Variables};
#[macro_use]
extern crate dgraph;
extern crate serde_json;

use schema::{Query, Schema};

fn main() {
    println!("Hello, world!");
    let (res, errors) = juniper::execute(
        "query { apiVersion, person { bestFriend { name friends { name } } } }",
        None,
        &Schema::new(Query, EmptyMutation::new()),
        &Variables::new(),
        &(),
    )
    .unwrap();
    dbg!(res);
    dbg!(errors);
}
