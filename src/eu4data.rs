use combine::{many1, parser, Parser, ParserExt, sep_end_by, space, spaces, newline, satisfy, skip_many, skip_many1, token};
use combine::primitives::{State, Stream, ParseResult};

#[derive(Debug)]
pub enum Eu4Value {
    String(String),
    Table(Eu4Table),
}

impl Eu4Value {
    pub fn as_str(&self) -> &str {
        if let &Eu4Value::String(ref val) = self {
            val
        } else {
            panic!("Value is not a string!");
        }
    }
}

#[derive(Debug)]
pub struct Eu4KeyValue {
    key: String,
    value: Eu4Value,
}

#[derive(Debug)]
pub struct Eu4Table {
    values: Vec<Eu4KeyValue>,
}

fn word<I>(input: State<I>) -> ParseResult<String, I>
    where I: Stream<Item=char>
{
    let word_char = satisfy(|c: char|
        c.is_alphanumeric() || c == '.'
    );
    let word = many1::<String, _>(word_char);

    word.expected("word").parse_state(input)
}

fn key_value<I>(input: State<I>) -> ParseResult<Eu4KeyValue, I>
    where I: Stream<Item=char>
{
    let value =
        parser(word).map(|v| Eu4Value::String(v))
        .or((token('{'), parser(table), token('}')).map(|v| Eu4Value::Table(v.1)));

    let mut key_value = (parser(word), spaces(), token('='), spaces(), value)
        .map(|v| Eu4KeyValue {
            key: v.0,
            value: v.4,
        });

    key_value.parse_state(input)
}

fn nl_ws<I>(input: State<I>) -> ParseResult<(), I>
    where I: Stream<Item=char>
{
    let comment = (token('#'), skip_many(satisfy(|c| c != '\n'))).map(|_| ());;
    let mut nl_ws = space().or(newline()).map(|_| ()).or(comment);

    nl_ws.parse_state(input)
}

fn table<I>(input: State<I>) -> ParseResult<Eu4Table, I>
    where I: Stream<Item=char>
{
    let table = sep_end_by(parser(key_value), skip_many1(parser(nl_ws))).map(|v| {
            Eu4Table {
                values: v
            }
        });

    (skip_many(parser(nl_ws)), table).map(|v| v.1).parse_state(input)
}

fn eu4data<I>(input: State<I>) -> ParseResult<Eu4Table, I>
    where I: Stream<Item=char>
{
    parser(table).parse_state(input)
}

impl Eu4Table {
    pub fn parse(text: &str) -> Eu4Table {
        parser(eu4data).parse(text).unwrap().0
    }
}

#[cfg(test)]
mod tests {
    use super::{Eu4Table, Eu4Value};

    #[test]
    fn parse_value() {
        let data = Eu4Table::parse("foo=bar");
        assert_eq!(data.values.len(), 1);
        assert_eq!(data.values[0].key, "foo");
        assert_eq!(data.values[0].value.as_str(), "bar");
    }

    #[test]
    fn parse_values() {
        let data = Eu4Table::parse("foo=bar\nbar=foo");
        assert_eq!(data.values.len(), 2);
        assert_eq!(data.values[0].key, "foo");
        assert_eq!(data.values[0].value.as_str(), "bar");
        assert_eq!(data.values[1].key, "bar");
        assert_eq!(data.values[1].value.as_str(), "foo");
    }

    #[test]
    fn parse_values_inline() {
        let data = Eu4Table::parse("foo=bar bar=foo");
        assert_eq!(data.values.len(), 2);
        assert_eq!(data.values[0].key, "foo");
        assert_eq!(data.values[0].value.as_str(), "bar");
        assert_eq!(data.values[1].key, "bar");
        assert_eq!(data.values[1].value.as_str(), "foo");
    }

    #[test]
    fn parse_whitespace() {
        let data = Eu4Table::parse(" foo  = bar  ");
        assert_eq!(data.values.len(), 1);
        assert_eq!(data.values[0].key, "foo");
        assert_eq!(data.values[0].value.as_str(), "bar");
    }

    #[test]
    fn parse_comments() {
        let data = Eu4Table::parse("foo=bar #things\nbar=foo");
        assert_eq!(data.values.len(), 2);
        assert_eq!(data.values[0].key, "foo");
        assert_eq!(data.values[0].value.as_str(), "bar");
        assert_eq!(data.values[1].key, "bar");
        assert_eq!(data.values[1].value.as_str(), "foo");
    }

    #[test]
    fn parse_nested() {
        let data = Eu4Table::parse("foo={bar=chickens foobar=frogs}\ncheeze=unfrogged");
        assert_eq!(data.values.len(), 2);
        assert_eq!(data.values[1].key, "cheeze");
        assert_eq!(data.values[1].value.as_str(), "unfrogged");

        if let &Eu4Value::Table(ref table) = &data.values[0].value {
            assert_eq!(table.values.len(), 2);
            assert_eq!(table.values[0].key, "bar");
            assert_eq!(table.values[0].value.as_str(), "chickens");
            assert_eq!(table.values[1].key, "foobar");
            assert_eq!(table.values[1].value.as_str(), "frogs");
        } else {
            assert!(false, "Wrong value type!");
        }
    }
}
