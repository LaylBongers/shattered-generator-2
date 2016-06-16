extern crate combine;

use combine::{many, many1, parser, Parser, ParserExt, space, spaces, newline, satisfy, skip_many, token, string, any, unexpected, between, try};
use combine::primitives::{State, Stream, ParseResult};

#[derive(Debug, Clone)]
pub enum Eu4Value {
    String(String),
    Table(Eu4Table),
    Array(Vec<Eu4Value>)
}

impl Eu4Value {
    pub fn color(r: u8, g: u8, b: u8) -> Self {
        Eu4Value::Array(vec![
            Eu4Value::String(r.to_string()),
            Eu4Value::String(g.to_string()),
            Eu4Value::String(b.to_string())
        ])
    }

    pub fn as_str(&self) -> &str {
        if let &Eu4Value::String(ref val) = self {
            val
        } else {
            panic!("Value is not a string!");
        }
    }
}

#[derive(Debug, Clone)]
pub struct Eu4KeyValue {
    pub key: String,
    pub value: Eu4Value,
}

#[derive(Debug, Clone)]
pub struct Eu4Table {
    pub values: Vec<Eu4KeyValue>,
}

fn nl_ws<I>(input: State<I>) -> ParseResult<(), I>
    where I: Stream<Item=char>
{
    let comment = (token('#'), skip_many(satisfy(|c| c != '\n'))).map(|_| ());;
    let mut nl_ws = space().or(newline()).map(|_| ()).or(comment);

    nl_ws.parse_state(input)
}

fn word<I>(input: State<I>) -> ParseResult<String, I>
    where I: Stream<Item=char>
{
    let word_char = satisfy(|c: char|
        c.is_alphanumeric() || c == '.' || c == '_' || c == '-'
    );
    let word = many1::<String, _>(word_char);

    word.expected("word").parse_state(input)
}

fn escape_char(c: char) -> char {
    match c {
        '\'' => '\'',
        '"' => '"',
        '\\' => '\\',
        '/' => '/',
        'b' => '\u{0008}',
        'f' => '\u{000c}',
        'n' => '\n',
        'r' => '\r',
        't' => '\t',
        c => c,//Should never happen
    }
}

fn string_char<I>(input: State<I>) -> ParseResult<char, I>
    where I: Stream<Item=char>
{
    let (c, input) = try!(any().parse_lazy(input));
    let mut back_slash_char = satisfy(|c| "\"\\/bfnrt".chars().find(|x| *x == c).is_some())
                                 .map(escape_char);
    match c {
        '\\' => input.combine(|input| back_slash_char.parse_state(input)),
        '"' => unexpected("\"").parse_state(input.into_inner()).map(|_| unreachable!()),
        _ => Ok((c, input)),
    }
}

fn string_literal<I>(input: State<I>) -> ParseResult<String, I>
    where I: Stream<Item=char>
{
    let literal = between(
        string("\""),
        string("\""),
        many(parser(string_char))
    ).map(|v| v);

    literal.expected("string literal").parse_state(input)
}

fn value<I>(input: State<I>) -> ParseResult<Eu4Value, I>
    where I: Stream<Item=char>
{
    let value =
        parser(word)
            .map(|v| Eu4Value::String(v))
        .or(parser(string_literal)
            .map(|v| Eu4Value::String(v)))
        .or((token('{'), parser(table), token('}'))
            .map(|v| {
                let table = v.1;

                // Devolve table to array if keyless
                // TODO: Perhaps instead use try() to attempt to parse a table first, then an array
                if table.values.iter().all(|v| v.key == "") {
                    Eu4Value::Array(table.values.into_iter().map(|v| v.value).collect())
                } else {
                    Eu4Value::Table(table)
                }
            }));

    value.expected("value").parse_state(input)
}

fn key_value<I>(input: State<I>) -> ParseResult<Eu4KeyValue, I>
    where I: Stream<Item=char>
{
    let key_value = (parser(word), spaces(), token('='), spaces(), parser(value))
        .map(|v| Eu4KeyValue {
            key: v.0,
            value: v.4,
        });

    key_value.expected("key-value").parse_state(input)
}

fn keyless_value<I>(input: State<I>) -> ParseResult<Eu4KeyValue, I>
    where I: Stream<Item=char>
{
    let key_value = parser(value)
        .map(|v| Eu4KeyValue {
            key: "".into(),
            value: v,
        });

    key_value.expected("keyless value").parse_state(input)
}

fn table<I>(input: State<I>) -> ParseResult<Eu4Table, I>
    where I: Stream<Item=char>
{
    let table = many(try(parser(key_value)).or(parser(keyless_value)).skip(skip_many(parser(nl_ws))))
        .map(|v| {
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

fn escape_str(text: &str) -> String {
    let mut target = String::new();

    target.push('\"');
    for c in text.chars() {
        match c {
            '\\' => target.push_str("\\\\"),
            _ => target.push(c)
        };
    }
    target.push('\"');

    target
}

fn escape_str_if_needed(text: &str) -> String {
    if text.chars().any(|c| c == '\\' || c == ' ') {
        escape_str(text)
    } else {
        text.into()
    }
}

impl Eu4Table {
    pub fn new() -> Self {
        Eu4Table {
            values: Vec::new()
        }
    }

    pub fn parse(text: &str) -> Eu4Table {
        parser(eu4data).parse(text).unwrap().0
    }

    pub fn serialize(&self) -> String {
        let mut target = String::new();

        for key_value in &self.values {
            // Serialize the key if we have one
            if key_value.key != "" {
                target.push_str(&escape_str_if_needed(&key_value.key));
                target.push_str(" = ");
            }

            // Serialize the value
            key_value.value.serialize_to(&mut target);
        }

        target
    }

    pub fn get(&self, key: &str) -> Option<&Eu4Value> {
        self.values.iter().find(|v| v.key == key).map(|v| &v.value)
    }

    pub fn set(&mut self, key: &str, value: Eu4Value) {
        // Check if a value already exists with this key
        if let Some(ref mut entry) = self.values.iter_mut().find(|v| v.key == key) {
            // It does, overwrite it
            entry.value = value;
            return; // < Can't use else, borrow checking complains
        }

        // It doesn't, add it
        self.values.push(Eu4KeyValue { key: key.into(), value: value });
    }
}

impl Eu4Value {
    fn serialize_to(&self, target: &mut String) {
        match self {
            &Eu4Value::String(ref v) => {
                target.push_str(&escape_str_if_needed(v));
                target.push('\n');
            },
            &Eu4Value::Table(ref t) => {
                target.push_str("{\n");
                target.push_str(&t.serialize());
                target.push_str("}\n");
            },
            &Eu4Value::Array(ref a) => {
                target.push_str("{\n");
                for val in a {
                    val.serialize_to(target);
                }
                target.push_str("}\n");
            }
        }
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
    fn parse_quoted() {
        let data = Eu4Table::parse("foo=\"I'm a little teapot\"");
        assert_eq!(data.values.len(), 1);
        assert_eq!(data.values[0].key, "foo");
        assert_eq!(data.values[0].value.as_str(), "I'm a little teapot");

        let data = Eu4Table::parse(r#"foo="I'm a little teapot \"short and stout\"""#);
        assert_eq!(data.values.len(), 1);
        assert_eq!(data.values[0].key, "foo");
        assert_eq!(data.values[0].value.as_str(), "I'm a little teapot \"short and stout\"");
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

    #[test]
    fn parse_annoying_nested() {
        let data = Eu4Table::parse("foo={bar=chickens foobar=frogs}cheeze=unfrogged");
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

    #[test]
    fn parse_array() {
        let data = Eu4Table::parse("foo={why \"does this\" exist}");
        assert_eq!(data.values.len(), 1);
        assert_eq!(data.values[0].key, "foo");

        if let &Eu4Value::Array(ref array) = &data.values[0].value {
            assert_eq!(array.len(), 3);
            assert_eq!(array[0].as_str(), "why");
            assert_eq!(array[1].as_str(), "does this");
            assert_eq!(array[2].as_str(), "exist");
        }
    }
}
