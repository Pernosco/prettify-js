/// Approach translated from
/// https://github.com/mozilla/pretty-fast/blob/55a44e7217f37219e30490b290fac2a181e1d216/pretty-fast.js
/// adapted to use the RESS JS tokenizer.
///
/// pretty-fast uses the Acorn tokenizer, which tokenizes into template strings. The RESS tokenizer
/// returns the entire template string as a token, so template-string handling is a little different.
/// In particular there are no ${ tokens.
use std::cmp::max;

use super::*;

use log::debug;
use ress::tokens::*;
use ress::*;

#[derive(Clone)]
struct Tok<'a> {
    token: Token<&'a str>,
    start: SourceCoord,
    end: SourceCoord,
    starts_array_literal: bool,
}

type Stack<'a> = Vec<Tok<'a>>;

fn top_token<'a, 'b>(s: &'b Stack<'a>) -> Option<&'b Token<&'a str>> {
    s.last().map(|v| &v.token)
}

fn is_pre_array_literal_token(token: &Tok) -> bool {
    match &token.token {
        &Token::Keyword(Keyword::Case(_))
        | &Token::Keyword(Keyword::Delete(_))
        | &Token::Keyword(Keyword::Do(_))
        | &Token::Keyword(Keyword::Else(_))
        | &Token::Keyword(Keyword::In(_))
        | &Token::Keyword(Keyword::InstanceOf(_))
        | &Token::Keyword(Keyword::TypeOf(_))
        | &Token::Keyword(Keyword::Void(_))
        | &Token::Punct(Punct::Ampersand)
        | &Token::Punct(Punct::AmpersandEqual)
        | &Token::Punct(Punct::Asterisk)
        | &Token::Punct(Punct::AsteriskEqual)
        | &Token::Punct(Punct::Bang)
        | &Token::Punct(Punct::BangDoubleEqual)
        | &Token::Punct(Punct::BangEqual)
        | &Token::Punct(Punct::Caret)
        | &Token::Punct(Punct::CaretEqual)
        | &Token::Punct(Punct::CloseBrace)
        | &Token::Punct(Punct::Colon)
        | &Token::Punct(Punct::Comma)
        | &Token::Punct(Punct::Dash)
        | &Token::Punct(Punct::DashEqual)
        | &Token::Punct(Punct::DoubleAmpersand)
        | &Token::Punct(Punct::DoubleAsterisk)
        | &Token::Punct(Punct::DoubleAsteriskEqual)
        | &Token::Punct(Punct::DoubleDash)
        | &Token::Punct(Punct::DoubleEqual)
        | &Token::Punct(Punct::DoubleGreaterThan)
        | &Token::Punct(Punct::DoubleGreaterThanEqual)
        | &Token::Punct(Punct::DoubleLessThan)
        | &Token::Punct(Punct::DoubleLessThanEqual)
        | &Token::Punct(Punct::DoublePipe)
        | &Token::Punct(Punct::DoublePlus)
        | &Token::Punct(Punct::Equal)
        | &Token::Punct(Punct::ForwardSlash)
        | &Token::Punct(Punct::ForwardSlashEqual)
        | &Token::Punct(Punct::GreaterThan)
        | &Token::Punct(Punct::GreaterThanEqual)
        | &Token::Punct(Punct::LessThan)
        | &Token::Punct(Punct::LessThanEqual)
        | &Token::Punct(Punct::OpenBrace)
        | &Token::Punct(Punct::Percent)
        | &Token::Punct(Punct::PercentEqual)
        | &Token::Punct(Punct::Pipe)
        | &Token::Punct(Punct::PipeEqual)
        | &Token::Punct(Punct::Plus)
        | &Token::Punct(Punct::PlusEqual)
        | &Token::Punct(Punct::QuestionMark)
        | &Token::Punct(Punct::Tilde)
        | &Token::Punct(Punct::TripleEqual)
        | &Token::Punct(Punct::TripleGreaterThan)
        | &Token::Punct(Punct::TripleGreaterThanEqual)
        | &Token::Punct(Punct::SemiColon) => true,
        _ => false,
    }
}

fn starts_array_literal<'a>(token: &Tok, last_token: &Option<Tok>) -> bool {
    if &token.token != &Token::Punct(Punct::OpenBracket) {
        return false;
    }
    if let Some(ref t) = last_token {
        is_pre_array_literal_token(t)
    } else {
        true
    }
}

fn prevent_asi_after_token(token: &Tok) -> bool {
    match &token.token {
        &Token::Keyword(Keyword::Delete(_))
        | &Token::Keyword(Keyword::In(_))
        | &Token::Keyword(Keyword::InstanceOf(_))
        | &Token::Keyword(Keyword::TypeOf(_))
        | &Token::Keyword(Keyword::Void(_))
        | &Token::Keyword(Keyword::New(_))
        | &Token::Punct(Punct::Ampersand)
        | &Token::Punct(Punct::AmpersandEqual)
        | &Token::Punct(Punct::Asterisk)
        | &Token::Punct(Punct::AsteriskEqual)
        | &Token::Punct(Punct::Bang)
        | &Token::Punct(Punct::BangDoubleEqual)
        | &Token::Punct(Punct::BangEqual)
        | &Token::Punct(Punct::Caret)
        | &Token::Punct(Punct::CaretEqual)
        | &Token::Punct(Punct::Comma)
        | &Token::Punct(Punct::Dash)
        | &Token::Punct(Punct::DashEqual)
        | &Token::Punct(Punct::DoubleAmpersand)
        | &Token::Punct(Punct::DoubleAsterisk)
        | &Token::Punct(Punct::DoubleAsteriskEqual)
        | &Token::Punct(Punct::DoubleGreaterThan)
        | &Token::Punct(Punct::DoubleGreaterThanEqual)
        | &Token::Punct(Punct::DoubleLessThan)
        | &Token::Punct(Punct::DoubleLessThanEqual)
        | &Token::Punct(Punct::DoublePipe)
        | &Token::Punct(Punct::Equal)
        | &Token::Punct(Punct::ForwardSlash)
        | &Token::Punct(Punct::ForwardSlashEqual)
        | &Token::Punct(Punct::GreaterThan)
        | &Token::Punct(Punct::GreaterThanEqual)
        | &Token::Punct(Punct::LessThan)
        | &Token::Punct(Punct::LessThanEqual)
        | &Token::Punct(Punct::OpenParen)
        | &Token::Punct(Punct::Percent)
        | &Token::Punct(Punct::PercentEqual)
        | &Token::Punct(Punct::Period)
        | &Token::Punct(Punct::Pipe)
        | &Token::Punct(Punct::PipeEqual)
        | &Token::Punct(Punct::Plus)
        | &Token::Punct(Punct::PlusEqual)
        | &Token::Punct(Punct::Tilde)
        | &Token::Punct(Punct::TripleEqual)
        | &Token::Punct(Punct::TripleGreaterThan)
        | &Token::Punct(Punct::TripleGreaterThanEqual) => true,
        _ => false,
    }
}

fn prevent_asi_before_token(token: &Tok) -> bool {
    match &token.token {
        &Token::Keyword(Keyword::In(_))
        | &Token::Keyword(Keyword::InstanceOf(_))
        | &Token::Punct(Punct::Ampersand)
        | &Token::Punct(Punct::AmpersandEqual)
        | &Token::Punct(Punct::Asterisk)
        | &Token::Punct(Punct::AsteriskEqual)
        | &Token::Punct(Punct::Bang)
        | &Token::Punct(Punct::BangDoubleEqual)
        | &Token::Punct(Punct::BangEqual)
        | &Token::Punct(Punct::Caret)
        | &Token::Punct(Punct::CaretEqual)
        | &Token::Punct(Punct::Comma)
        | &Token::Punct(Punct::Dash)
        | &Token::Punct(Punct::DashEqual)
        | &Token::Punct(Punct::DoubleAmpersand)
        | &Token::Punct(Punct::DoubleAsterisk)
        | &Token::Punct(Punct::DoubleAsteriskEqual)
        | &Token::Punct(Punct::DoubleGreaterThan)
        | &Token::Punct(Punct::DoubleGreaterThanEqual)
        | &Token::Punct(Punct::DoubleLessThan)
        | &Token::Punct(Punct::DoubleLessThanEqual)
        | &Token::Punct(Punct::DoublePipe)
        | &Token::Punct(Punct::Equal)
        | &Token::Punct(Punct::ForwardSlash)
        | &Token::Punct(Punct::ForwardSlashEqual)
        | &Token::Punct(Punct::GreaterThan)
        | &Token::Punct(Punct::GreaterThanEqual)
        | &Token::Punct(Punct::LessThan)
        | &Token::Punct(Punct::LessThanEqual)
        | &Token::Punct(Punct::OpenParen)
        | &Token::Punct(Punct::Percent)
        | &Token::Punct(Punct::PercentEqual)
        | &Token::Punct(Punct::Period)
        | &Token::Punct(Punct::Pipe)
        | &Token::Punct(Punct::PipeEqual)
        | &Token::Punct(Punct::Plus)
        | &Token::Punct(Punct::PlusEqual)
        | &Token::Punct(Punct::Tilde)
        | &Token::Punct(Punct::TripleEqual)
        | &Token::Punct(Punct::TripleGreaterThan)
        | &Token::Punct(Punct::TripleGreaterThanEqual) => true,
        _ => false,
    }
}

fn is_identifier_like(token: &Tok) -> bool {
    match &token.token {
        &Token::Boolean(_)
        | &Token::Ident(_)
        | &Token::Keyword(_)
        | &Token::Null
        | &Token::Number(_) => true,
        _ => false,
    }
}

fn is_asi(token: &Tok, last_token: &Option<Tok>) -> bool {
    let t = if let Some(ref t) = last_token {
        t
    } else {
        return false;
    };
    if token.start.line == t.start.line {
        return false;
    }
    match &t.token {
        &Token::Keyword(Keyword::Return(_)) | &Token::Keyword(Keyword::Yield(_)) => return true,
        _ => (),
    }
    if prevent_asi_after_token(t) || prevent_asi_before_token(token) {
        return false;
    }
    true
}

fn is_line_delimiter(token: &Tok, stack: &Stack) -> bool {
    if token.starts_array_literal {
        return true;
    }
    match &token.token {
        &Token::Punct(Punct::SemiColon) | &Token::Punct(Punct::Comma) => {
            top_token(stack) != Some(&Token::Punct(Punct::OpenParen))
        }
        &Token::Punct(Punct::OpenBrace) => true,
        &Token::Punct(Punct::Colon) => match stack.last().map(|v| &v.token) {
            Some(&Token::Keyword(Keyword::Case(_)))
            | Some(&Token::Keyword(Keyword::Default(_))) => true,
            _ => false,
        },
        _ => false,
    }
}

struct Writer {
    buffer: String,
    current: SourceCoord,
    last_from: SourceCoord,
    mappings: Vec<SourceMapping>,
    indent: u32,
}

impl Writer {
    fn new(indent: u32) -> Writer {
        Writer {
            buffer: String::new(),
            current: SourceCoord {
                line: SourceMapLine(0),
                column: SourceMapColumn(0),
            },
            last_from: SourceCoord {
                line: SourceMapLine(0),
                column: SourceMapColumn(0),
            },
            mappings: Vec::new(),
            indent: indent,
        }
    }
    fn write_new(&mut self, s: &str) {
        if self.mappings.is_empty() {
            self.mappings.push(SourceMapping {
                from: self.current,
                to: self.last_from,
            });
        }
        self.update_current(s);
    }
    fn write(&mut self, s: &str, from: SourceCoord) {
        self.last_from = from;
        self.mappings.push(SourceMapping {
            from: self.current,
            to: self.last_from,
        });
        self.update_current(s);
    }
    fn write_indent(&mut self, level: u32, from: SourceCoord) {
        if level == 0 {
            return;
        }
        self.last_from = from;
        self.mappings.push(SourceMapping {
            from: self.current,
            to: self.last_from,
        });
        let count = level * self.indent;
        for _ in 0..count {
            self.buffer.push(' ');
        }
        self.current.column.0 += count;
    }
    fn update_current(&mut self, s: &str) {
        self.buffer.push_str(s);
        for ch in s.chars() {
            if ch == '\n' {
                self.current.line.0 += 1;
                self.current.column.0 = 0;
            } else {
                self.current.column.0 += ch.len_utf16() as u32;
            }
        }
    }
}

fn append_newline(token: &Tok, stack: &Stack, out: &mut Writer) -> bool {
    if is_line_delimiter(token, stack) {
        out.write("\n", token.start);
        return true;
    }
    false
}

fn need_space_after(token: &Tok, last_token: &Option<Tok>) -> bool {
    if let Some(t) = last_token.as_ref() {
        match &t.token {
            &Token::Keyword(Keyword::Do(_))
            | &Token::Keyword(Keyword::For(_))
            | &Token::Keyword(Keyword::While(_))
            | &Token::Punct(Punct::Ampersand)
            | &Token::Punct(Punct::AmpersandEqual)
            | &Token::Punct(Punct::Asterisk)
            | &Token::Punct(Punct::AsteriskEqual)
            | &Token::Punct(Punct::BangDoubleEqual)
            | &Token::Punct(Punct::BangEqual)
            | &Token::Punct(Punct::Caret)
            | &Token::Punct(Punct::CaretEqual)
            | &Token::Punct(Punct::Colon)
            | &Token::Punct(Punct::Comma)
            | &Token::Punct(Punct::Dash)
            | &Token::Punct(Punct::DashEqual)
            | &Token::Punct(Punct::DoubleAmpersand)
            | &Token::Punct(Punct::DoubleAsterisk)
            | &Token::Punct(Punct::DoubleAsteriskEqual)
            | &Token::Punct(Punct::DoubleEqual)
            | &Token::Punct(Punct::DoubleGreaterThan)
            | &Token::Punct(Punct::DoubleGreaterThanEqual)
            | &Token::Punct(Punct::DoubleLessThan)
            | &Token::Punct(Punct::DoubleLessThanEqual)
            | &Token::Punct(Punct::DoublePipe)
            | &Token::Punct(Punct::Equal)
            | &Token::Punct(Punct::ForwardSlash)
            | &Token::Punct(Punct::ForwardSlashEqual)
            | &Token::Punct(Punct::GreaterThan)
            | &Token::Punct(Punct::GreaterThanEqual)
            | &Token::Punct(Punct::LessThan)
            | &Token::Punct(Punct::LessThanEqual)
            | &Token::Punct(Punct::Percent)
            | &Token::Punct(Punct::PercentEqual)
            | &Token::Punct(Punct::Pipe)
            | &Token::Punct(Punct::PipeEqual)
            | &Token::Punct(Punct::Plus)
            | &Token::Punct(Punct::PlusEqual)
            | &Token::Punct(Punct::QuestionMark)
            | &Token::Punct(Punct::SemiColon)
            | &Token::Punct(Punct::TripleEqual)
            | &Token::Punct(Punct::TripleGreaterThan)
            | &Token::Punct(Punct::TripleGreaterThanEqual) => return true,
            &Token::Number(_) => {
                if let &Token::Punct(Punct::Period) = &token.token {
                    return true;
                }
            }
            &Token::Keyword(Keyword::Break(_))
            | &Token::Keyword(Keyword::Continue(_))
            | &Token::Keyword(Keyword::Return(_)) => match &token.token {
                &Token::Punct(Punct::Period) | &Token::Punct(Punct::SemiColon) => (),
                _ => return true,
            },
            &Token::Keyword(Keyword::Debugger(_))
            | &Token::Keyword(Keyword::Default(_))
            | &Token::Keyword(Keyword::This(_)) => (),
            &Token::Keyword(_) => match &token.token {
                &Token::Punct(Punct::Period) => (),
                _ => return true,
            },
            &Token::Punct(Punct::CloseParen) => match &token.token {
                &Token::Punct(Punct::CloseParen)
                | &Token::Punct(Punct::CloseBracket)
                | &Token::Punct(Punct::SemiColon)
                | &Token::Punct(Punct::Comma)
                | &Token::Punct(Punct::Period) => (),
                _ => return true,
            },
            &Token::Ident(_) => {
                if let &Token::Punct(Punct::OpenBrace) = &token.token {
                    return true;
                }
            }
            _ => (),
        }
        if is_identifier_like(token) && is_identifier_like(t) {
            return true;
        }
    }

    match &token.token {
        &Token::Punct(Punct::AmpersandEqual)
        | &Token::Punct(Punct::AsteriskEqual)
        | &Token::Punct(Punct::CaretEqual)
        | &Token::Punct(Punct::DashEqual)
        | &Token::Punct(Punct::DoubleAsteriskEqual)
        | &Token::Punct(Punct::Equal)
        | &Token::Punct(Punct::ForwardSlashEqual)
        | &Token::Punct(Punct::PercentEqual)
        | &Token::Punct(Punct::PipeEqual)
        | &Token::Punct(Punct::PlusEqual)
        | &Token::Punct(Punct::QuestionMark) => true,
        &Token::Punct(Punct::Ampersand)
        | &Token::Punct(Punct::Asterisk)
        | &Token::Punct(Punct::BangDoubleEqual)
        | &Token::Punct(Punct::BangEqual)
        | &Token::Punct(Punct::Caret)
        | &Token::Punct(Punct::Dash)
        | &Token::Punct(Punct::DoubleAmpersand)
        | &Token::Punct(Punct::DoubleAsterisk)
        | &Token::Punct(Punct::DoubleEqual)
        | &Token::Punct(Punct::DoubleGreaterThan)
        | &Token::Punct(Punct::DoubleGreaterThanEqual)
        | &Token::Punct(Punct::DoubleLessThan)
        | &Token::Punct(Punct::DoubleLessThanEqual)
        | &Token::Punct(Punct::DoublePipe)
        | &Token::Punct(Punct::GreaterThan)
        | &Token::Punct(Punct::GreaterThanEqual)
        | &Token::Punct(Punct::LessThan)
        | &Token::Punct(Punct::LessThanEqual)
        | &Token::Punct(Punct::Percent)
        | &Token::Punct(Punct::Pipe)
        | &Token::Punct(Punct::Plus) => last_token.is_some(),
        _ => false,
    }
}

fn increments_indent(token: &Tok) -> bool {
    match &token.token {
        &Token::Punct(Punct::OpenBrace) | &Token::Keyword(Keyword::Switch(_)) => true,
        _ => token.starts_array_literal,
    }
}

fn decrements_indent(token: &Tok, stack: &Stack) -> bool {
    match &token.token {
        &Token::Punct(Punct::CloseBrace) => true,
        &Token::Punct(Punct::CloseBracket) => stack
            .last()
            .map(|v| v.starts_array_literal)
            .unwrap_or(false),
        _ => false,
    }
}

fn prepend_white_space(
    token: &Tok,
    last_token: &Option<Tok>,
    stack: &Stack,
    mut added_newline: bool,
    mut added_space: bool,
    indent_level: u32,
    out: &mut Writer,
) {
    if let Some(&Token::Punct(Punct::CloseBrace)) = last_token.as_ref().map(|v| &v.token) {
        let start = last_token.as_ref().unwrap().start;
        match &token.token {
            &Token::Keyword(Keyword::While(_)) => {
                if let Some(&Token::Keyword(Keyword::Do(_))) = top_token(stack) {
                    out.write(" ", start);
                    added_space = true;
                } else {
                    out.write("\n", start);
                    added_newline = true;
                }
            }
            &Token::Keyword(Keyword::Else(_))
            | &Token::Keyword(Keyword::Catch(_))
            | &Token::Keyword(Keyword::Finally(_)) => {
                out.write(" ", start);
                added_space = true;
            }
            &Token::Punct(Punct::OpenParen)
            | &Token::Punct(Punct::SemiColon)
            | &Token::Punct(Punct::Comma)
            | &Token::Punct(Punct::CloseParen)
            | &Token::Punct(Punct::Period)
            | &Token::Template(_) => (),
            _ => {
                out.write("\n", start);
                added_newline = true;
            }
        }
    }

    match &token.token {
        &Token::Punct(Punct::Colon) => {
            if let Some(&Token::Punct(Punct::QuestionMark)) = top_token(stack) {
                out.write(" ", last_token.as_ref().unwrap().start);
                added_space = true;
            }
        }
        &Token::Keyword(Keyword::Else(_)) => match last_token.as_ref().map(|v| &v.token) {
            Some(&Token::Punct(Punct::CloseBrace)) | Some(&Token::Punct(Punct::Period)) => (),
            Some(_) => {
                out.write(" ", last_token.as_ref().unwrap().start);
                added_space = true;
            }
            None => (),
        },
        _ => (),
    }

    if is_asi(token, last_token) || decrements_indent(token, stack) {
        if !added_newline {
            out.write("\n", last_token.as_ref().unwrap().start);
            added_newline = true;
        }
    }

    if added_newline {
        match &token.token {
            &Token::Keyword(Keyword::Case(_)) | &Token::Keyword(Keyword::Default(_)) => {
                out.write_indent(max(1, indent_level) - 1, token.start);
            }
            _ => {
                out.write_indent(indent_level, token.start);
            }
        }
    } else if !added_space && need_space_after(token, last_token) {
        out.write(" ", last_token.as_ref().unwrap().start);
    }
}

fn add_token(token: &Tok, out: &mut Writer) {
    out.write(&token.token.to_string(), token.start);
}

fn belongs_on_stack(token: &Tok) -> bool {
    match &token.token {
        &Token::Keyword(Keyword::Case(_))
        | &Token::Keyword(Keyword::Default(_))
        | &Token::Keyword(Keyword::Do(_))
        | &Token::Keyword(Keyword::Switch(_))
        | &Token::Punct(Punct::OpenBrace)
        | &Token::Punct(Punct::OpenParen)
        | &Token::Punct(Punct::OpenBracket)
        | &Token::Punct(Punct::QuestionMark) => true,
        _ => false,
    }
}

fn should_pop_stack(token: &Tok, stack: &Stack) -> bool {
    match &token.token {
        &Token::Keyword(Keyword::While(_)) => match top_token(stack) {
            Some(&Token::Keyword(Keyword::Do(_))) => true,
            _ => false,
        },
        &Token::Punct(Punct::CloseBracket)
        | &Token::Punct(Punct::CloseParen)
        | &Token::Punct(Punct::CloseBrace) => true,
        &Token::Punct(Punct::Colon) => match top_token(stack) {
            Some(&Token::Keyword(Keyword::Case(_)))
            | Some(&Token::Keyword(Keyword::Default(_)))
            | Some(&Token::Punct(Punct::QuestionMark)) => true,
            _ => false,
        },
        _ => false,
    }
}

fn add_comment(token: &Tok, next_token: Option<&Tok>, indent_level: u32, out: &mut Writer) -> bool {
    out.write_indent(indent_level, token.start);
    let comment = if let &Token::Comment(ref c) = &token.token {
        c
    } else {
        panic!("Must be a comment");
    };
    let need_new_line = match comment.kind {
        CommentKind::Multi | CommentKind::Html => {
            out.write_new("/*");
            let mut first = true;
            for line in comment.content.lines() {
                if first {
                    first = false;
                } else {
                    out.write_indent(indent_level, token.start);
                    out.write_new("  ");
                }
                out.write_new(line);
            }
            out.write_indent(indent_level, token.start);
            out.write_new("*/");
            next_token
                .map(|t| t.start.line != token.start.line)
                .unwrap_or(false)
        }
        CommentKind::Hashbang | CommentKind::Single => {
            out.write_new(&comment.to_string());
            true
        }
    };
    if need_new_line {
        out.write_new("\n");
    } else {
        out.write_new(" ");
    }
    need_new_line
}

fn convert_position(pos: Position) -> SourceCoord {
    SourceCoord {
        line: SourceMapLine(pos.line as u32 - 1),
        column: SourceMapColumn(pos.column as u32 - 1),
    }
}

fn convert_token<'a>(item: Item<Token<&'a str>>) -> Tok<'a> {
    debug!("token: {:?} -> {:?}", item.location.start, item.token);
    Tok {
        token: item.token,
        start: convert_position(item.location.start),
        end: convert_position(item.location.end),
        starts_array_literal: false,
    }
}

/// Prettyprint JS source code. Returns the prettyprinted code,
/// plus a list of SourceMappings in source order (both in original and prettyprinted
/// code ... we don't reorder code).
///
/// The SourceMapping 'from' coordinates are
/// in the prettyprinted code, the 'to' coordinates are in the original (presumably
/// minified/obfuscated code).
///
/// Example:
/// ```
/// let (pretty, _) = prettify_js::prettyprint("function x(a){return a;}");
/// assert_eq!(pretty, "function x(a) {\n  return a;\n}\n");
/// ```
pub fn prettyprint(source: &str) -> (String, Vec<SourceMapping>) {
    let mut indent_level = 0;
    let mut out = Writer::new(2);
    let mut added_newline = false;
    let mut added_space = false;
    let mut stack: Stack = Vec::new();
    let mut scanner = Scanner::new(source)
        .filter_map(|v| match v {
            Ok(v) => Some(convert_token(v)),
            Err(_) => None,
        })
        .peekable();
    let mut last_token: Option<Tok> = None;

    while let Some(mut token) = scanner.next() {
        let next_token = scanner.peek();
        match &token.token {
            &Token::Comment(_) => {
                let comment_indent_level = if last_token
                    .as_ref()
                    .map(|v| v.end.line == token.start.line)
                    .unwrap_or(false)
                {
                    out.write_new(" ");
                    0
                } else {
                    indent_level
                };
                added_newline = add_comment(&token, next_token, comment_indent_level, &mut out);
                added_space = !added_newline;
                continue;
            }
            &Token::EoF => break,
            _ => (),
        }

        token.starts_array_literal = starts_array_literal(&token, &last_token);

        if belongs_on_stack(&token) {
            stack.push(token.clone());
        }

        if decrements_indent(&token, &stack) {
            indent_level = max(1, indent_level) - 1;
            if let &Token::Punct(Punct::CloseBrace) = &token.token {
                if stack.len() >= 2 {
                    if let &Token::Keyword(Keyword::Switch(_)) = &stack[stack.len() - 2].token {
                        indent_level = max(1, indent_level) - 1;
                    }
                }
            }
        }

        prepend_white_space(
            &token,
            &last_token,
            &stack,
            added_newline,
            added_space,
            indent_level,
            &mut out,
        );
        add_token(&token, &mut out);
        added_space = false;
        let mut same_line_comment = false;
        if let Some(&Token::Comment(_)) = next_token.as_ref().map(|v| &v.token) {
            if next_token.unwrap().start.line == token.end.line {
                same_line_comment = true;
            }
        }
        if !same_line_comment {
            added_newline = append_newline(&token, &stack, &mut out);
        }

        if should_pop_stack(&token, &stack) {
            stack.pop();
            if let &Token::Punct(Punct::CloseBrace) = &token.token {
                if let Some(&Token::Keyword(Keyword::Switch(_))) = stack.last().map(|v| &v.token) {
                    stack.pop();
                }
            }
        }

        if increments_indent(&token) {
            indent_level += 1;
        }

        last_token = Some(token);
    }

    if !added_newline {
        out.write_new("\n");
    }

    (out.buffer, out.mappings)
}
