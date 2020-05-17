use super::super::error::{internal_error, invalid_input, QueryCreationError};
use super::discord_channel::DiscordChannel;
use super::discord_server::DiscordServer;
use super::discord_user::DiscordUser;
use super::util;
use chrono::{DateTime, Utc};
use juniper::{DefaultScalarValue, FieldError, FieldResult, LookAheadMethods, LookAheadSelection};
use regex::Regex;

#[derive(Debug)]
pub struct Haiku {
    inner: serde_json::Value,
}

impl From<serde_json::Value> for Haiku {
    fn from(inner: serde_json::Value) -> Self {
        Self { inner }
    }
}

#[juniper::object]
impl Haiku {
    fn id(&self) -> FieldResult<String> {
        match self.inner.get("id") {
            Some(serde_json::Value::String(id)) => Ok(id.clone()),
            _ => Err(internal_error()),
        }
    }

    fn authors(&self) -> FieldResult<Vec<DiscordUser>> {
        match self.inner.get("authors") {
            Some(serde_json::Value::Array(authors)) => Ok(authors
                .iter()
                .map(|json| DiscordUser::from(json.clone()))
                .collect()),
            _ => Err(internal_error()),
        }
    }

    fn content(&self) -> FieldResult<String> {
        match self.inner.get("content") {
            Some(serde_json::Value::String(content)) => Ok(content.clone()),
            _ => Err(internal_error()),
        }
    }

    fn channel(&self) -> FieldResult<DiscordChannel> {
        match self.inner.get("channel") {
            Some(json) => Ok(DiscordChannel::from(json.clone())),
            _ => Err(internal_error()),
        }
    }

    fn server(&self) -> FieldResult<DiscordServer> {
        match self
            .inner
            .get("serverChannel")
            .map(|channel_json| channel_json.get("server"))
        {
            Some(Some(json)) => Ok(DiscordServer::from(json.clone())),
            _ => Err(internal_error()),
        }
    }

    fn rulesVersion(&self) -> FieldResult<i32> {
        if let Some(serde_json::Value::Number(version)) = self.inner.get("rulesVersion") {
            if let Some(version) = version.as_i64() {
                return Ok(version as i32);
            }
        }
        return Err(internal_error());
    }

    fn timestamp(&self) -> FieldResult<DateTime<Utc>> {
        match self.inner.get("timestamp") {
            Some(timestamp) => serde_json::from_value(timestamp.clone()).map_err(|err| {
                error!("Error deserializing timestamp");
                internal_error()
            }),
            _ => Err(internal_error()),
        }
    }
}

impl util::MapsToDgraphQuery for Haiku {
    fn generate_inner_query_for_field(
        child_selection: &LookAheadSelection<DefaultScalarValue>,
    ) -> Result<String, QueryCreationError> {
        match child_selection.field_name() {
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

pub fn valid_haiku_id(id: String) -> Result<String, FieldError> {
    lazy_static! {
        static ref HAIKU_ID_REGEX: Regex = Regex::new(r"^0x\d+$").unwrap();
    }
    if HAIKU_ID_REGEX.is_match(&id) {
        Ok(id)
    } else {
        Err(invalid_input(
            r#"Invalid haiku id: must be of the form "0x<ID>""#,
        ))
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
            &Schema::new(Haiku::from(haiku_json), EmptyMutation::new()),
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
