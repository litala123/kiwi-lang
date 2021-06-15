use nom;

pub fn id_nonid_parser(i: &str) -> nom::IResult<&str, &str> {
    nom::branch::alt((identifier_parser, non_identifier_parser))(i)
}

pub fn identifier_parser(i: &str) -> nom::IResult<&str, &str> {
    println!("in ID parser................");
    nom::combinator::recognize(
        nom::sequence::pair(
            nom::branch::alt((nom::character::complete::alpha1, nom::bytes::complete::tag("_"))),
            nom::multi::many0(
                nom::branch::alt((nom::character::complete::alphanumeric1, nom::bytes::complete::tag("_")))
            )
        )
    )(i)
}

pub fn non_identifier_parser(i: &str) -> nom::IResult<&str, &str> {
    println!("in non ID parser||||||||||||");
    nom::combinator::recognize(
        nom::sequence::pair(
            nom::character::complete::none_of("QWERTYUIOPASDFGHJKLZXCVBNMqwertyuiopasdfghjklzxcvbnm_"),
            nom::multi::many0(
                nom::character::complete::none_of(" \t\r\n")
            )
        )
    )(i)
}

pub fn whitespace_maybe_parser(i: &str) -> nom::IResult<&str, &str> {
    nom::character::complete::multispace0(i)
}

pub fn whitespace_parser(i: &str) -> nom::IResult<&str, &str> {
    nom::character::complete::multispace1(i)
}
