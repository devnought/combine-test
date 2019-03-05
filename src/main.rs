use combine::{
    char::{self, letter, space},
    many1, satisfy, skip_many, skip_many1, ParseError, Parser, Stream,
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

    let result = config().easy_parse(data);

    println!("{:#?}", result);
}

#[derive(Debug, PartialEq)]
struct Section {
    host: String,
    values: HashMap<String, String>,
}

/*fn comment_whitespace<T>() -> impl Parser<Input = T>
where
    T: Stream<Item = char>,
    T::Error: ParseError<T::Item, T::Range, T::Position>,
{
    let comment = (token('#'), skip_many(satisfy(|c| c != '\n' && c != '\r'))).map(|_| ());

    // Wrap the `spaces().or(comment)` in `skip_many` so that it skips alternating whitespace and
    // comments
    skip_many(skip_many1(space()).or(comment))
}*/

fn property<T>() -> impl Parser<Input = T, Output = (String, String)>
where
    T: Stream<Item = char>,
    T::Error: ParseError<T::Item, T::Range, T::Position>,
{
    (
        skip_many1(space()),
        many1(letter()),
        skip_many1(space()),
        many1(satisfy(|c| c != '\n' && c != '\r')),
    )
        .map(|(_, key, _, value)| (key, value))
        .message("while parsing property")
}

fn section<T>() -> impl Parser<Input = T, Output = Section>
where
    T: Stream<Item = char>,
    T::Error: ParseError<T::Item, T::Range, T::Position>,
{
    (
        char::string("Host"), // Change this to range::range("Host")
        skip_many1(space()),
        many1(satisfy(|c: char| c.is_alphanumeric() || c == '-')),
        many1(property()),
    )
        .map(|(_, _, host, values)| Section { host, values })
        .message("while parsing section")
}

fn config<T>() -> impl Parser<Input = T, Output = Vec<Section>>
where
    T: Stream<Item = char>,
    T::Error: ParseError<T::Item, T::Range, T::Position>,
{
    (skip_many(space()), many1(section()))
        .map(|(_, s)| s)
        .message("while parsing config")
}

/*combine::parser::parser! {
    fn config[T]()(T) -> Expr
    where[T: Stream<Item = char>]
    {
        section()
    }
}*/
