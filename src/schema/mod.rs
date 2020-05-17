#[macro_use]
mod util;
mod discord_channel;
mod discord_server;
mod discord_user;
mod haiku;

use super::error::{internal_error, DgraphQueryError};
use haiku::{valid_haiku_id, Haiku};
use juniper::{EmptyMutation, FieldResult};
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
        let haiku_id = valid_haiku_id(haiku_id)?;
        let query = Haiku::generate_inner_query(&executor.look_ahead())?;
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
                    error!("Error parsing Dgraph Query result - malformed response");
                }
            }
            Err(err) => error!("Dgraph error - {:?}", err),
        };
        Err(internal_error())
    }
}

pub type Schema = juniper::RootNode<'static, Query, EmptyMutation<Context>>;
