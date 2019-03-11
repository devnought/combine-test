// Many incorrect assumptions were made when creating this initially.
// See the following for a better description on the format:
// https://www.cyberciti.biz/faq/create-ssh-config-file-on-linux-unix/
// https://linux.die.net/man/5/ssh_config

use combine::{
    char::{self, space},
    many, many1,
    range::{range, take_while, take_while1},
    skip_many, skip_many1,
    stream::state::State,
    ParseError, Parser, RangeStream,
};
use std::{fs, path::PathBuf};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opts {
    pub input: PathBuf,
}

fn main() {
    let opt = Opts::from_args();
    let data = fs::read_to_string(opt.input).expect("Could not read input file");
    let res = section().easy_parse(State::new(data.as_str()));

    dbg!(res);
}

fn whitespace<'a, I>() -> impl Parser<Input = I>
where
    I: RangeStream<Item = char, Range = &'a str>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    skip_many1(space())
}

fn host_name<'a, I>() -> impl Parser<Input = I, Output = &'a str>
where
    I: RangeStream<Item = char, Range = &'a str>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        range("Host"),
        whitespace(),
        take_while(|c: char| !c.is_whitespace()),
    )
        .map(|(_, _, h)| h)
        .message("while parsing host name")
}

fn property<'a, I>() -> impl Parser<Input = I, Output = (&'a str, &'a str)>
where
    I: RangeStream<Item = char, Range = &'a str>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        take_while(|c: char| !c.is_whitespace()), // This needs to take NOT 'Host'
        whitespace(),
        take_while(|c: char| !c.is_whitespace()),
    )
        .map(|(key, _, value)| (key, value))
        .message("while parsing property")
}

fn properties<'a, I>() -> impl Parser<Input = I, Output = Vec<(&'a str, &'a str)>>
where
    I: RangeStream<Item = char, Range = &'a str>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    many(property().skip(whitespace())).message("while parsing properties")
}

fn section<'a, I>() -> impl Parser<Input = I, Output = (&'a str, Vec<(&'a str, &'a str)>)>
where
    I: RangeStream<Item = char, Range = &'a str>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (host_name(), whitespace(), properties())
        .map(|(host_name, _, properties)| (host_name, properties))
        .message("while parsing hosts")
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_property_success() {}
}
