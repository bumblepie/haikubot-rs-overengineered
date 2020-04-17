use super::super::error::{
    QueryCreationError, DB_QUERY_RESULT_PARSE_ERR, INTERNAL_ERROR, UNABLE_TO_RESOLVE_FIELD,
};
use super::discord_channel::DiscordChannel;
use super::haiku::Haiku;
use super::util;
use juniper::{DefaultScalarValue, FieldError, FieldResult, LookAheadSelection};

#[derive(Debug)]
pub struct DiscordServer {
    pub result_json: serde_json::Value,
}

impl From<serde_json::Value> for DiscordServer {
    fn from(result_json: serde_json::Value) -> DiscordServer {
        DiscordServer { result_json }
    }
}

#[juniper::object]
impl DiscordServer {
    fn discordSnowflake(&self) -> FieldResult<String> {
        match self.result_json.get("discordSnowflake") {
            Some(serde_json::Value::String(id)) => Ok(id.clone()),
            _ => Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: DB_QUERY_RESULT_PARSE_ERR }),
            )),
        }
    }

    fn channels(&self) -> FieldResult<Vec<DiscordChannel>> {
        match self.result_json.get("channels") {
            Some(serde_json::Value::Array(channels)) => Ok(channels
                .iter()
                .map(|json| DiscordChannel {
                    result_json: json.clone(),
                })
                .collect()),
            _ => Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: DB_QUERY_RESULT_PARSE_ERR }),
            )),
        }
    }

    fn haikus(&self) -> FieldResult<Vec<Haiku>> {
        if let Some(serde_json::Value::Array(channels)) = self.result_json.get("haiku_channels") {
            Ok(channels
                .iter()
                .flat_map(|channel_json| match channel_json.get("haikus") {
                    Some(serde_json::Value::Array(haikus)) => haikus
                        .iter()
                        .map(|haiku_json| Haiku {
                            result_json: haiku_json.clone(),
                        })
                        .collect::<Vec<Haiku>>(),
                    _ => vec![],
                })
                .collect::<Vec<Haiku>>())
        } else {
            Ok(vec![])
        }
    }
}

impl util::MapsToDgraphQuery for DiscordServer {
    fn generate_inner_query_for_field(
        field_name: &str,
        child_selection: &LookAheadSelection<DefaultScalarValue>,
    ) -> Result<String, QueryCreationError> {
        match field_name {
            "discordSnowflake" => Ok("discordSnowflake".to_owned()),
            "channels" => Ok(format!(
                "channels: ~server @filter(type(DiscordChannel)) {{ {} }}",
                DiscordChannel::generate_inner_query(child_selection)?
            )),
            "haikus" => Ok(format!(
                r#"
                haiku_channels: ~server @filter(type(DiscordChannel)) {{
                    haikus: ~channel @filter(type(Haiku)) {{
                        {}
                    }}
                }}"#,
                Haiku::generate_inner_query(child_selection)?
            )),
            unknown_field => Err(QueryCreationError::UnknownField(unknown_field.to_owned())),
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::util;
    use super::*;
    use juniper::{EmptyMutation, RootNode, Variables};

    type Schema = RootNode<'static, DiscordServer, EmptyMutation<()>>;

    #[test]
    fn resolve_fields() {
        let server_json = json!(
        {
            "discordSnowflake": "0000000000000000001",
            "channels": [{
                "discordSnowflake": "0000000000000000002"
            }],
            "haiku_channels": [
                {
                    "haikus": [{
                        "id": "1"
                    }],
                }
            ]
        });
        let query = r#"
        query {
            discordSnowflake
            channels {
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
                DiscordServer {
                    result_json: server_json,
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
                "channels": [{
                    "discordSnowflake": "0000000000000000002"
                }],
                "haikus": [{
                    "id": "1",
                }],
            })
        )
    }

    #[test]
    fn resolve_missing_fields() {
        util::resolve_missing_field_error::<DiscordServer>(
            r#"query { discordSnowflake }"#,
            "discordSnowflake",
            (),
        );
        // util::resolve_missing_field_error::<DiscordServer>(
        //     r#"query { haikus { id } }"#,
        //     "haikus",
        //     (),
        // );
        // util::resolve_missing_field_error::<DiscordServer>(
        //     r#"query { channels { discordSnowflake } }"#,
        //     "channels",
        //     (),
        // );
    }
}
