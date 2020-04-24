use std::collections::{HashSet};

#[macro_use] extern crate juniper;
use juniper::{FieldResult, Variables, EmptyMutation, LookAheadSelection, DefaultScalarValue};

struct Query;

struct Context;
impl juniper::Context for Context {}

struct Person {
    name: String,
    friends: Vec<Self>,
    best_friend: Box<Self>,
}

#[juniper::object]
impl Person {
    fn name(&self) -> &str {
        &self.name
    }

    fn friends(&self) -> &Vec<Self> {
        &self.friends
    }

    fn best_friend(&self) -> &Self {
        &self.best_friend
    }
}

struct PersonDTO {
    name: String,
    friends: Vec<Self>,
    best_friend: Box<Self>,
}

trait QueryConstructable {
    fn construct_query(selection: &LookAheadSelection<DefaultScalarValue>) -> String;
}


use juniper::LookAheadMethods;
impl QueryConstructable for Person {
    fn construct_query(selection: &LookAheadSelection<DefaultScalarValue>) -> String {
        let unique_names: HashSet<&str> = selection.child_names().into_iter().collect();
        let children_string = unique_names.iter().map(|field_name| {
            match *field_name {
                "name" => "name".to_owned(),
                "friends" => Person::construct_query(selection.select_child(field_name).unwrap()),
                "bestFriend" => Person::construct_query(selection.select_child(field_name).unwrap()),
                unknown_field => unreachable!("Unkown field {} on type Person, should be caught by juniper", unknown_field)
            }
        }).collect::<Vec<String>>()
        .join("\n");
        format!("{} {{\n{}\n}}", selection.field_name(), children_string)
    }
}
// 1. Construct query from selection
// 2. Perform query
// 3. Store query result as json
// 4. Resolve fields from result using context for current place, json

fn perform_query(query: &str) -> &str {
    unimplemented!()
}

fn parse_query_result(result: &str) -> PersonDTO {
    unimplemented!()
}

fn transform_dto(person_dto: &PersonDTO) -> Person {
    Person {
        name: person_dto.name.clone(),
        friends: person_dto.friends.iter().map( |p_dto| {
            transform_dto(p_dto)
        }).collect(),
        best_friend: Box::new(transform_dto(&person_dto.best_friend)),
    }
}

graphql_object!(Query: Context |&self| {
    field apiVersion() -> &str {
        "1.0"
    }

    field person (&executor) -> FieldResult<Person> {
        dbg!(executor.look_ahead());
        let query = Person::construct_query(&executor.look_ahead());
        println!("QUERY:\n{}", query);
        let result = perform_query(&query);
        let result = parse_query_result(result);
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
