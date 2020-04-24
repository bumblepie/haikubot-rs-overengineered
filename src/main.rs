use std::collections::{HashSet};

#[macro_use] extern crate juniper;
use juniper::{FieldResult, Variables, EmptyMutation, LookAheadSelection, DefaultScalarValue};

#[macro_use] extern crate dgraph;

struct Query;

struct Context;
impl juniper::Context for Context {}

struct Person;

#[juniper::object (Context = Context)]
impl Person {
    fn name(context: &Context) -> &str {
        &self.name
    }

    fn friends(&self) -> &Vec<Self> {
        &self.friends
    }

    fn best_friend(&self) -> &Self {
        &self.best_friend
    }
}

trait QueryConstructable {
    fn construct_query(selection: &LookAheadSelection<DefaultScalarValue>) -> String;
}


use juniper::LookAheadMethods;
// write out inner query
impl QueryConstructable for Person {
    fn construct_query(selection: &LookAheadSelection<DefaultScalarValue>) -> String {
        let unique_names: HashSet<&str> = selection.child_names().into_iter().collect();
        unique_names.iter().map(|field_name| {
            match *field_name {
                "name" => "name".to_owned(),
                "friends" => format!("friend {{\n{}\n}}", Person::construct_query(selection.select_child(field_name).unwrap())),
                "bestFriend" => format!("bestFriend: friend @facets(orderdesc: score) (first: 1) {{\n{}\n}}", Person::construct_query(selection.select_child(field_name).unwrap())),
                unknown_field => unreachable!("Unkown field {} on type Person, should be caught by juniper", unknown_field)
            }
        }).collect::<Vec<String>>()
        .join("\n")
    }
}
// 1. Construct query from selection
// 2. Perform query
// 3. Store query result as json
// 4. Resolve fields from result using context for current place, json

fn perform_query(query: &str) -> String {
    let client = make_dgraph!(dgraph::new_dgraph_client("localhost:9080"));
    let response = client.new_readonly_txn().query(query).unwrap();
    String::from_utf8(response.json).unwrap()
}

graphql_object!(Query: Context |&self| {
    field apiVersion() -> &str {
        "1.0"
    }

    field person (&executor) -> FieldResult<Person> {
        dbg!(executor.look_ahead());
        let query = Person::construct_query(&executor.look_ahead());
        let query = format!("{{\ndave(func: anyofterms(name, \"Dave\")) {{\n{}\n}}\n}}", query);
        println!("QUERY:\n{}", query);
        let result = perform_query(&query);
        dbg!(&result);
        let result = parse_query_result(&result);
        Ok(transform_dto(&result))
    }
});

type Schema = juniper::RootNode<'static, Query, EmptyMutation<Context>>;

fn main() {
    println!("Hello, world!");
    let ctx = Context;
    let (res, _errors) = juniper::execute(
        "query { apiVersion, person { name friends { name } bestFriend { name bestFriend { name } } } }",
        None,
        &Schema::new(Query, EmptyMutation::new()),
        &Variables::new(),
        &ctx,
    ).unwrap();
    dbg!(res);
}
