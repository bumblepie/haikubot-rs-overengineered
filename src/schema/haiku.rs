use super::super::error::{
    QueryCreationError, DB_QUERY_RESULT_PARSE_ERR, INTERNAL_ERROR, UNABLE_TO_RESOLVE_FIELD,
};
use super::discord_channel::DiscordChannel;
use super::discord_server::DiscordServer;
use super::discord_user::DiscordUser;
use super::util;
use chrono::{DateTime, FixedOffset};
use juniper::{DefaultScalarValue, FieldError, FieldResult, LookAheadSelection, ID};
#[derive(Debug)]
pub struct Haiku {
    pub result_json: serde_json::Value,
}

impl From<serde_json::Value> for Haiku {
    fn from(result_json: serde_json::Value) -> Haiku {
        Haiku { result_json }
    }
}

#[juniper::object]
impl Haiku {
    fn id(&self) -> FieldResult<ID> {
        match self.result_json.get("id") {
            Some(serde_json::Value::String(id)) => Ok(ID::new(id)),
            _ => Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: DB_QUERY_RESULT_PARSE_ERR }),
            )),
        }
    }

    fn authors(&self) -> FieldResult<Vec<DiscordUser>> {
        match self.result_json.get("authors") {
            Some(serde_json::Value::Array(authors)) => Ok(authors
                .iter()
                .map(|json| DiscordUser {
                    result_json: json.clone(),
                })
                .collect()),
            _ => Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: DB_QUERY_RESULT_PARSE_ERR }),
            )),
        }
    }

    fn content(&self) -> FieldResult<String> {
        match self.result_json.get("content") {
            Some(serde_json::Value::String(content)) => Ok(content.clone()),
            _ => Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: DB_QUERY_RESULT_PARSE_ERR }),
            )),
        }
    }

    fn channel(&self) -> FieldResult<DiscordChannel> {
        match self.result_json.get("channel") {
            Some(json) => Ok(DiscordChannel {
                result_json: json.clone(),
            }),
            _ => Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: DB_QUERY_RESULT_PARSE_ERR }),
            )),
        }
    }

    fn server(&self) -> FieldResult<DiscordServer> {
        match self
            .result_json
            .get("server_channel")
            .map(|channel_json| channel_json.get("server"))
        {
            Some(Some(json)) => Ok(DiscordServer {
                result_json: json.clone(),
            }),
            _ => Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: DB_QUERY_RESULT_PARSE_ERR }),
            )),
        }
    }

    fn rulesVersion(&self) -> FieldResult<i32> {
        if let Some(serde_json::Value::Number(version)) = self.result_json.get("rulesVersion") {
            if let Some(version) = version.as_i64() {
                return Ok(version as i32);
            }
        }
        return Err(FieldError::new(
            UNABLE_TO_RESOLVE_FIELD,
            graphql_value!({ INTERNAL_ERROR: DB_QUERY_RESULT_PARSE_ERR }),
        ));
    }

    fn timestamp(&self) -> FieldResult<DateTime<FixedOffset>> {
        match self.result_json.get("timestamp") {
            Some(serde_json::Value::String(ts)) => DateTime::<FixedOffset>::parse_from_rfc3339(ts)
                .map_err(|_| {
                    FieldError::new(
                        UNABLE_TO_RESOLVE_FIELD,
                        graphql_value!({ INTERNAL_ERROR: DB_QUERY_RESULT_PARSE_ERR }),
                    )
                }),
            _ => Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: DB_QUERY_RESULT_PARSE_ERR }),
            )),
        }
    }
}

impl util::MapsToDgraphQuery for Haiku {
    fn generate_inner_query_for_field(
        field_name: &str,
        child_selection: &LookAheadSelection<DefaultScalarValue>,
    ) -> Result<String, QueryCreationError> {
        match field_name {
            "id" => Ok("id: uid".to_owned()),
            "authors" => Ok(format!(
                "authors: author @filter(type(DiscordUser)) {{ {} }}",
                DiscordUser::generate_inner_query(child_selection)?
            )),
            "content" => Ok("content".to_owned()),
            "channel" => Ok(format!(
                "channel @filter(type(DiscordChannel)) {{ {} }}",
                DiscordChannel::generate_inner_query(child_selection)?
            )),
            "server" => Ok(format!(
                r#"
                server_channel: channel @filter(type(DiscordChannel)) {{
                    server @filter(type(DiscordServer)) {{
                        {}
                    }}
                }}"#,
                DiscordServer::generate_inner_query(child_selection)?
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

    type Schema = RootNode<'static, Haiku, EmptyMutation<()>>;

    #[test]
    fn resolve_fields() {
        let haiku_json = json!(
        {
            "id": "1",
            "authors": [{
                "discordSnowflake": "0000000000000000001"
            }],
            "content": "line 1\nline 2\nline 3",
            "channel": {
                "discordSnowflake": "0000000000000000002"
            },
            "server_channel": {
                "server": {
                    "discordSnowflake": "0000000000000000003"
                }
            },
            "rulesVersion": 1,
            "timestamp": "1977-02-03T05:00:00+00:00"
        });
        let query = r#"
        query {
            id
            authors {
                discordSnowflake
            }
            content
            channel {
                discordSnowflake
            }
            server {
                discordSnowflake
            }
            rulesVersion
            timestamp
        }"#;
        let (result, _errs) = juniper::execute(
            query,
            None,
            &Schema::new(
                Haiku {
                    result_json: haiku_json,
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
                "id": "1",
                "authors": [{
                    "discordSnowflake": "0000000000000000001"
                }],
                "content": "line 1\nline 2\nline 3",
                "channel": {
                    "discordSnowflake": "0000000000000000002"
                },
                "server": {
                    "discordSnowflake": "0000000000000000003"
                },
                "rulesVersion": 1,
                "timestamp": "1977-02-03T05:00:00+00:00"
            })
        )
    }

    #[test]
    fn resolve_missing_fields() {
        util::resolve_missing_field_error::<Haiku>(r#"query { id }"#, "id", ());
        util::resolve_missing_field_error::<Haiku>(
            r#"query { authors { discordSnowflake } }"#,
            "authors",
            (),
        );
        util::resolve_missing_field_error::<Haiku>(r#"query { content }"#, "content", ());
        util::resolve_missing_field_error::<Haiku>(
            r#"query { channel { discordSnowflake } }"#,
            "channel",
            (),
        );
        util::resolve_missing_field_error::<Haiku>(
            r#"query { server { discordSnowflake } }"#,
            "server",
            (),
        );
        util::resolve_missing_field_error::<Haiku>(r#"query { rulesVersion }"#, "rulesVersion", ());
        util::resolve_missing_field_error::<Haiku>(r#"query { timestamp }"#, "timestamp", ());
    }
}
