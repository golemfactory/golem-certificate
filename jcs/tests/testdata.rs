use anyhow::Result;
use serde_json::Value;

use jcs::*;

fn read_files(filename: &str) -> Result<(Value, String, Vec<u8>)> {
    let resources_folder = std::path::PathBuf::from("tests/resources/testdata");
    let get_path = |directory: &str, extension: &str| {
        let mut filepath = resources_folder.clone();
        filepath.push(directory);
        filepath.push(filename);
        filepath.set_extension(extension);
        filepath
    };
    let input_file = get_path("input", "json");
    let expected_string_file = get_path("output", "json");
    let expected_vec_file = get_path("outhex", "txt");

    let input_bytes = std::fs::read(input_file)?;
    let input = serde_json::from_slice(&input_bytes)?;
    let expected_string = std::fs::read_to_string(expected_string_file)?;
    let expected_vec = std::fs::read_to_string(expected_vec_file)?
        .split_whitespace()
        .map(|hex| u8::from_str_radix(hex, 16))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((input, expected_string, expected_vec))
}

include!(concat!(env!("OUT_DIR"), "/testdata_tests.rs"));
