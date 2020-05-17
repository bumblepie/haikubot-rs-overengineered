use super::super::error::{internal_error, QueryCreationError};
use super::haiku::Haiku;
use super::util;
use juniper::{
    DefaultScalarValue, FieldResult, LookAheadMethods, LookAheadSelection, LookAheadValue,
};
use regex::Regex;

#[derive(Debug)]
pub struct DiscordUser {
    inner: serde_json::Value,
}

impl From<serde_json::Value> for DiscordUser {
    fn from(inner: serde_json::Value) -> Self {
        Self { inner }
    }
}

#[juniper::object]
impl DiscordUser {
    fn discordSnowflake(&self) -> FieldResult<String> {
        match self.inner.get("discordSnowflake") {
            Some(serde_json::Value::String(snowflake)) => Ok(snowflake.clone()),
            _ => Err(internal_error()),
        }
    }

    fn haikus(&self) -> FieldResult<Vec<Haiku>> {
        match self.inner.get("haikus") {
            Some(serde_json::Value::Array(haikus)) => Ok(haikus
                .iter()
                .map(|json| Haiku::from(json.clone()))
                .collect()),
            None => Ok(Vec::new()),
            _ => Err(internal_error()),
        }
    }

    fn haikus_search(&self, search_term: String, max: i32) -> FieldResult<Vec<Haiku>> {
        let alias = format!("haikusSearch_{:#x}", hash!(&search_term, &max));
        match self.inner.get(&alias) {
            Some(serde_json::Value::Array(haikus)) => Ok(haikus
                .iter()
                .map(|json| Haiku::from(json.clone()))
                .collect()),
            None => Ok(Vec::new()),
            _ => Err(internal_error()),
        }
    }
}

impl util::MapsToDgraphQuery for DiscordUser {
    fn generate_inner_query_for_field(
        child_selection: &LookAheadSelection<DefaultScalarValue>,
    ) -> Result<String, QueryCreationError> {
        match child_selection.field_name() {
            "discordSnowflake" => Ok("discordSnowflake".to_owned()),
            "haikus" => Ok(format!(
                "haikus: ~author @filter(type(Haiku)) {{ {} }}",
                Haiku::generate_inner_query(child_selection)?
            )),
            "haikusSearch" => {
                let search_term = child_selection
                    .argument("searchTerm")
                    .ok_or(QueryCreationError::MissingArgument("searchTerm".to_owned()))?;
                let search_term = match search_term.value() {
                    LookAheadValue::Scalar(DefaultScalarValue::String(term)) => Ok(term),
                    _ => Err(QueryCreationError::InvalidArgument(
                        "searchTerm".to_owned(),
                        "must be a string".to_owned(),
                    )),
                }?
                .clone();
                let search_term = valid_search_terms(search_term)?;

                let max = child_selection
                    .argument("max")
                    .ok_or(QueryCreationError::MissingArgument("max".to_owned()))?;
                let max = match max.value() {
                    LookAheadValue::Scalar(DefaultScalarValue::Int(max)) => Ok(max),
                    _ => Err(QueryCreationError::InvalidArgument(
                        "max".to_owned(),
                        "must be an integer".to_owned(),
                    )),
                }?;

                Ok(format!(
                    r#"{}: ~author @filter(type(Haiku) AND anyofterms(content, "{}")) (first: {}) {{ {} }}"#,
                    format!("haikusSearch_{:#x}", hash!(&search_term, &max)),
                    search_term,
                    max,
                    Haiku::generate_inner_query(child_selection)?
                ))
            }
            unknown_field => Err(QueryCreationError::UnknownField(unknown_field.to_owned())),
        }
    }
}

fn valid_search_terms(terms: String) -> Result<String, QueryCreationError> {
    lazy_static! {
        static ref SEARCH_TERM_REGEX: Regex =
            Regex::new(r"^([[:alpha:]]+ )*[[:alpha:]]+$").unwrap();
    }
    if SEARCH_TERM_REGEX.is_match(&terms) {
        Ok(terms)
    } else {
        Err(QueryCreationError::InvalidArgument(
            "searchTerm".to_owned(),
            "must be a set of words made up of only alphabetic characters separated by spaces"
                .to_owned(),
        ))
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
                "id": "1",
                "id": "2"
            }],
            format!("haikusSearch_{:#x}", hash!("a", &2)): [{
                "id": "1"
            }],
            format!("haikusSearch_{:#x}", hash!("b", &2)): [{
                "id": "2"
            }],
        });
        let query = r#"
        query {
            discordSnowflake
            haikus {
                id
            }
            haikusSearch(searchTerm: "a", max: 2) {
                id
            }
            secondSearch: haikusSearch(searchTerm: "b", max: 2) {
                id
            }
        }"#;
        let (result, _errs) = juniper::execute(
            query,
            None,
            &Schema::new(DiscordUser::from(user_json), EmptyMutation::new()),
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
                    "id": "2"
                }],
                "haikusSearch": [{
                    "id": "1",
                }],
                "secondSearch": [{
                    "id": "2",
                }],
            })
        )
    }

    #[rstest(query, expected_result,
        case("discordSnowflake", Err(vec!["discordSnowflake"])),
        case(r#"haikus { id }"#, Ok(graphql_value!({"haikus": []}))),
        case(r#"haikusSearch(searchTerm: "a", max: 2) { id }"#, Ok(graphql_value!({"haikusSearch": []}))),
    )]
    fn resolve_missing_fields(query: &str, expected_result: Result<juniper::Value, Vec<&str>>) {
        util::resolve_missing_field::<DiscordUser>(query, (), expected_result);
    }
}
