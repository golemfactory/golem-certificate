use anyhow::Result;
use serde_json::{ json, Value };
use test_case::test_case;

use jcs::*;

// These do not play well with test_case macro
#[test]
fn literals() -> Result<()> {
    assert_eq!("null", to_string(&json!(null))?);
    assert_eq!("true", to_string(&json!(true))?);
    assert_eq!("false", to_string(&json!(false))?);
    Ok(())
}

#[test_case(json!(42), "42" ; "number")]
#[test_case(json!("42"), "\"42\"" ; "string")]
#[test_case(json!([]), "[]" ; "empty array")]
#[test_case(json!({}), "{}" ; "empty object")]
fn basic_types(value: Value, expected: &str) -> Result<()> {
    assert_eq!(expected, to_string(&value)?);
    Ok(())
}

