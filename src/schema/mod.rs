mod person;
use super::error::{
    DgraphQueryError, DB_QUERY_GENERATION_ERR, DB_QUERY_RESULT_PARSE_ERR, INTERNAL_ERROR,
    UNABLE_TO_RESOLVE_FIELD,
};
pub use person::Person;

use juniper::{EmptyMutation, FieldError, FieldResult};

fn perform_query(
    client: &dgraph::Dgraph,
    query: &str,
) -> Result<serde_json::Value, DgraphQueryError> {
    let response = client.new_readonly_txn().query(query)?;
    let response = String::from_utf8(response.json)?;
    let response = serde_json::from_str::<serde_json::Value>(&response)?;
    Ok(response)
}

pub struct Query;
pub struct Context {
    pub dgraph_client: dgraph::Dgraph,
}

impl juniper::Context for Context {}

#[juniper::object (Context = Context)]
impl Query {
    fn apiVersion() -> &str {
        "1.0"
    }

    fn person(context: &Context, executor: &Executor) -> FieldResult<Person> {
        let query = Person::generate_query(&executor.look_ahead());
        if let Err(err) = query {
            error!("{} {:?}", DB_QUERY_GENERATION_ERR, err);
            return Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: DB_QUERY_GENERATION_ERR }),
            ));
        }
        let query: String = query.unwrap();
        let query = format!(
            "{{\njames(func: anyofterms(name, \"James\")) {{\n{}\n}}\n}}",
            query
        );
        let result = perform_query(&context.dgraph_client, &query);
        match result {
            Ok(result) => Ok(Person {
                result_json: result["james"][0].clone(),
            }),
            Err(err) => {
                error!("{} {:?}", DB_QUERY_RESULT_PARSE_ERR, err);
                Err(FieldError::new(
                    UNABLE_TO_RESOLVE_FIELD,
                    graphql_value!({ INTERNAL_ERROR: DB_QUERY_RESULT_PARSE_ERR }),
                ))
            }
        }
    }
}

pub type Schema = juniper::RootNode<'static, Query, EmptyMutation<Context>>;