use super::super::error::{
    CompositeQueryCreationError, QueryCreationError, INTERNAL_ERROR, UNABLE_TO_RESOLVE_FIELD,
};
use juniper::{
    DefaultScalarValue, EmptyMutation, FieldError, GraphQLType, LookAheadMethods,
    LookAheadSelection, RootNode, Variables,
};
use serde_json::json;

#[allow(dead_code)]
pub fn resolve_missing_field<T>(
    query: &str,
    context: <T as GraphQLType>::Context,
    expected_result: Result<juniper::Value, Vec<&str>>,
) where
    T: From<serde_json::Value> + GraphQLType<TypeInfo = ()>,
{
    let query = format!(r#"query {{ {} }}"#, query);
    let (result, errs) = juniper::execute(
        &query,
        None,
        &RootNode::new(T::from(json!({})), EmptyMutation::new()),
        &Variables::new(),
        &context,
    )
    .unwrap();
    match expected_result {
        Ok(expected_val) => {
            assert_eq!(result, expected_val);
            assert_eq!(errs.len(), 0);
        }
        Err(error_path) => {
            assert_eq!(result, juniper::Value::Null);
            assert_eq!(errs.len(), 1);
            let err = &errs[0];
            assert_eq!(err.path(), &error_path[..]);
            assert_eq!(
                err.error(),
                &FieldError::new(
                    UNABLE_TO_RESOLVE_FIELD,
                    graphql_value!({ INTERNAL_ERROR: INTERNAL_ERROR }),
                )
            );
        }
    };
}

pub trait MapsToDgraphQuery {
    fn generate_inner_query_for_field(
        child_selection: &LookAheadSelection<DefaultScalarValue>,
    ) -> Result<String, QueryCreationError>;

    fn generate_inner_query(
        selection: &LookAheadSelection<DefaultScalarValue>,
    ) -> Result<String, QueryCreationError> {
        let (query_sections, errs): (Vec<_>, Vec<_>) = selection
            .child_names()
            .iter()
            .map(|field_name| selection.select_child(field_name).unwrap())
            .map(|child_selection| Self::generate_inner_query_for_field(child_selection))
            .partition(Result::is_ok);
        if errs.is_empty() {
            // Extract Vec<Result<String, QueryCreationError>> into Vec<String> and join
            Ok(query_sections
                .into_iter()
                .map(Result::unwrap)
                .collect::<Vec<String>>()
                .join("\n"))
        } else {
            // Extract Vec<Result<String, QueryCreationError>> into Vec<QueryCreationError>
            // Gather errors into composite error
            Err(QueryCreationError::Composite(CompositeQueryCreationError {
                at_field: selection.field_name().to_owned(),
                children: errs.into_iter().map(Result::unwrap_err).collect(),
            }))
        }
    }
}
