use super::super::error::{QueryCreationError, INTERNAL_ERROR, UNABLE_TO_RESOLVE_FIELD};
use super::haiku::Haiku;
use super::util;
use juniper::{DefaultScalarValue, FieldError, FieldResult, LookAheadSelection};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordUser {
    discord_snowflake: Option<String>,
    #[serde(default)]
    haikus: Vec<Haiku>,
}

#[juniper::object]
impl DiscordUser {
    fn discordSnowflake(&self) -> FieldResult<String> {
        self.discord_snowflake.clone().ok_or(FieldError::new(
            UNABLE_TO_RESOLVE_FIELD,
            graphql_value!({ INTERNAL_ERROR: INTERNAL_ERROR }),
        ))
    }

    fn haikus(&self) -> FieldResult<Vec<&Haiku>> {
        Ok(self.haikus.iter().collect())
    }
}

impl util::MapsToDgraphQuery for DiscordUser {
    fn generate_inner_query_for_field(
        field_name: &str,
        child_selection: &LookAheadSelection<DefaultScalarValue>,
    ) -> Result<String, QueryCreationError> {
        match field_name {
            "discordSnowflake" => Ok("discordSnowflake".to_owned()),
            "haikus" => Ok(format!(
                "haikus: ~author @filter(type(Haiku)) {{ {} }}",
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

    type Schema = RootNode<'static, DiscordUser, EmptyMutation<()>>;

    #[test]
    fn resolve_fields() {
        let user_json = json!(
        {
            "discordSnowflake": "0000000000000000001",
            "haikus": [{
                "id": "1"
            }],
        });
        let query = r#"
        query {
            discordSnowflake
            haikus {
                id
            }
        }"#;
        let (result, _errs) = juniper::execute(
            query,
            None,
            &Schema::new(
                serde_json::from_value(user_json).unwrap(),
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
                "haikus": [{
                    "id": "1",
                }],
            })
        )
    }

    #[rstest(query, expected_result,
        case("discordSnowflake", Err(vec!["discordSnowflake"])),
        case(r#"haikus { id }"#, Ok(graphql_value!({"haikus": []}))),
    )]
    fn resolve_missing_fields(query: &str, expected_result: Result<juniper::Value, Vec<&str>>) {
        util::resolve_missing_field::<DiscordUser>(query, (), expected_result);
    }
}
