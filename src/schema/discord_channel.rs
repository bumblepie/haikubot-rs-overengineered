use super::super::error::{QueryCreationError, INTERNAL_ERROR, UNABLE_TO_RESOLVE_FIELD};
use super::discord_server::DiscordServer;
use super::haiku::Haiku;
use super::util;
use juniper::{DefaultScalarValue, FieldError, FieldResult, LookAheadSelection};

#[derive(Debug)]
pub struct DiscordChannel {
    inner: serde_json::Value,
}

impl From<serde_json::Value> for DiscordChannel {
    fn from(inner: serde_json::Value) -> Self {
        Self { inner }
    }
}

#[juniper::object]
impl DiscordChannel {
    fn discordSnowflake(&self) -> FieldResult<String> {
        match self.inner.get("discordSnowflake") {
            Some(serde_json::Value::String(snowflake)) => Ok(snowflake.clone()),
            _ => Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: INTERNAL_ERROR }),
            )),
        }
    }

    fn server(&self) -> FieldResult<DiscordServer> {
        match self.inner.get("server") {
            Some(server) => Ok(DiscordServer::from(server.clone())),
            _ => Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: INTERNAL_ERROR }),
            )),
        }
    }

    fn haikus(&self) -> FieldResult<Vec<Haiku>> {
        match self.inner.get("haikus") {
            Some(serde_json::Value::Array(haikus)) => Ok(haikus
                .iter()
                .map(|json| Haiku::from(json.clone()))
                .collect()),
            None => Ok(Vec::new()),
            _ => Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: INTERNAL_ERROR }),
            )),
        }
    }
}

impl util::MapsToDgraphQuery for DiscordChannel {
    fn generate_inner_query_for_field(
        field_name: &str,
        child_selection: &LookAheadSelection<DefaultScalarValue>,
    ) -> Result<String, QueryCreationError> {
        match field_name {
            "discordSnowflake" => Ok("discordSnowflake".to_owned()),
            "server" => Ok(format!(
                "server: server @filter(type(DiscordServer)) {{ {} }}",
                DiscordServer::generate_inner_query(child_selection)?
            )),
            "haikus" => Ok(format!(
                "haikus: ~channel @filter(type(Haiku)) {{ {} }}",
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
            &Schema::new(DiscordChannel::from(channel_json), EmptyMutation::new()),
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

    #[rstest(query, expected_result,
        case("discordSnowflake", Err(vec!["discordSnowflake"])),
        case(r#"server { discordSnowflake }"#, Err(vec!["server"])),
        case(r#"haikus { id }"#, Ok(graphql_value!({"haikus": []}))),
    )]
    fn resolve_missing_fields(query: &str, expected_result: Result<juniper::Value, Vec<&str>>) {
        util::resolve_missing_field::<DiscordChannel>(query, (), expected_result);
    }
}
