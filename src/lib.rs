use anyhow::{anyhow, Result};
use async_graphql_parser::types::OperationDefinition;
use serde_json::Value;
use std::collections::HashMap;

pub fn resolve_op_value(
    name: &str,
    operation: &OperationDefinition,
    key_values: &Vec<(String, String)>,
) -> Result<String> {
    let operation = operation.clone();
    let mut subtitution_map = HashMap::new();
    for (k, v) in key_values {
        subtitution_map.insert(k, v);
    }
    let item = operation.selection_set.into_inner().items[0]
        .clone()
        .into_inner();
    let query_arguments = match item {
        async_graphql_parser::types::Selection::Field(data) => data.into_inner().arguments,
        async_graphql_parser::types::Selection::FragmentSpread(_) => {
            return Err(anyhow!("Invalid Operation"));
        }
        async_graphql_parser::types::Selection::InlineFragment(_) => {
            return Err(anyhow!("Invalid_operation"));
        }
    };
    let operations_ls: Vec<&str> = name.split(".").collect();
    if operations_ls.is_empty() {
        return Err(anyhow!("Invalid value to resolve"));
    }
    let value = query_arguments
        .clone()
        .into_iter()
        .find(|(key, _)| key.clone().into_inner() == *operations_ls[0])
        .ok_or(anyhow!("Invalid value to resolve"))?
        .1
        .into_inner();

    let json_val = value.into_json().map_err(|e| anyhow!("{:?}", e))?;
    let parsed_value = parse_value(json_val, operations_ls[1..].iter())?
        .trim_matches('\"')
        .to_string();
    Ok(subtitution_map
        .get(&parsed_value)
        .unwrap_or(&&parsed_value)
        .to_string())
}

pub fn parse_value(value: Value, mut resolve_values: std::slice::Iter<'_, &str>) -> Result<String> {
    let next_data = resolve_values
        .next()
        .ok_or(anyhow!("invalid combination of data"))?;
    match value {
        serde_json::Value::Null => return Ok("".into()),
        serde_json::Value::Bool(data) => return Ok(data.to_string()),
        serde_json::Value::Number(data) => return Ok(data.to_string()),
        serde_json::Value::String(data) => return Ok(data),
        serde_json::Value::Array(data) => return Ok(format!("{:?}", data)),
        serde_json::Value::Object(data) => {
            let data = data
                .get((*next_data).into())
                .ok_or(anyhow!("Invalid combination of data"))?;
            if let Some(result_value) = data.get("$var") {
                return Ok(result_value.to_string());
            } else {
                return parse_value(data.clone(), resolve_values);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_graphql_parser::parse_query;

    #[test]
    fn it_works() {
        let query = r#"
           mutation {
            updatepetwithdetail(
            petid: 1,
            details: {
            age: $age,
            weight: 13,
            band: "ac/dc",
            more: {
            gender: $gender,
            even_more: { 
                city: $big_city
                }
            }
            },
            hungry: true,
            name: "Honolulu"
            )
            }        
        "#;
        let subtitutions: Vec<(String, String)> = vec![
            ("age".into(), "20".into()),
            ("gender".into(), "male".into()),
            ("big_city".into(), "Islamabad".into()),
        ];
        let query = parse_query(query).unwrap();
        let doc_definition = query
            .operations
            .iter()
            .next()
            .unwrap()
            .1
            .clone()
            .into_inner();
        resolve_op_value("details.more.gender", &doc_definition, &subtitutions).unwrap();
    }
}
