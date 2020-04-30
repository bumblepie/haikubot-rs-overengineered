use super::super::error::{DB_QUERY_RESULT_PARSE_ERR, INTERNAL_ERROR, UNABLE_TO_RESOLVE_FIELD};
use juniper::{FieldError, FieldResult};

use super::haiku::Haiku;

#[derive(Debug)]
pub struct DiscordUser {
    pub result_json: serde_json::Value,
}

impl From<serde_json::Value> for DiscordUser {
    fn from(result_json: serde_json::Value) -> DiscordUser {
        DiscordUser { result_json }
    }
}

#[juniper::object]
impl DiscordUser {
    fn discordSnowflake(&self) -> FieldResult<String> {
        match self.result_json.get("discordSnowflake") {
            Some(serde_json::Value::String(id)) => Ok(id.clone()),
            _ => Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: DB_QUERY_RESULT_PARSE_ERR }),
            )),
        }
    }

    fn haikus(&self) -> FieldResult<Vec<Haiku>> {
        match self.result_json.get("haikus") {
            Some(serde_json::Value::Array(haikus)) => Ok(haikus
                .iter()
                .map(|json| Haiku {
                    result_json: json.clone(),
                })
                .collect()),
            _ => Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: DB_QUERY_RESULT_PARSE_ERR }),
            )),
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::util;
    use super::*;
    use juniper::{EmptyMutation, RootNode, Variables};

    type Schema = RootNode<'static, DiscordUser, EmptyMutation<()>>;

    #[test]
    fn resolve_fields() {
        let user_json = json!(
        {
            "discordSnowflake": "0000000000000000001",
            "haikus": [{
                "id": "1"
            }],
        });
        let query = r#"
        query {
            discordSnowflake
            haikus {
                id
            }
        }"#;
        let (result, _errs) = juniper::execute(
            query,
            None,
            &Schema::new(
                DiscordUser {
                    result_json: user_json,
                },
                EmptyMutation::new(),
            ),
            &Variables::new(),
            &(),
        )
        .unwrap();
        assert_eq!(
            result,
            graphql_value!({
                "discordSnowflake": "0000000000000000001",
                "haikus": [{
                    "id": "1",
                }],
            })
        )
    }

    #[test]
    fn resolve_missing_fields() {
        util::resolve_missing_field::<DiscordUser>(
            r#"query { discordSnowflake }"#,
            "discordSnowflake",
            (),
        );
        util::resolve_missing_field::<DiscordUser>(r#"query { haikus { id } }"#, "haikus", ());
    }
}
