// Many incorrect assumptions were made when creating this initially.
// See the following for a better description on the format:
// https://www.cyberciti.biz/faq/create-ssh-config-file-on-linux-unix/

use combine::{
    char::{self, space},
    many, many1, satisfy, skip_many, skip_many1, token, ParseError, Parser, Stream,
};
use std::collections::HashMap;

fn main() {
    // TODO: TEST WITH COMMENTS
    let data = r#"Host dev
    HostName dev.example.com
    Port 22000
    User fooey
Host github-project1
    User git
    HostName github.com
    IdentityFile ~/.ssh/github.project1.key


Host github-org
    User git
    HostName github.com
    IdentityFile ~/.ssh/github.org.key
Host github.com
    User git
    IdentityFile ~/.ssh/github.key

Host tunnel
    HostName database.example.com
    IdentityFile ~/.ssh/coolio.example.key
    LocalForward 9906 127.0.0.1:3306
    User coolio
"#;

    let result = sections().easy_parse(data);

    println!("{:#?}", result);
}

#[derive(Debug, PartialEq)]
struct Section {
    host: String,
    values: HashMap<String, String>,
}

fn whitespace<I>() -> impl Parser<Input = I>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let comment = (token('#'), skip_many(satisfy(|c| c != '\n'))).map(|_| ());
    // Wrap the `spaces().or(comment)` in `skip_many` so that it skips alternating whitespace and
    // comments
    skip_many(skip_many1(space()).or(comment))
}

fn parse_value<I>() -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    many1(satisfy(|c| c != '\n' && c != '#')).message("while parsing value")
}

fn parse_key<I>() -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    many1(satisfy(|c: char| !c.is_whitespace())).message("while parsing key")
}

fn property<T>() -> impl Parser<Input = T, Output = (String, String)>
where
    T: Stream<Item = char>,
    T::Error: ParseError<T::Item, T::Range, T::Position>,
{
    (
        skip_many(space()),
        parse_key(),
        skip_many1(space()),
        parse_value(),
    )
        .map(|(_, key, _, value)| (key, value))
        .message("while parsing property")
}

fn properties<I>() -> impl Parser<Input = I, Output = HashMap<String, String>>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    // After each property we skip any whitespace that followed it
    many(property().skip(whitespace())).message("while parsing properties")
}

fn section<T>() -> impl Parser<Input = T, Output = Section>
where
    T: Stream<Item = char>,
    T::Error: ParseError<T::Item, T::Range, T::Position>,
{
    (
        char::string("Host"), // Change this to range::range("Host")
        skip_many1(space()),
        parse_value(),
        properties(),
    )
        .map(|(_, _, host, values)| Section { host, values })
        .message("while parsing section")
}

fn sections<T>() -> impl Parser<Input = T, Output = Vec<Section>>
where
    T: Stream<Item = char>,
    T::Error: ParseError<T::Item, T::Range, T::Position>,
{
    many1(section()).message("while parsing sections")
}

/*combine::parser::parser! {
    fn sections[T]()(T) -> Expr
    where[T: Stream<Item = char>]
    {
        section()
    }
}*/

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn parse_property_success() {
        // Arrange
        let data = "LocalForward      9906 127.0.0.1:3306";
        let expected = Ok((
            (
                String::from("LocalForward"),
                String::from("9906 127.0.0.1:3306"),
            ),
            "",
        ));

        // Act
        let actual = property().easy_parse(data);

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_properties_success() {
        // Arrange
        let data = r#"LocalForward    9906 127.0.0.1:3306
   HostName github.com"#;

        let expected_map = [
            (
                String::from("LocalForward"),
                String::from("9906 127.0.0.1:3306"),
            ),
            (String::from("HostName"), String::from("github.com")),
        ]
        .iter()
        .cloned()
        .collect();

        let expected = Ok((expected_map, ""));

        // Act
        let actual = properties().easy_parse(data);

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_section_with_remainder_success() {
        // Arrange
        let data = r#"Host github-org
    User git
    HostName github.com
    IdentityFile ~/.ssh/github.org.key
Host tunnel"#;

        let expected_section = Section {
            host: String::from("github-org"),
            values: [
                (String::from("User"), String::from("git")),
                (String::from("HostName"), String::from("github.com")),
                (
                    String::from("IdentityFile"),
                    String::from("~/.ssh/github.org.key"),
                ),
            ]
            .iter()
            .cloned()
            .collect(),
        };

        let expected = Ok((expected_section, "Host tunnel"));

        // Act
        let actual = section().easy_parse(data);

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_section_without_remainder_success() {
        // Arrange
        let data = r#"Host github-org
    User git
    HostName github.com
    IdentityFile ~/.ssh/github.org.key"#;

        let expected_section = Section {
            host: String::from("github-org"),
            values: [
                (String::from("User"), String::from("git")),
                (String::from("HostName"), String::from("github.com")),
                (
                    String::from("IdentityFile"),
                    String::from("~/.ssh/github.org.key"),
                ),
            ]
            .iter()
            .cloned()
            .collect(),
        };

        let expected = Ok((expected_section, ""));

        // Act
        let actual = section().easy_parse(data);

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_sections_success() {
        // Arrange
        let data = r#"Host github-org
    User git
    HostName github.com
    IdentityFile ~/.ssh/github.org.key
Host tunnel
    HostNameZ database.example.com
    IdentityFileZ ~/.ssh/coolio.example.key
    LocalForwardZ 9906 127.0.0.1:3306
    UserZ coolio"#;

        let expected_section0 = Section {
            host: String::from("github-org"),
            values: [
                (String::from("User"), String::from("git")),
                (String::from("HostName"), String::from("github.com")),
                (
                    String::from("IdentityFile"),
                    String::from("~/.ssh/github.org.key"),
                ),
            ]
            .iter()
            .cloned()
            .collect(),
        };

        let expected_section1 = Section {
            host: String::from("tunnel"),
            values: [
                (
                    String::from("HostNameZ"),
                    String::from("database.example.com"),
                ),
                (
                    String::from("IdentityFileZ"),
                    String::from("~/.ssh/coolio.example.key"),
                ),
                (
                    String::from("LocalForwardZ"),
                    String::from("9906 127.0.0.1:3306"),
                ),
                (String::from("UserZ"), String::from("coolio")),
            ]
            .iter()
            .cloned()
            .collect(),
        };

        let expected = Ok((vec![expected_section0, expected_section1], ""));

        // Act
        let actual = sections().easy_parse(data);
        dbg!(&actual);

        // Assert
        assert_eq!(expected, actual);
    }
}
