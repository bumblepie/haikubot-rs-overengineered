use super::super::error::*;
use juniper::{DefaultScalarValue, FieldError, FieldResult, LookAheadMethods, LookAheadSelection};

#[derive(Debug)]
pub struct Person {
    pub result_json: serde_json::Value,
}

#[juniper::object]
#[derive(Debug)]
impl Person {
    fn name(&self) -> FieldResult<String> {
        match self.result_json.get("name") {
            Some(serde_json::Value::String(name)) => Ok(name.clone()),
            _ => Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: DB_QUERY_RESULT_PARSE_ERR }),
            )),
        }
    }

    fn friends() -> FieldResult<Vec<Person>> {
        match &self.result_json.get("friend") {
            Some(serde_json::Value::Array(friends)) => Ok(friends
                .iter()
                .map(|json| Person {
                    result_json: json.clone(),
                })
                .collect()),
            None => Ok(Vec::new()),
            _ => Err(FieldError::new(
                UNABLE_TO_RESOLVE_FIELD,
                graphql_value!({ INTERNAL_ERROR: DB_QUERY_RESULT_PARSE_ERR }),
            )),
        }
    }

    fn bestFriend() -> FieldResult<Option<Person>> {
        let best_friend_array = match &self.result_json.get("bestFriend") {
            Some(serde_json::Value::Array(json)) => json,
            _ => {
                return Err(FieldError::new(
                    UNABLE_TO_RESOLVE_FIELD,
                    graphql_value!({ INTERNAL_ERROR: DB_QUERY_RESULT_PARSE_ERR }),
                ))
            }
        };
        let best_friend = best_friend_array.get(0);
        Ok(best_friend.map(|json| Person {
            result_json: json.clone(),
        }))
    }
}

impl Person {
    pub fn generate_query(
        selection: &LookAheadSelection<DefaultScalarValue>,
    ) -> Result<String, QueryCreationError> {
        let (query_sections, errs): (Vec<_>, Vec<_>) = selection
            .child_names()
            .iter()
            .map(|field_name| match *field_name {
                // "name" => Ok("name".to_owned()),
                "friends" => Person::generate_query(selection.select_child(field_name).unwrap())
                    .map(|inner_query| format!("friend {{\n{}\n}}", inner_query)),
                "bestFriend" => Person::generate_query(selection.select_child(field_name).unwrap())
                    .map(|inner_query| {
                        format!(
                            "bestFriend: friend @facets(orderdesc: score) (first: 1) {{\n{}\n}}",
                            inner_query
                        )
                    }),
                unknown_field => Err(QueryCreationError::UnknownField(unknown_field.to_owned())),
            })
            .partition(Result::is_ok);
        if !errs.is_empty() {
            // Extract Vec<Result<String, QueryCreationError>> into Vec<QueryCreationError>
            // Gather errors into composite error
            Err(QueryCreationError::Composite(CompositeQueryCreationError {
                at_field: selection.field_name().to_owned(),
                children: errs.into_iter().map(|result| result.unwrap_err()).collect(),
            }))
        } else {
            // Extract Vec<Result<String, QueryCreationError>> into Vec<String> and join
            Ok(query_sections
                .into_iter()
                .map(|result| result.ok().unwrap())
                .collect::<Vec<String>>()
                .join("\n"))
        }
    }
}
