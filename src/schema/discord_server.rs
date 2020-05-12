use super::super::error::{QueryCreationError, INTERNAL_ERROR, UNABLE_TO_RESOLVE_FIELD};
use super::discord_channel::DiscordChannel;
use super::haiku::Haiku;
use super::util;
use juniper::{DefaultScalarValue, FieldError, FieldResult, LookAheadSelection};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordServer {
    discord_snowflake: Option<String>,
    #[serde(default)]
    channels: Vec<DiscordChannel>,
    #[serde(default)]
    haiku_channels: Vec<HaikuChannel>,
}
#[derive(Debug, Deserialize)]
struct HaikuChannel {
    #[serde(default)]
    haikus: Vec<Haiku>,
}

#[juniper::object]
impl DiscordServer {
    fn discordSnowflake(&self) -> FieldResult<String> {
        self.discord_snowflake.clone().ok_or(FieldError::new(
            UNABLE_TO_RESOLVE_FIELD,
            graphql_value!({ INTERNAL_ERROR: INTERNAL_ERROR }),
        ))
    }

    fn channels(&self) -> FieldResult<Vec<&DiscordChannel>> {
        Ok(self.channels.iter().collect())
    }

    fn haikus(&self) -> FieldResult<Vec<&Haiku>> {
        Ok(self
            .haiku_channels
            .iter()
            .flat_map(|channel| channel.haikus.iter())
            .collect())
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
            &Schema::new(
                serde_json::from_value(server_json).unwrap(),
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

    #[rstest(query, expected_result,
        case("discordSnowflake", Err(vec!["discordSnowflake"])),
        case(r#"channels { discordSnowflake }"#, Ok(graphql_value!({"channels": []}))),
        case(r#"haikus { id }"#, Ok(graphql_value!({"haikus": []}))),
    )]
    fn resolve_missing_fields(query: &str, expected_result: Result<juniper::Value, Vec<&str>>) {
        util::resolve_missing_field::<DiscordServer>(query, (), expected_result);
    }
}
