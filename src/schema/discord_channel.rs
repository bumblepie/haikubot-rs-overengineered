use super::super::error::{DB_QUERY_RESULT_PARSE_ERR, INTERNAL_ERROR, UNABLE_TO_RESOLVE_FIELD};
use juniper::{FieldError, FieldResult};

use super::discord_server::DiscordServer;
use super::haiku::Haiku;

#[derive(Debug)]
pub struct DiscordChannel {
    pub result_json: serde_json::Value,
}

impl From<serde_json::Value> for DiscordChannel {
    fn from(result_json: serde_json::Value) -> DiscordChannel {
        DiscordChannel { result_json }
    }
}

#[juniper::object]
impl DiscordChannel {
    fn discordSnowflake(&self) -> FieldResult<String> {
        match self.result_json.get("discordSnowflake") {
            Some(serde_json::Value::String(id)) => Ok(id.clone()),
            _ => Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: DB_QUERY_RESULT_PARSE_ERR }),
            )),
        }
    }

    fn server(&self) -> FieldResult<DiscordServer> {
        match self.result_json.get("server") {
            Some(json) => Ok(DiscordServer {
                result_json: json.clone(),
            }),
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

    type Schema = RootNode<'static, DiscordChannel, EmptyMutation<()>>;

    #[test]
    fn resolve_fields() {
        let channel_json = json!(
        {
            "discordSnowflake": "0000000000000000001",
            "server": {
                "discordSnowflake": "0000000000000000002"
            },
            "haikus": [{
                "id": "1"
            }],
        });
        let query = r#"
        query {
            discordSnowflake
            server {
                discordSnowflake
            }
            haikus {
                id
            }
        }"#;
        let (result, _errs) = juniper::execute(
            query,
            None,
            &Schema::new(
                DiscordChannel {
                    result_json: channel_json,
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
                "server": {
                    "discordSnowflake": "0000000000000000002"
                },
                "haikus": [{
                    "id": "1",
                }],
            })
        )
    }

    #[test]
    fn resolve_missing_fields() {
        util::resolve_missing_field::<DiscordChannel>(
            r#"query { discordSnowflake }"#,
            "discordSnowflake",
            (),
        );
        util::resolve_missing_field::<DiscordChannel>(r#"query { haikus { id } }"#, "haikus", ());
        util::resolve_missing_field::<DiscordChannel>(
            r#"query { server { discordSnowflake } }"#,
            "server",
            (),
        );
    }
}
