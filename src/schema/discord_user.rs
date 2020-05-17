use super::super::error::{QueryCreationError, INTERNAL_ERROR, UNABLE_TO_RESOLVE_FIELD};
use super::haiku::Haiku;
use super::util;
use juniper::{
    DefaultScalarValue, FieldError, FieldResult, LookAheadMethods, LookAheadSelection,
    LookAheadValue,
};
use regex::Regex;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

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

    fn haikus_search(&self, search_term: String, max: i32) -> FieldResult<Vec<Haiku>> {
        let alias = Self::haikus_search_alias(&search_term, &max);
        match self.inner.get(&alias) {
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

impl DiscordUser {
    fn haikus_search_alias(search_term: &str, max: &i32) -> String {
        let mut hasher = DefaultHasher::new();
        search_term.hash(&mut hasher);
        max.hash(&mut hasher);
        format!("haikusSearch_{:#x}", hasher.finish())
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
                let args: HashMap<_, _> = child_selection
                    .arguments()
                    .iter()
                    .map(|arg| (arg.name(), arg.value()))
                    .collect();
                let search_term = match args.get("searchTerm") {
                    Some(LookAheadValue::Scalar(DefaultScalarValue::String(term))) => Ok(term),
                    _ => Err(QueryCreationError::BadArgument("searchTerm".to_owned())),
                }?
                .clone();
                let search_term = valid_search_terms(search_term)
                    .map_err(|_| QueryCreationError::BadArgument("searchTerm".to_owned()))?;
                let max = match args.get("max") {
                    Some(LookAheadValue::Scalar(DefaultScalarValue::Int(max))) => Ok(max),
                    _ => Err(QueryCreationError::BadArgument("max".to_owned())),
                }?;

                Ok(format!(
                    r#"{}: ~author @filter(type(Haiku) AND anyofterms(content, "{}")) (first: {}) {{ {} }}"#,
                    DiscordUser::haikus_search_alias(&search_term, max),
                    search_term,
                    max,
                    Haiku::generate_inner_query(child_selection)?
                ))
            }
            unknown_field => Err(QueryCreationError::UnknownField(unknown_field.to_owned())),
        }
    }
}

fn valid_search_terms(terms: String) -> Result<String, ()> {
    lazy_static! {
        static ref SEARCH_TERM_REGEX: Regex =
            Regex::new(r"^([[:alpha:]]+ )*[[:alpha:]]+$").unwrap();
    }
    if SEARCH_TERM_REGEX.is_match(&terms) {
        Ok(terms)
    } else {
        Err(())
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
            DiscordUser::haikus_search_alias("a", &2): [{
                "id": "1"
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
