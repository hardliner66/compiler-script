use rune::{ContextError, Module};

use crate::ast_types::{AstNode, Attr, Scanner, Span};

/// Build and return the `ast` Rune module.
///
/// All constructors and helpers are available at the `ast::` path in scripts:
///   `ast::node(...)`, `ast::leaf(...)`, `ast::scanner(...)`, etc.
pub fn module() -> Result<Module, ContextError> {
    let mut m = Module::with_item(["ast"])?;

    // ── Register types ────────────────────────────────────────────────────────
    m.ty::<Span>()?;
    m.ty::<Attr>()?;
    m.ty::<AstNode>()?;
    m.ty::<Scanner>()?;

    // ── Span constructor ──────────────────────────────────────────────────────

    // ast::span(start, end, line, col) — build a source span manually.
    m.function("span", |start: u64, end: u64, line: u64, col: u64| Span {
        start,
        end,
        line,
        col,
    })
    .build()?;

    // ── AstNode constructors ──────────────────────────────────────────────────

    // ast::node(kind, children) — interior node with positional children.
    m.function("node", |kind: &str, children: Vec<AstNode>| AstNode {
        kind: kind.to_owned(),
        name: None,
        value: None,
        span: None,
        children,
        attrs: vec![],
    })
    .build()?;

    // ast::leaf(kind, value) — terminal/leaf node carrying a string value.
    m.function("leaf", |kind: &str, value: &str| AstNode {
        kind: kind.to_owned(),
        name: None,
        value: Some(value.to_owned()),
        span: None,
        children: vec![],
        attrs: vec![],
    })
    .build()?;

    // ast::empty(kind) — a node with no children and no value.
    m.function("empty", |kind: &str| AstNode {
        kind: kind.to_owned(),
        name: None,
        value: None,
        span: None,
        children: vec![],
        attrs: vec![],
    })
    .build()?;

    // ast::named_node(name, kind, children) — interior node with a role name.
    // The `name` field lets a later pass find this child by name rather than
    // by index, e.g. `parent.get_named_child("body")`.
    m.function(
        "named_node",
        |name: &str, kind: &str, children: Vec<AstNode>| AstNode {
            kind: kind.to_owned(),
            name: Some(name.to_owned()),
            value: None,
            span: None,
            children,
            attrs: vec![],
        },
    )
    .build()?;

    // ast::named_leaf(name, kind, value) — leaf node with a role name.
    m.function("named_leaf", |name: &str, kind: &str, value: &str| {
        AstNode {
            kind: kind.to_owned(),
            name: Some(name.to_owned()),
            value: Some(value.to_owned()),
            span: None,
            children: vec![],
            attrs: vec![],
        }
    })
    .build()?;

    // ── AstNode instance methods ──────────────────────────────────────────────

    m.associated_function("with_span", AstNode::with_span)?;
    m.associated_function("with_attr", AstNode::with_attr)?;
    m.associated_function("with_name", AstNode::with_name)?;
    m.associated_function("add_child", AstNode::add_child)?;
    m.associated_function("display", AstNode::display)?;
    m.associated_function("child_count", AstNode::child_count)?;
    m.associated_function("get_attr", AstNode::get_attr)?;
    m.associated_function("get_named_child", AstNode::get_named_child)?;
    m.associated_function("get_child", AstNode::get_child)?;

    // ── Scanner constructor ───────────────────────────────────────────────────

    // ast::scanner(input) — create a new scanner over the given string.
    m.function("scanner", |input: &str| Scanner::new(input))
        .build()?;

    // ── Scanner instance methods ──────────────────────────────────────────────
    //
    // Methods that inspect state without moving take `&self`; methods that
    // consume input take `&mut self`.  Rune handles the borrow automatically
    // based on the function signature.

    // — State / position —
    m.associated_function("is_done", Scanner::is_done)?;
    m.associated_function("pos", Scanner::pos)?;
    m.associated_function("line_num", Scanner::line_num)?;
    m.associated_function("col_num", Scanner::col_num)?;
    m.associated_function("remaining", Scanner::remaining)?;

    // — Peeking (shared borrow) —
    m.associated_function("peek", Scanner::peek)?;
    m.associated_function("peek_str", Scanner::peek_str)?;
    m.associated_function("match_str", Scanner::match_str)?;

    // — Current-character predicates (shared borrow) —
    m.associated_function("is_alpha", Scanner::is_alpha)?;
    m.associated_function("is_digit", Scanner::is_digit)?;
    m.associated_function("is_alphanumeric", Scanner::is_alphanumeric)?;
    m.associated_function("is_whitespace", Scanner::is_whitespace)?;

    // — Consuming (mutable borrow) —
    m.associated_function("advance", Scanner::advance)?;
    m.associated_function("consume_str", Scanner::consume_str)?;
    m.associated_function("expect_str", Scanner::expect_str)?;

    // — Skipping (mutable borrow) —
    m.associated_function("skip_whitespace", Scanner::skip_whitespace)?;
    m.associated_function("skip_whitespace_inline", Scanner::skip_whitespace_inline)?;
    m.associated_function("skip_line_comment", Scanner::skip_line_comment)?;
    m.associated_function("skip_block_comment", Scanner::skip_block_comment)?;

    // — Reading structured tokens (mutable borrow) —
    m.associated_function("read_ident", Scanner::read_ident)?;
    m.associated_function("read_digits", Scanner::read_digits)?;
    m.associated_function("read_number", Scanner::read_number)?;
    m.associated_function("read_line", Scanner::read_line)?;
    m.associated_function("read_until_str", Scanner::read_until_str)?;
    m.associated_function("read_quoted", Scanner::read_quoted)?;

    // — Span helpers —
    m.associated_function("current_span", Scanner::current_span)?;
    m.associated_function("span_from", Scanner::span_from)?;

    Ok(m)
}
