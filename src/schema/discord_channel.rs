use super::super::error::{QueryCreationError, INTERNAL_ERROR, UNABLE_TO_RESOLVE_FIELD};
use super::discord_server::DiscordServer;
use super::haiku::Haiku;
use super::util;
use juniper::{DefaultScalarValue, FieldError, FieldResult, LookAheadSelection};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordChannel {
    discord_snowflake: Option<String>,
    server: Option<DiscordServer>,
    #[serde(default)]
    haikus: Vec<Haiku>,
}

#[juniper::object]
impl DiscordChannel {
    fn discordSnowflake(&self) -> FieldResult<String> {
        self.discord_snowflake.clone().ok_or(FieldError::new(
            UNABLE_TO_RESOLVE_FIELD,
            graphql_value!({ INTERNAL_ERROR: INTERNAL_ERROR }),
        ))
    }

    fn server(&self) -> FieldResult<&DiscordServer> {
        match self.server {
            Some(ref server) => Ok(server),
            None => Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: INTERNAL_ERROR }),
            )),
        }
    }

    fn haikus(&self) -> FieldResult<Vec<&Haiku>> {
        Ok(self.haikus.iter().collect())
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
            &Schema::new(
                serde_json::from_value::<DiscordChannel>(channel_json).unwrap(),
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

    #[rstest(query, expected_result,
        case("discordSnowflake", Err(vec!["discordSnowflake"])),
        case(r#"server { discordSnowflake }"#, Err(vec!["server"])),
        case(r#"haikus { id }"#, Ok(graphql_value!({"haikus": []}))),
    )]
    fn resolve_missing_fields(query: &str, expected_result: Result<juniper::Value, Vec<&str>>) {
        util::resolve_missing_field::<DiscordChannel>(query, (), expected_result);
    }
}
