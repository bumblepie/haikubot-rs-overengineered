#[macro_use]
mod util;
mod discord_channel;
mod discord_server;
mod discord_user;
mod haiku;

use super::error::{
    DgraphQueryError, DB_QUERY_GENERATION_ERR, DB_QUERY_RESULT_ERR, DB_QUERY_RESULT_PARSE_ERR,
    INTERNAL_ERROR, INVALID_INPUT, UNABLE_TO_RESOLVE_FIELD,
};
use haiku::{is_valid_haiku_id, Haiku};
use juniper::{EmptyMutation, FieldError, FieldResult};
use std::collections::HashMap;
use util::MapsToDgraphQuery;

fn perform_query(
    client: &dgraph::Dgraph,
    query: &str,
    vars: HashMap<String, String>,
) -> Result<serde_json::Value, DgraphQueryError> {
    let response = client.new_readonly_txn().query_with_vars(query, vars)?;
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

    fn haiku(
        context: &Context,
        executor: &Executor,
        haiku_id: String,
    ) -> FieldResult<Option<Haiku>> {
        if !is_valid_haiku_id(&haiku_id) {
            return Err(FieldError::new(
                INVALID_INPUT,
                graphql_value!({ INVALID_INPUT: INVALID_INPUT }),
            ));
        }

        let query = Haiku::generate_inner_query(&executor.look_ahead());
        if let Err(err) = query {
            error!("{} {:?}", DB_QUERY_GENERATION_ERR, err);
            return Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: DB_QUERY_GENERATION_ERR }),
            ));
        }
        let query: String = query.unwrap();
        let query = format!(
            r#"
query haiku($id: string){{
    haiku(func: uid($id)) @filter(type(Haiku)) {{
        {}
    }}
}}"#,
            query
        );
        let mut vars = HashMap::new();
        vars.insert("$id".to_string(), haiku_id);
        let result = perform_query(&context.dgraph_client, dbg!(&query), vars);
        match dbg!(result) {
            Ok(result) => {
                if let Some(haikus) = result.get("haiku") {
                    if let Some(json) = haikus.get(0) {
                        return Ok(Some(Haiku::from(json.clone())));
                    } else {
                        return Ok(None);
                    }
                } else {
                    error!("{} - TODO err msg", DB_QUERY_RESULT_PARSE_ERR);
                }
            }
            Err(err) => error!("{} - {:?}", DB_QUERY_RESULT_ERR, err),
        };
        Err(FieldError::new(
            UNABLE_TO_RESOLVE_FIELD,
            graphql_value!({ INTERNAL_ERROR: DB_QUERY_RESULT_ERR }),
        ))
    }
}

pub type Schema = juniper::RootNode<'static, Query, EmptyMutation<Context>>;
