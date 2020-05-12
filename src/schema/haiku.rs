use super::super::error::{QueryCreationError, INTERNAL_ERROR, UNABLE_TO_RESOLVE_FIELD};
use super::discord_channel::DiscordChannel;
use super::discord_server::DiscordServer;
use super::discord_user::DiscordUser;
use super::util;
use chrono::{DateTime, Utc};
use juniper::{DefaultScalarValue, FieldError, FieldResult, LookAheadSelection, ID};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Haiku {
    id: Option<String>,
    authors: Option<Vec<DiscordUser>>,
    content: Option<String>,
    channel: Option<DiscordChannel>,
    server_channel: Option<ServerChannel>,
    rules_version: Option<i32>,
    timestamp: Option<DateTime<Utc>>,
}
#[derive(Debug, Deserialize)]
struct ServerChannel {
    server: Option<DiscordServer>,
}

#[juniper::object]
impl Haiku {
    fn id(&self) -> FieldResult<ID> {
        match self.id {
            Some(ref id) => Ok(ID::new(id)),
            None => Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: INTERNAL_ERROR }),
            )),
        }
    }

    fn authors(&self) -> FieldResult<Vec<&DiscordUser>> {
        match self.authors {
            Some(ref authors) => Ok(authors.iter().collect()),
            None => Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: INTERNAL_ERROR }),
            )),
        }
    }

    fn content(&self) -> FieldResult<String> {
        self.content.clone().ok_or(FieldError::new(
            UNABLE_TO_RESOLVE_FIELD,
            graphql_value!({ INTERNAL_ERROR: INTERNAL_ERROR }),
        ))
    }

    fn channel(&self) -> FieldResult<&DiscordChannel> {
        match self.channel {
            Some(ref channel) => Ok(channel),
            None => Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: INTERNAL_ERROR }),
            )),
        }
    }

    fn server(&self) -> FieldResult<&DiscordServer> {
        let server_channel = match self.server_channel {
            Some(ref server_channel) => Ok(server_channel),
            None => Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: INTERNAL_ERROR }),
            )),
        }?;
        match server_channel.server {
            Some(ref server) => Ok(server),
            None => Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: INTERNAL_ERROR }),
            )),
        }
    }

    fn rulesVersion(&self) -> FieldResult<i32> {
        self.rules_version.ok_or(FieldError::new(
            UNABLE_TO_RESOLVE_FIELD,
            graphql_value!({ INTERNAL_ERROR: INTERNAL_ERROR }),
        ))
    }

    fn timestamp(&self) -> FieldResult<DateTime<Utc>> {
        self.timestamp.ok_or(FieldError::new(
            UNABLE_TO_RESOLVE_FIELD,
            graphql_value!({ INTERNAL_ERROR: INTERNAL_ERROR }),
        ))
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
                serverChannel: channel @filter(type(DiscordChannel)) {{
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
    use super::*;
    use juniper::{EmptyMutation, RootNode, Variables};
    use rstest::rstest;

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
            "serverChannel": {
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
                serde_json::from_value(haiku_json).unwrap(),
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

    #[rstest(query, expected_result,
        case("id", Err(vec!["id"])),
        case(r#"authors { discordSnowflake }"#, Err(vec!["authors"])),
        case("content", Err(vec!["content"])),
        case(r#"channel { discordSnowflake }"#, Err(vec!["channel"])),
        case(r#"server { discordSnowflake }"#, Err(vec!["server"])),
        case("rulesVersion", Err(vec!["rulesVersion"])),
        case("timestamp", Err(vec!["timestamp"])),
    )]
    fn resolve_missing_fields(query: &str, expected_result: Result<juniper::Value, Vec<&str>>) {
        util::resolve_missing_field::<Haiku>(query, (), expected_result);
    }
}
