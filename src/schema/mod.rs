mod person;
use super::error::*;
pub use person::Person;

use juniper::{EmptyMutation, FieldError, FieldResult};

fn perform_query(query: &str) -> String {
    let client = make_dgraph!(dgraph::new_dgraph_client("localhost:9080"));
    let response = client.new_readonly_txn().query(query).unwrap();
    String::from_utf8(response.json).unwrap()
}

pub struct Query;

#[juniper::object]
impl Query {
    fn apiVersion() -> &str {
        "1.0"
    }

    fn person(executor: &Executor) -> FieldResult<Person> {
        let query = Person::generate_query(&executor.look_ahead());
        if query.is_err() {
            dbg!(error!(
                "{} {:?}",
                DB_QUERY_GENERATION_ERR,
                query.unwrap_err()
            ));
            return Err(FieldError::new(
                "Unable to generate query",
                graphql_value!({ INTERNAL_ERROR: DB_QUERY_GENERATION_ERR }),
            ));
        }
        let query: String = query.ok().unwrap();
        let query = format!(
            "{{\njames(func: anyofterms(name, \"James\")) {{\n{}\n}}\n}}",
            query
        );
        let result = perform_query(&query);
        Ok(Person {
            result_json: serde_json::from_str::<serde_json::Value>(&result).unwrap()["james"][0]
                .clone(),
        })
    }
}

pub type Schema = juniper::RootNode<'static, Query, EmptyMutation<()>>;
