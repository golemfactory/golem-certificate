use anyhow::Result;
use serde_json::ser::Formatter;
use test_case::test_case;

use jcs::*;

#[test_case(0x0000000000000000, "0" ; "Zero")]
#[test_case(0x8000000000000000, "0" ; "Minus zero")]
#[test_case(0x0000000000000001, "5e-324" ; "Min pos number")]
#[test_case(0x8000000000000001, "-5e-324" ; "Min neg number")]
#[test_case(0x7fefffffffffffff, "1.7976931348623157e+308" ; "Max pos number")]
#[test_case(0xffefffffffffffff, "-1.7976931348623157e+308" ; "Max neg number")]
#[test_case(0x4340000000000000, "9007199254740992" ; "Max pos int")]
#[test_case(0xc340000000000000, "-9007199254740992" ; "Max neg int")]
#[test_case(0x4430000000000000, "295147905179352830000" ; "~2**68")]
#[test_case(0x44b52d02c7e14af5, "9.999999999999997e+22" ; "example 1")]
#[test_case(0x44b52d02c7e14af6, "1e+23" ; "example 2")]
#[test_case(0x44b52d02c7e14af7, "1.0000000000000001e+23" ; "example 3")]
#[test_case(0x444b1ae4d6e2ef4e, "999999999999999700000" ; "example 4")]
#[test_case(0x444b1ae4d6e2ef4f, "999999999999999900000" ; "example 5")]
#[test_case(0x444b1ae4d6e2ef50, "1e+21" ; "example 6")]
#[test_case(0x3eb0c6f7a0b5ed8c, "9.999999999999997e-7" ; "example 7")]
#[test_case(0x3eb0c6f7a0b5ed8d, "0.000001" ; "example 8")]
#[test_case(0x41b3de4355555553, "333333333.3333332" ; "example 9")]
#[test_case(0x41b3de4355555554, "333333333.33333325" ; "example 10")]
#[test_case(0x41b3de4355555555, "333333333.3333333" ; "example 11")]
#[test_case(0x41b3de4355555556, "333333333.3333334" ; "example 12")]
#[test_case(0x41b3de4355555557, "333333333.33333343" ; "example 13")]
#[test_case(0xbecbf647612f3696, "-0.0000033333333333333333" ; "example 14")]
#[test_case(0x43143ff3c1cb0959, "1424953923781206.2" ; "round to even")]
fn jcs_rfc_appendix_b(bits: u64, expected: &str) -> Result<()> {
    let number: f64 = f64::from_bits(bits);
    let mut buffer = Vec::new();
    JcsFormatter::new().write_f64(&mut buffer, number)?;
    let serialized = String::from_utf8(buffer)?;

    assert_eq!(expected, serialized);
    Ok(())
}

#[test_case(f64::NAN ; "NaN")]
#[test_case(f64::INFINITY ; "Infinity")]
#[test_case(f64::NEG_INFINITY ; "-Infinity")]
fn infinite_numbers(number: f64) {
    let mut buffer = Vec::new();
    assert!(JcsFormatter::new().write_f64(&mut buffer, number).is_err());
}

#[test_case(u8::MAX as f64, "255" ; "u8::MAX exact")]
#[test_case(i8::MAX as f64, "127" ; "i8::MAX exact")]
#[test_case(i8::MIN as f64, "-128" ; "i8::MIN exact")]
#[test_case(u16::MAX as f64, "65535" ; "u16::MAX exact")]
#[test_case(i16::MAX as f64, "32767" ; "i16::MAX exact")]
#[test_case(i16::MIN as f64, "-32768" ; "i16::MIN exact")]
#[test_case(u32::MAX as f64, "4294967295" ; "u32::MAX exact")]
#[test_case(i32::MAX as f64, "2147483647" ; "i32::MAX exact")]
#[test_case(i32::MIN as f64, "-2147483648" ; "i32::MIN exact")]
// The RFC states that "For maximum compliance with the ECMAScript "JSON"
// object, values that are to be interpreted as true integers SHOULD be in
// the range -9007199254740991 to 9007199254740991. However, how numbers are
// used in applications does not affect the JCS algorithm."
// We check that our implementation matches with what v8 JSON.stringify does
// as the most widely used javascript engine.
#[test_case(u64::MAX as f64, "18446744073709552000" ; "u64::MAX exact (v8 compat)")]
#[test_case(i64::MAX as f64, "9223372036854776000" ; "i64::MAX exact (v8 compat)")]
#[test_case(i64::MIN as f64, "-9223372036854776000" ; "i64::MIN exact (v8 compat)")]
#[test_case(184467440737095520000u128 as f64, "184467440737095500000" ; "v8 compat rounding 1")]
#[test_case(1844674407370955200000u128 as f64, "1.8446744073709552e+21" ; "v8 compat rounding 2")]
#[test_case(184467440737095520000000u128 as f64, "1.844674407370955e+23" ; "v8 compat rounding 3")]
fn integers(number: f64, expected: &str) -> Result<()> {
    let mut buffer = Vec::new();
    JcsFormatter::new().write_f64(&mut buffer, number)?;
    let serialized = String::from_utf8(buffer)?;

    assert_eq!(expected, serialized);
    Ok(())
}

#[test_case(u8::MAX, "255" ; "u8::MAX exact")]
fn integers_u8(number: u8, expected: &str) -> Result<()> {
    let mut buffer = Vec::new();
    JcsFormatter::new().write_u8(&mut buffer, number)?;
    let serialized = String::from_utf8(buffer)?;

    assert_eq!(expected, serialized);
    Ok(())
}

#[test_case(i8::MAX, "127" ; "i8::MAX exact")]
#[test_case(i8::MIN, "-128" ; "i8::MIN exact")]
fn integers_i8(number: i8, expected: &str) -> Result<()> {
    let mut buffer = Vec::new();
    JcsFormatter::new().write_i8(&mut buffer, number)?;
    let serialized = String::from_utf8(buffer)?;

    assert_eq!(expected, serialized);
    Ok(())
}

#[test_case(u16::MAX, "65535" ; "u16::MAX exact")]
fn integers_u16(number: u16, expected: &str) -> Result<()> {
    let mut buffer = Vec::new();
    JcsFormatter::new().write_u16(&mut buffer, number)?;
    let serialized = String::from_utf8(buffer)?;

    assert_eq!(expected, serialized);
    Ok(())
}

#[test_case(i16::MAX, "32767" ; "i16::MAX exact")]
#[test_case(i16::MIN, "-32768" ; "i16::MIN exact")]
fn integers_i16(number: i16, expected: &str) -> Result<()> {
    let mut buffer = Vec::new();
    JcsFormatter::new().write_i16(&mut buffer, number)?;
    let serialized = String::from_utf8(buffer)?;

    assert_eq!(expected, serialized);
    Ok(())
}

#[test_case(u32::MAX, "4294967295" ; "u32::MAX exact")]
fn integers_u32(number: u32, expected: &str) -> Result<()> {
    let mut buffer = Vec::new();
    JcsFormatter::new().write_u32(&mut buffer, number)?;
    let serialized = String::from_utf8(buffer)?;

    assert_eq!(expected, serialized);
    Ok(())
}

#[test_case(i32::MAX, "2147483647" ; "i32::MAX exact")]
#[test_case(i32::MIN, "-2147483648" ; "i32::MIN exact")]
fn integers_i32(number: i32, expected: &str) -> Result<()> {
    let mut buffer = Vec::new();
    JcsFormatter::new().write_i32(&mut buffer, number)?;
    let serialized = String::from_utf8(buffer)?;

    assert_eq!(expected, serialized);
    Ok(())
}

// The RFC states that "For maximum compliance with the ECMAScript "JSON"
// object, values that are to be interpreted as true integers SHOULD be in
// the range -9007199254740991 to 9007199254740991. However, how numbers are
// used in applications does not affect the JCS algorithm."
// We check that our implementation matches with what v8 JSON.stringify does
// as the most widely used javascript engine.
#[test_case(u64::MAX, "18446744073709552000" ; "u64::MAX exact (v8 compat)")]
fn integers_u64(number: u64, expected: &str) -> Result<()> {
    let mut buffer = Vec::new();
    JcsFormatter::new().write_u64(&mut buffer, number)?;
    let serialized = String::from_utf8(buffer)?;

    assert_eq!(expected, serialized);
    Ok(())
}

#[test_case(i64::MAX, "9223372036854776000" ; "i64::MAX exact (v8 compat)")]
#[test_case(i64::MIN, "-9223372036854776000" ; "i64::MIN exact (v8 compat)")]
fn integers_i64(number: i64, expected: &str) -> Result<()> {
    let mut buffer = Vec::new();
    JcsFormatter::new().write_i64(&mut buffer, number)?;
    let serialized = String::from_utf8(buffer)?;

    assert_eq!(expected, serialized);
    Ok(())
}

#[test_case(184467440737095520000u128, "184467440737095500000" ; "v8 compat 1")]
#[test_case(1844674407370955200000u128, "1.8446744073709552e+21" ; "v8 compat 2")]
#[test_case(184467440737095520000000u128, "1.844674407370955e+23" ; "v8 compat 3")]
fn integers_u128(number: u128, expected: &str) -> Result<()> {
    let mut buffer = Vec::new();
    JcsFormatter::new().write_u128(&mut buffer, number)?;
    let serialized = String::from_utf8(buffer)?;

    assert_eq!(expected, serialized);
    Ok(())
}

#[test_case(-184467440737095520000i128, "-184467440737095500000" ; "v8 compat 1")]
#[test_case(-1844674407370955200000i128, "-1.8446744073709552e+21" ; "v8 compat 2")]
#[test_case(-184467440737095520000000i128, "-1.844674407370955e+23" ; "v8 compat 3")]
fn integers_i128(number: i128, expected: &str) -> Result<()> {
    let mut buffer = Vec::new();
    JcsFormatter::new().write_i128(&mut buffer, number)?;
    let serialized = String::from_utf8(buffer)?;

    assert_eq!(expected, serialized);
    Ok(())
}

#[test_case(0.0f64, "0" ; "zero")]
#[test_case(-0.0f64, "0" ; "minus zero")]
fn zeroes(number: f64, expected: &str) -> Result<()> {
    let mut buffer = Vec::new();
    JcsFormatter::new().write_f64(&mut buffer, number)?;
    let serialized = String::from_utf8(buffer)?;

    assert_eq!(expected, serialized);
    Ok(())
}


// This test is running on a generated file, the generator is copied from the reference implementation
// https://github.com/cyberphone/json-canonicalization/blob/dc406ceaf94b5fa554fcabb92c091089c2357e83/testdata/numgen.js
// To run the test, generate the input file by executing the generation script
// in `resources/generated-numbers` by the command line `node numgen.js` (requires nodejs to be installed)
// This is a very long test and should be executed via
// `cargo test --tests generated_numbers -- --nocapture --include-ignored`
#[ignore]
#[test]
fn generated_numbers() -> Result<()> {
    use std::io::{ BufRead, BufReader, Write, stdout };
    let file = std::fs::File::open("tests/resources/generated-numbers/es6testfile100m.txt")?;
    let reader = BufReader::new(file);

    let one_percent = 1000000usize;
    let mut threshold = one_percent - 1;
    let mut percent = 0;
    let mut stdout = stdout().lock();
    let mut buffer: Vec<u8> = Vec::with_capacity(32);
    let mut formatter = JcsFormatter::new();
    for (idx, line) in reader.lines().enumerate() {
        let line = line?;
        let parts = line.split(',').collect::<Vec<_>>();
        assert_eq!(2, parts.len());
        let input = f64::from_bits(u64::from_str_radix(parts[0], 16)?);
        formatter.write_f64(&mut buffer, input)?;
        assert_eq!(parts[1].as_bytes(), &buffer, "Testing input {} parsed into {}", parts[0], input);
        buffer.clear();
        if idx == threshold {
            threshold += one_percent;
            percent += 1;
            if percent % 10 == 0 {
                write!(stdout, "{percent}%").ok();
            } else {
                stdout.write(b".").ok();
            }
            stdout.flush().ok();
        }
    }
    stdout.write(b"\n").ok();
    Ok(())
}