use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_until, take_until1, take_while, take_while_m_n},
    character::complete::{
        alphanumeric1, anychar, char, multispace0, multispace1, newline, none_of, one_of, space0,
    },
    combinator::{consumed, cut, eof, fail, not, opt, peek, recognize, rest, success},
    error::context,
    multi::{many0_count, many1_count},
    sequence::{delimited, preceded, terminated},
    Parser as _,
};
use nom_language::error::VerboseError;

use crate::{
    executor::{Action, Command, InType, OutType},
    nom_ext::FnParser,
    MdPiece, BEGIN_MDSH, END_MDSH,
};

/// Trait alias, sort of like
/// ```future_rust
/// type Parser<'a, T> = nom::Parser<
///     'a,
///     &'a str,
///     Output = T,
///     Error = VerboseError<&'a str>
/// >;
/// ```
pub trait Parser<'a, T>: nom::Parser<&'a str, Output = T, Error = VerboseError<&'a str>> {}

impl<'a, T, X: nom::Parser<&'a str, Output = T, Error = VerboseError<&'a str>>> Parser<'a, T>
    for X
{
}

// pub type IRes<I, O> = nom::IResult<I, O, VerboseError<I>>;

pub fn markdown_piece<'a>() -> impl Parser<'a, MdPiece<'a>> {
    alt((
        FencedBlockParser.map(|_| MdPiece::FencedBlock),
        action_with_source().map(MdPiece::Action),
        preceded(tag(BEGIN_MDSH), fail()),
        comment().map(MdPiece::RawLine),
        non_actionable_code_block().map(MdPiece::RawLine),
        recognize(context("raw line", (take_until("\n"), newline))).map(MdPiece::RawLine),
    ))
}

pub type ActionWithSource<'a> = (&'a str, Action<'a>);

fn action_with_source<'a>() -> impl Parser<'a, ActionWithSource<'a>> {
    consumed((
        alt((
            actionable_code_block(),
            inline_code(),
            link(),
            actionable_comment(),
        )),
        opt(transition_line()),
    ))
    .map(|(source, (mut action, transition))| {
        if action.transition.is_none() {
            action.transition = transition;
        }
        (source, action)
    })
}

/// Need this to avoid "recursive opaque type" error.
/// Recursive definition to allow included markdown code
/// to be also processable by mdsh
struct FencedBlockParser;

impl<'a> nom::Parser<&'a str> for FencedBlockParser {
    type Output = ();
    type Error = VerboseError<&'a str>;

    fn process<OM: nom::OutputMode>(
        &mut self,
        input: &'a str,
    ) -> nom::PResult<OM, &'a str, Self::Output, Self::Error> {
        context(
            "fenced block",
            delimited(
                (tag(BEGIN_MDSH), newline),
                recognize(
                    // markdown_piece(), // TODO
                    many0_count(not(tag(BEGIN_MDSH).or(tag(END_MDSH))).and(anychar))
                        .and(alt((peek(tag(END_MDSH)), recognize(Self)))),
                ),
                cut(tag(END_MDSH)
                    .and(space0)
                    .and(recognize(newline).or(eof))
                    .and(multispace0)),
            ),
        )
        .map(|_| ())
        .process::<OM>(input)
    }
}

/// Link container:
/// ```md
/// [> yaml < yaml example](./sample.yaml)`
/// ```
fn link<'a>() -> impl Parser<'a, Action<'a>> {
    context(
        "link",
        (
            char('['),
            command(),
            not(char('[')),
            cut((take_until("]"), tag("]("))),
            cut(filepath()),
            cut((char(')'), newline)),
        ),
    )
    .map(|(_, command, _, (description, _), filepath, _)| {
        let transition = description
            .split_once(" :: ")
            .map(|(_, t)| t.trim_end());
        Action {
            command,
            data_line: Some(filepath),
            data: None,
            transition,
        }
    })
}

fn out_type<'a>() -> impl Parser<'a, OutType<'a>> {
    context(
        "output type",
        alt((
            (char('>'), space0, filepath()).map(|x| OutType::CodeBlock(x.2)),
            (char('>')).map(|_| OutType::Markdown),
            (char('!')).map(|_| OutType::Environment),
        )),
    )
}

fn filepath<'a>() -> impl Parser<'a, &'a str> {
    context(
        "filepath",
        recognize(many1_count(alt((
            recognize(alphanumeric1),
            recognize(one_of("/._-")),
        )))),
    )
}

fn in_type<'a>() -> impl Parser<'a, InType> {
    context(
        "input type",
        alt((
            char('$').map(|_| InType::Execute),
            char('<').map(|_| InType::Read),
            success(()).map(|_| InType::RawData),
        )),
    )
}

fn command<'a>() -> impl Parser<'a, Command<'a>> {
    context(
        "mdsh command",
        (out_type(), space0, in_type()).map(|(out_type, _, in_type)| Command { in_type, out_type }),
    )
}

fn transition_line<'a>() -> impl Parser<'a, &'a str> {
    context(
        "transition line",
        delimited(
            tag("<!-- :: "),
            take_until(" -->"),
            (tag(" -->"), space0, newline),
        ),
    )
}

/// Split a string on ` :: ` to extract an optional transition text suffix.
/// Returns `(data_line, transition)`.
fn split_transition(s: Option<&str>) -> (Option<&str>, Option<&str>) {
    match s {
        Some(s) if s.starts_with(":: ") => (None, Some(&s[3..])),
        Some(s) if s == "::" => (None, None),
        Some(s) => match s.split_once(" :: ") {
            Some(("", after)) => (None, Some(after)),
            Some((before, after)) => (Some(before), Some(after)),
            None => {
                if s.is_empty() {
                    (None, None)
                } else {
                    (Some(s), None)
                }
            }
        },
        None => (None, None),
    }
}

fn comment<'a>() -> impl Parser<'a, &'a str> {
    recognize(context(
        "comment",
        (tag("<!--"), cut(take_until("-->")), tag("-->")),
    ))
}

fn actionable_comment<'a>() -> impl Parser<'a, Action<'a>> {
    context(
        "comment",
        delimited(
            tag("<!--"),
            take_until1("-->"),
            tag("-->").and(space0).and(newline),
        )
        .and_then(
            (
                space0,
                command(),
                space0,
                opt(
                    recognize(many1_count(not(tag("-->").or(tag("\n"))).and(anychar)))
                        .map(|s: &str| s.trim_end()),
                ),
                opt(newline),
                rest,
            )
                .map(|(_, command, _, raw_data_line, _, data)| {
                    let (data_line, transition) = split_transition(raw_data_line);
                    Action {
                        command,
                        data_line,
                        data: Some(data),
                        transition,
                    }
                }),
        ),
    )
}

/// Inline code container:
/// ```md
/// `> yml $ echo 'foo: bar'`
/// ```
fn inline_code<'a>() -> impl Parser<'a, Action<'a>> {
    context(
        "inline code",
        recognize(take_while_m_n(1, 2, |x| x == '`').and(not(char('`'))))
            .flat_map(|q1| terminated(take_until1(q1), tag(q1).and(newline)))
            .and_then((command(), space0, rest))
            .map(|(command, _, rest)| {
                let (data_line, transition) = split_transition(Some(rest));
                Action {
                    command,
                    data_line,
                    data: None,
                    transition,
                }
            }),
    )
}

fn non_actionable_code_block<'a>() -> impl Parser<'a, &'a str> {
    fn meta_line<'a>() -> impl Parser<'a, ()> {
        take_until("\n").and(newline).map(|_| ())
    }
    recognize(code_block(FnParser::new(meta_line)))
}

fn actionable_code_block<'a>() -> impl Parser<'a, Action<'a>> {
    fn meta_line<'a>(
    ) -> impl Parser<'a, (Command<'a>, Option<&'a str>, Option<&'a str>)> {
        (
            opt(filepath()).and(space0),
            command(),
            space0,
            opt(take_until1("\n")),
            newline,
        )
            .map(|(_srclang, command, _, rest, _)| {
                let (data_line, transition) = split_transition(rest);
                (command, data_line, transition)
            })
    }
    context(
        "code block with mdsh command",
        code_block(FnParser::new(meta_line)),
    )
    .map(|((command, data_line, transition), data)| Action {
        command,
        data_line,
        data: Some(data),
        transition,
    })
}

fn code_block<'a, X>(
    meta_line: FnParser<impl Parser<'a, X> + 'a>,
) -> impl Parser<'a, (X, &'a str)> {
    let fence = alt((
        (tag("```"), take_while(|x| x == '`')),
        (tag("~~~"), take_while(|x| x == '~')),
    ));
    context(
        "code block",
        recognize((space0, fence)).flat_map(move |q| {
            (
                meta_line.clone(),
                cut(alt((
                    peek(tag(q)).map(|_| "\n"), // covers the edge case with empty code block
                    recognize(many0_count(not(newline.and(tag(q))).and(anychar)).and(newline)),
                ))),
                cut(tag(q).and(newline)),
            )
        }),
    )
    .map(|(meta_line, data, _)| (meta_line, data))
}

pub fn env_var_line<'a>() -> impl Parser<'a, Option<(&'a str, &'a str)>> {
    let kv_definition = (
        recognize(many1_count(alphanumeric1.or(recognize(char('_'))))),
        cut(char('=')),
        cut(alt((
            delimited(
                char('"'),
                escaped(none_of("\n\"\\"), '\\', one_of("\n\"\\")),
                char('"'),
            ),
            delimited(
                char('\''),
                escaped(none_of("'\\"), '\\', one_of("'\\")),
                char('\''),
            ),
            take_until(" "),
            take_until("\n"),
            rest,
        ))),
        opt(newline),
    )
        .map(|(k, _, v, _)| (k, v));
    alt((
        (char('#'), take_until("\n"), newline).map(|_| None),
        (multispace1).map(|_| None),
        kv_definition.map(Some),
    ))
}

pub fn fmt_nom_error<'a>(
    input: &'a str,
    src_name: &'a str,
) -> impl FnOnce(VerboseError<&'a str>) -> anyhow::Error {
    move |e| {
        anyhow::anyhow!(
            "Parsing error in {}:\n{}",
            src_name,
            nom_language::error::convert_error(input, e)
        )
    }
}
