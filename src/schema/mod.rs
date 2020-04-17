mod discord_channel;
mod discord_server;
mod discord_user;
mod haiku;
mod util;

use super::error::{
    DgraphQueryError, DB_QUERY_GENERATION_ERR, DB_QUERY_RESULT_ERR, INTERNAL_ERROR,
    UNABLE_TO_RESOLVE_FIELD,
};
use haiku::Haiku;
use juniper::{EmptyMutation, FieldError, FieldResult};
use util::MapsToDgraphQuery;

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

    fn haiku(context: &Context, executor: &Executor) -> FieldResult<Haiku> {
        let query = Haiku::generate_inner_query(&executor.look_ahead());
        if let Err(err) = query {
            error!("{} {:?}", DB_QUERY_GENERATION_ERR, err);
            return Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: DB_QUERY_GENERATION_ERR }),
            ));
        }
        let query: String = query.unwrap();
        let query = format!("{{\nhaiku(func: type(Haiku)) {{\n{}\n}}\n}}", query);
        let result = perform_query(&context.dgraph_client, dbg!(&query));
        match dbg!(result) {
            Ok(result) => Ok(Haiku {
                result_json: result["haiku"][0].clone(),
            }),
            Err(err) => {
                error!("{} - {:?}", DB_QUERY_RESULT_ERR, err);
                Err(FieldError::new(
                    UNABLE_TO_RESOLVE_FIELD,
                    graphql_value!({ INTERNAL_ERROR: DB_QUERY_RESULT_ERR }),
                ))
            }
        }
    }
}

pub type Schema = juniper::RootNode<'static, Query, EmptyMutation<Context>>;
