use super::*;

#[test]
pub fn i32_literal_parser() {
    assert!(parser::I32LiteralParser::new().parse("1").is_ok());
    assert!(parser::I32LiteralParser::new().parse("1234567890").is_ok());
    assert!(parser::I32LiteralParser::new().parse("x").is_err());
    assert!(parser::I32LiteralParser::new().parse("123i").is_ok());
    assert!(parser::I32LiteralParser::new().parse("123u").is_err());
    assert!(parser::I32LiteralParser::new().parse("123x").is_err());
    assert!(parser::I32LiteralParser::new().parse("123uu").is_err());
    assert!(parser::I32LiteralParser::new().parse("12.3").is_err());
}

#[test]
pub fn u32_literal_parser() {
    assert!(parser::U32LiteralParser::new().parse("1").is_err());
    assert!(parser::U32LiteralParser::new().parse("12345678901234567890").is_err());
    assert!(parser::U32LiteralParser::new().parse("x").is_err());
    assert!(parser::U32LiteralParser::new().parse("123i").is_err());
    assert!(parser::U32LiteralParser::new().parse("123u").is_ok());
    assert!(parser::U32LiteralParser::new().parse("123x").is_err());
    assert!(parser::U32LiteralParser::new().parse("123uu").is_err());
    assert!(parser::U32LiteralParser::new().parse("12.3").is_err());
}
