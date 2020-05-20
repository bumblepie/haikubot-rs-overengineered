use super::super::error::{QueryCreationError, INTERNAL_ERROR, UNABLE_TO_RESOLVE_FIELD};
use super::discord_channel::DiscordChannel;
use super::haiku::Haiku;
use super::util;
use juniper::{DefaultScalarValue, FieldError, FieldResult, LookAheadSelection};

#[derive(Debug)]
pub struct DiscordServer {
    inner: serde_json::Value,
}

impl From<serde_json::Value> for DiscordServer {
    fn from(inner: serde_json::Value) -> Self {
        Self { inner }
    }
}

#[juniper::object]
impl DiscordServer {
    fn discordSnowflake(&self) -> FieldResult<String> {
        match self.inner.get("discordSnowflake") {
            Some(serde_json::Value::String(snowflake)) => Ok(snowflake.clone()),
            _ => Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: INTERNAL_ERROR }),
            )),
        }
    }

    fn channels(&self) -> FieldResult<Vec<DiscordChannel>> {
        match self.inner.get("channels") {
            Some(serde_json::Value::Array(channels)) => Ok(channels
                .iter()
                .map(|json| DiscordChannel::from(json.clone()))
                .collect()),
            None => Ok(Vec::new()),
            _ => Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: INTERNAL_ERROR }),
            )),
        }
    }

    fn haikus(&self) -> FieldResult<Vec<Haiku>> {
        if let Some(serde_json::Value::Array(channels)) = self.inner.get("haikuChannels") {
            Ok(channels
                .iter()
                .flat_map(|channel_json| match channel_json.get("haikus") {
                    Some(serde_json::Value::Array(haikus)) => haikus
                        .iter()
                        .map(|json| Haiku::from(json.clone()))
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
                haikuChannels: ~server @filter(type(DiscordChannel)) {{
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
    use super::*;
    use juniper::{EmptyMutation, RootNode, Variables};
    use rstest::rstest;

    type Schema = RootNode<'static, DiscordServer, EmptyMutation<()>>;

    #[test]
    fn resolve_fields() {
        let server_json = json!(
        {
            "discordSnowflake": "0000000000000000001",
            "channels": [{
                "discordSnowflake": "0000000000000000002"
            }],
            "haikuChannels": [
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
            &Schema::new(DiscordServer::from(server_json), EmptyMutation::new()),
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

    #[rstest(query, expected_result,
        case("discordSnowflake", Err(vec!["discordSnowflake"])),
        case(r#"channels { discordSnowflake }"#, Ok(graphql_value!({"channels": []}))),
        case(r#"haikus { id }"#, Ok(graphql_value!({"haikus": []}))),
    )]
    fn resolve_missing_fields(query: &str, expected_result: Result<juniper::Value, Vec<&str>>) {
        util::resolve_missing_field::<DiscordServer>(query, (), expected_result);
    }
}
