#[cfg(test)]
mod tests {
    
    use crate::parser;
    use test_case::test_case;
    
    #[test_case("x", true; "Single letter")]
    #[test_case("1", false; "Single digit")]
    #[test_case("_", true; "Single underscore")]
    #[test_case("x123", true; "Starts with letter, followed by numbers")]
    #[test_case("123x", false; "Starts with number, followed by letters")]
    #[test_case("_23x3_dfsD2__", true; "Starts with number, followed by misc alphanumeric and '_'")]
    #[test_case("(raeswr", false; "Starts with invalid character")]
    #[test_case("raes)wr", false; "Starts with correct character, contains invalid character")]
    pub fn test_string_is_identifier(test_str: &str, should_pass: bool) {
        
        let result = nom::exact!(test_str, parser::identifier_parser);
        
        assert_eq!(should_pass, result.is_ok());
    }
    
    #[test_case("x123", true; "Simple identifier")]
    #[test_case("123x", true; "Simple non-identifier")]
    #[test_case("x123 a3casda2133_ dsdajd____1 _123jdas QW213 )d9sa ((123123a_b", true; "Long input")]
    pub fn test_line(test_str: &str, should_pass: bool) {
        println!("making sure this works...");
        
        println!("test_str is \"{}\"", test_str);
        
        let mut result = parser::id_nonid_parser(test_str);
        let mut rest: &str;
        let mut found = match result {
            Ok(r) => {
                rest = r.0;
                r.1
            },
            Err(_) => {
                rest = "error";
                "error"
            }
        };
        
        let mut whitespace = true;
        
        while rest != "" {
            if whitespace {
                result = parser::whitespace_parser(rest);
            }
            else {
                result = parser::id_nonid_parser(rest);
            }
            found = match result {
                Ok(r) => {
                    rest = r.0;
                    r.1
                },
                Err(_) => {
                    rest = "";
                    "error"
                }
            };
            
            if found == "error" {
                break;
            }
            
            whitespace = !whitespace;
        }
        assert_eq!(should_pass, found != "error");
    }
}


