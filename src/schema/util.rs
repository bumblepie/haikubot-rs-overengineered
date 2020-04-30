use super::super::error::{DB_QUERY_RESULT_PARSE_ERR, INTERNAL_ERROR, UNABLE_TO_RESOLVE_FIELD};
use juniper::{EmptyMutation, FieldError, GraphQLType, RootNode, Variables};
use serde_json::json;

/// Test what happens when the json returned by DGraph does not contain a field
pub fn resolve_missing_field<T>(query: &str, path: &str, context: <T as GraphQLType>::Context)
where
    T: From<serde_json::Value> + GraphQLType<TypeInfo = ()>,
{
    let (result, errs) = juniper::execute(
        query,
        None,
        &RootNode::new(T::from(json!({})), EmptyMutation::new()),
        &Variables::new(),
        &context,
    )
    .unwrap();
    assert_eq!(result, juniper::Value::Null);
    assert_eq!(errs.len(), 1);
    let err = &errs[0];
    assert_eq!(err.path(), &[path]);
    assert_eq!(
        err.error(),
        &FieldError::new(
            UNABLE_TO_RESOLVE_FIELD,
            graphql_value!({ INTERNAL_ERROR: DB_QUERY_RESULT_PARSE_ERR }),
        )
    );
}
