use super::super::error::{DB_QUERY_RESULT_PARSE_ERR, INTERNAL_ERROR, UNABLE_TO_RESOLVE_FIELD};
use super::discord_channel::DiscordChannel;
use super::discord_server::DiscordServer;
use super::discord_user::DiscordUser;
use chrono::{DateTime, FixedOffset};
use juniper::{FieldError, FieldResult, ID};

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

    fn lines(&self) -> FieldResult<Vec<String>> {
        match self.result_json.get("lines") {
            Some(lines) => serde_json::from_value(lines.clone()).map_err(|_| {
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
            "lines": [
                "line 1",
                "line 2",
                "line 3"
            ],
            "channel": {
                "discordSnowflake": "0000000000000000002"
            },
            "server": {
                "discordSnowflake": "0000000000000000003"
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
            lines
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
                "lines": [
                    "line 1",
                    "line 2",
                    "line 3"
                ],
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
        util::resolve_missing_field::<Haiku>(r#"query { id }"#, "id", ());
        util::resolve_missing_field::<Haiku>(
            r#"query { authors { discordSnowflake } }"#,
            "authors",
            (),
        );
        util::resolve_missing_field::<Haiku>(r#"query { lines }"#, "lines", ());
        util::resolve_missing_field::<Haiku>(
            r#"query { channel { discordSnowflake } }"#,
            "channel",
            (),
        );
        util::resolve_missing_field::<Haiku>(
            r#"query { server { discordSnowflake } }"#,
            "server",
            (),
        );
        util::resolve_missing_field::<Haiku>(r#"query { rulesVersion }"#, "rulesVersion", ());
        util::resolve_missing_field::<Haiku>(r#"query { timestamp }"#, "timestamp", ());
    }
}
