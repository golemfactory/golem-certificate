use std::env;
use std::fs::read_dir;
use std::fs::File;
use std::io::{Error, ErrorKind, Result, Write};
use std::path::Path;

// Generate test cases for all files in tests/resources/testdata/input
fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let destination = Path::new(&out_dir).join("testdata_tests.rs");
    let mut test_file = File::create(&destination).unwrap();

    let test_data = read_dir("tests/resources/testdata/input")?;

    for file in test_data {
        let file = file?;
        if file.file_type()?.is_file()
            && file.path().extension().and_then(|ext| ext.to_str()) == Some("json")
        {
            let path = file.path();
            let filename = path
                .file_stem()
                .ok_or_else(|| Error::new(ErrorKind::NotFound, "Empty filename"))?
                .to_str()
                .ok_or_else(|| {
                    Error::new(ErrorKind::NotFound, "Unsupported characters in filename")
                })?;
            write_test(&mut test_file, filename)?;
        }
    }
    println!("cargo:rerun-if-changed=tests/resources/testdata/input");
    Ok(())
}

fn write_test(test_file: &mut File, filename: &str) -> Result<()> {
    write!(
        test_file,
        r#"
        #[test]
        fn testdata_{filename}() -> Result<()> {{
            let (input, expected_string, expected_vec) = read_files("{filename}")?;
            assert_eq!(expected_string, to_string(&input)?);
            assert_eq!(expected_vec, to_vec(&input)?);
            Ok(())
        }}

        "#,
        filename = filename,
    )
}
