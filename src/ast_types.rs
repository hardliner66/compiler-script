use rune::Any;
use serde::{Deserialize, Serialize};

// ── Span ─────────────────────────────────────────────────────────────────────

/// A half-open byte range in the source text, plus the line/column at the start.
#[derive(Debug, Clone, Any, Serialize, Deserialize)]
#[rune(item = ::ast)]
pub struct Span {
    /// Byte offset of the first character (inclusive).
    pub start: u64,
    /// Byte offset just past the last character (exclusive).
    pub end: u64,
    /// 1-based line number at `start`.
    pub line: u64,
    /// 1-based column number at `start`.
    pub col: u64,
}

// ── Attr ─────────────────────────────────────────────────────────────────────

/// A single key-value metadata entry attached to an [`AstNode`].
#[derive(Debug, Clone, Any, Serialize, Deserialize)]
#[rune(item = ::ast)]
pub struct Attr {
    pub key: String,
    pub value: String,
}

// ── AstNode ──────────────────────────────────────────────────────────────────

/// A generic, tree-structured AST node.
///
/// Leaf nodes carry a `value`; interior nodes carry `children`.
/// Every node has a `kind` string that identifies what syntactic construct it
/// represents (e.g. `"program"`, `"fn_def"`, `"binary_expr"`, `"ident"`).
///
/// The optional `name` field lets a parent label a child with a role name
/// (e.g. `"params"`, `"body"`) so that later passes can look children up by
/// name rather than by index.
#[derive(Debug, Clone, Any, Serialize, Deserialize)]
#[rune(item = ::ast)]
pub struct AstNode {
    /// The syntactic category of this node.
    pub kind: String,

    /// Role name of this node within its parent, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Terminal (leaf) value — present for token-level nodes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// Source location, if tracked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,

    /// Child nodes.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub children: Vec<AstNode>,

    /// Arbitrary key-value metadata.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub attrs: Vec<Attr>,
}

impl AstNode {
    // ── Builder methods (take ownership, return Self) ─────────────────────

    /// Attach a source span to this node.
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    /// Add a single key-value attribute.
    pub fn with_attr(mut self, key: &str, value: &str) -> Self {
        self.attrs.push(Attr {
            key: key.to_string(),
            value: value.to_string(),
        });
        self
    }

    /// Set the role name of this node within its parent.
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Append one child node.
    pub fn add_child(mut self, child: AstNode) -> Self {
        self.children.push(child);
        self
    }

    // ── Query methods ─────────────────────────────────────────────────────

    /// Return a short human-readable summary.
    pub fn display(&self) -> String {
        match &self.value {
            Some(v) => format!("{}({:?})", self.kind, v),
            None => format!("{}[{}]", self.kind, self.children.len()),
        }
    }

    /// Number of direct children.
    pub fn child_count(&self) -> i64 {
        self.children.len() as i64
    }

    /// Look up an attribute value by key.
    pub fn get_attr(&self, key: &str) -> Option<String> {
        self.attrs
            .iter()
            .find(|a| a.key == key)
            .map(|a| a.value.clone())
    }

    /// Find the first child whose `name` field equals `name`.
    pub fn get_named_child(&self, name: &str) -> Option<AstNode> {
        self.children
            .iter()
            .find(|c| c.name.as_deref() == Some(name))
            .cloned()
    }

    /// Return the child at the given index, or `None` if out of range.
    pub fn get_child(&self, index: i64) -> Option<AstNode> {
        if index < 0 {
            return None;
        }
        self.children.get(index as usize).cloned()
    }
}

// ── Scanner ───────────────────────────────────────────────────────────────────

/// A stateful, forward-only scanner over a Unicode text input.
///
/// Not serialisable — it is a purely ephemeral helper used by parser scripts
/// while they build an [`AstNode`] tree.
#[derive(Debug, Any)]
#[rune(item = ::ast)]
pub struct Scanner {
    /// The input decoded into a random-access `Vec<char>`.
    chars: Vec<char>,
    /// Current position (index into `chars`).
    pos: usize,
    /// Current 1-based line number.
    line: u64,
    /// Current 1-based column number.
    col: u64,
}

impl Scanner {
    /// Create a new scanner positioned at the start of `input`.
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    // ── Internal helpers ──────────────────────────────────────────────────

    /// Advance by exactly one character, updating line/column tracking.
    fn step(&mut self) {
        if self.pos < self.chars.len() {
            if self.chars[self.pos] == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            self.pos += 1;
        }
    }

    // ── Position / state ──────────────────────────────────────────────────

    /// `true` when all input has been consumed.
    pub fn is_done(&self) -> bool {
        self.pos >= self.chars.len()
    }

    /// Current byte offset (character index).
    pub fn pos(&self) -> u64 {
        self.pos as u64
    }

    /// Current 1-based line number.
    pub fn line_num(&self) -> u64 {
        self.line
    }

    /// Current 1-based column number.
    pub fn col_num(&self) -> u64 {
        self.col
    }

    /// Everything from the current position to the end as a `String`.
    pub fn remaining(&self) -> String {
        self.chars[self.pos..].iter().collect()
    }

    // ── Peeking ───────────────────────────────────────────────────────────

    /// Return the current character without consuming it, or `None` at EOF.
    pub fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    /// Return the next `n` characters as a `String` without consuming them.
    /// Returns fewer characters if fewer remain.
    pub fn peek_str(&self, n: i64) -> String {
        let n = (n as usize).min(self.chars.len().saturating_sub(self.pos));
        self.chars[self.pos..self.pos + n].iter().collect()
    }

    /// `true` if the upcoming characters exactly match `s` (without consuming).
    pub fn match_str(&self, s: &str) -> bool {
        let needle: Vec<char> = s.chars().collect();
        let end = self.pos + needle.len();
        end <= self.chars.len() && self.chars[self.pos..end] == needle[..]
    }

    // ── Predicates on the current character ──────────────────────────────

    /// `true` if the current character is an ASCII letter or `_`.
    pub fn is_alpha(&self) -> bool {
        self.chars
            .get(self.pos)
            .is_some_and(|c| c.is_alphabetic() || *c == '_')
    }

    /// `true` if the current character is an ASCII digit.
    pub fn is_digit(&self) -> bool {
        self.chars.get(self.pos).is_some_and(char::is_ascii_digit)
    }

    /// `true` if the current character is an ASCII letter, digit, or `_`.
    pub fn is_alphanumeric(&self) -> bool {
        self.chars
            .get(self.pos)
            .is_some_and(|c| c.is_alphanumeric() || *c == '_')
    }

    /// `true` if the current character is ASCII whitespace.
    pub fn is_whitespace(&self) -> bool {
        self.chars
            .get(self.pos)
            .is_some_and(char::is_ascii_whitespace)
    }

    // ── Consuming ─────────────────────────────────────────────────────────

    /// Consume and return one character, or `None` at EOF.
    pub fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied()?;
        self.step();
        Some(c)
    }

    /// If the upcoming characters match `s`, consume them and return `true`.
    /// Otherwise return `false` without moving.
    pub fn consume_str(&mut self, s: &str) -> bool {
        if self.match_str(s) {
            for _ in s.chars() {
                self.step();
            }
            true
        } else {
            false
        }
    }

    /// Consume `s` and return `Ok(())`, or return `Err(message)` if it doesn't match.
    pub fn expect_str(&mut self, s: &str) -> Result<(), String> {
        if self.consume_str(s) {
            Ok(())
        } else {
            let got = self.peek_str(s.chars().count() as i64);
            Err(format!(
                "expected {:?} but found {:?} at line {} col {}",
                s, got, self.line, self.col
            ))
        }
    }

    // ── Skipping ─────────────────────────────────────────────────────────

    /// Skip any number of whitespace characters (spaces, tabs, newlines, …).
    pub fn skip_whitespace(&mut self) {
        while self
            .chars
            .get(self.pos)
            .is_some_and(char::is_ascii_whitespace)
        {
            self.step();
        }
    }

    /// Skip horizontal whitespace only — spaces and tabs, but **not** newlines.
    pub fn skip_whitespace_inline(&mut self) {
        while let Some(c) = self.chars.get(self.pos) {
            if *c == ' ' || *c == '\t' || *c == '\r' {
                self.step();
            } else {
                break;
            }
        }
    }

    /// If the current position starts a line comment (beginning with `prefix`),
    /// skip to the end of the line and return `true`. Otherwise return `false`.
    pub fn skip_line_comment(&mut self, prefix: &str) -> bool {
        if !self.match_str(prefix) {
            return false;
        }
        for _ in prefix.chars() {
            self.step();
        }
        while let Some(c) = self.chars.get(self.pos) {
            if *c == '\n' {
                break;
            }
            self.step();
        }
        true
    }

    /// If the current position starts a block comment delimited by `open` and
    /// `close`, consume until the matching close delimiter and return `true`.
    /// Returns `false` (and does not move) if `open` doesn't match.
    /// Note: does **not** handle nested block comments.
    pub fn skip_block_comment(&mut self, open: &str, close: &str) -> bool {
        if !self.match_str(open) {
            return false;
        }
        for _ in open.chars() {
            self.step();
        }
        while !self.is_done() {
            if self.match_str(close) {
                for _ in close.chars() {
                    self.step();
                }
                break;
            }
            self.step();
        }
        true
    }

    // ── Reading structured tokens ─────────────────────────────────────────

    /// Read and return an identifier: `[A-Za-z_][A-Za-z0-9_]*`.
    /// Returns an empty string if the current character does not start one.
    pub fn read_ident(&mut self) -> String {
        let mut out = String::new();
        if self
            .chars
            .get(self.pos)
            .is_some_and(|c| c.is_alphabetic() || *c == '_')
        {
            out.push(self.chars[self.pos]);
            self.step();
            while self
                .chars
                .get(self.pos)
                .is_some_and(|c| c.is_alphanumeric() || *c == '_')
            {
                out.push(self.chars[self.pos]);
                self.step();
            }
        }
        out
    }

    /// Read and return a sequence of ASCII digits.
    /// Returns an empty string if the current character is not a digit.
    pub fn read_digits(&mut self) -> String {
        let mut out = String::new();
        while self.chars.get(self.pos).is_some_and(char::is_ascii_digit) {
            out.push(self.chars[self.pos]);
            self.step();
        }
        out
    }

    /// Read an integer or floating-point literal: `[0-9]+(.[0-9]+)?`.
    pub fn read_number(&mut self) -> String {
        let mut out = self.read_digits();
        if self.chars.get(self.pos) == Some(&'.') {
            // Only treat it as a decimal if the next character after the dot is
            // also a digit, to avoid consuming a range operator like `1..`.
            if self
                .chars
                .get(self.pos + 1)
                .is_some_and(char::is_ascii_digit)
            {
                out.push('.');
                self.step(); // consume '.'
                out.push_str(&self.read_digits());
            }
        }
        out
    }

    /// Read and return all characters up to (but not including) the next newline.
    pub fn read_line(&mut self) -> String {
        let mut out = String::new();
        while let Some(c) = self.chars.get(self.pos) {
            if *c == '\n' {
                break;
            }
            out.push(*c);
            self.step();
        }
        out
    }

    /// Read and return characters until the substring `end` is found.
    /// The `end` string itself is **not** consumed.
    pub fn read_until_str(&mut self, end: &str) -> String {
        let mut out = String::new();
        while !self.is_done() && !self.match_str(end) {
            out.push(self.chars[self.pos]);
            self.step();
        }
        out
    }

    /// Read a quoted string delimited by `delim` (e.g. `'"'` or `'\''`),
    /// handling `\`-escape sequences.  The opening delimiter must already be
    /// the current character; it is consumed together with the closing one.
    ///
    /// Recognised escapes: `\n`, `\t`, `\r`, `\\`, `\"`, `\'`, and `\<delim>`.
    /// Any other `\x` is returned as the literal character `x`.
    pub fn read_quoted(&mut self, delim: char) -> String {
        // consume opening delimiter
        if self.chars.get(self.pos) == Some(&delim) {
            self.step();
        }
        let mut out = String::new();
        loop {
            match self.chars.get(self.pos).copied() {
                None => break, // unterminated — just stop
                Some('\\') => {
                    self.step(); // consume backslash
                    let escaped = match self.chars.get(self.pos).copied() {
                        Some('n') => '\n',
                        Some('t') => '\t',
                        Some('r') => '\r',
                        Some('\\') => '\\',
                        Some('"') => '"',
                        Some('\'') => '\'',
                        Some(c) => c,
                        None => break,
                    };
                    out.push(escaped);
                    self.step();
                }
                Some(c) if c == delim => {
                    self.step(); // consume closing delimiter
                    break;
                }
                Some(c) => {
                    out.push(c);
                    self.step();
                }
            }
        }
        out
    }

    // ── Span helpers ──────────────────────────────────────────────────────

    /// Return a zero-length span at the current position.
    pub fn current_span(&self) -> Span {
        Span {
            start: self.pos as u64,
            end: self.pos as u64,
            line: self.line,
            col: self.col,
        }
    }

    /// Return a span from byte offset `start` up to the current position.
    /// The `line` and `col` fields record the position at *`start`* — callers
    /// should capture them with `line_num()` / `col_num()` before parsing.
    pub fn span_from(&self, start: u64, start_line: u64, start_col: u64) -> Span {
        Span {
            start,
            end: self.pos as u64,
            line: start_line,
            col: start_col,
        }
    }
}
