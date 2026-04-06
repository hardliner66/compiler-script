use rune::Any;
use serde::{Deserialize, Serialize};

// ─── LitValue ─────────────────────────────────────────────────────────────────

/// A literal value.  Private implementation detail — not exposed as Any.
/// Uses struct variants to satisfy serde's internally-tagged format constraint
/// (newtype variants wrapping primitives are unsupported in that mode).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum LitValue {
    Int { value: i128 },
    Float { value: f64 },
    Bool { value: bool },
    Str { value: String },
    Null,
}

// ─── CodeType ─────────────────────────────────────────────────────────────────

/// A type reference — e.g. `"i32"`, `"Vec<String>"`, `"Option<&str>"`.
#[derive(Debug, Clone, Any, Serialize, Deserialize)]
#[rune(item = ::code)]
pub struct CodeType {
    pub repr: String,
}

// ─── Field ────────────────────────────────────────────────────────────────────

/// A named field in a struct or enum variant.
#[derive(Debug, Clone, Any, Serialize, Deserialize)]
#[rune(item = ::code)]
pub struct Field {
    pub name: String,
    pub ty: CodeType,
    pub optional: bool,
}

// ─── Param ────────────────────────────────────────────────────────────────────

/// A function parameter (`name: ty`).
#[derive(Debug, Clone, Any, Serialize, Deserialize)]
#[rune(item = ::code)]
pub struct Param {
    pub name: String,
    pub ty: CodeType,
}

// ─── Variant ──────────────────────────────────────────────────────────────────

/// An enum variant — unit, tuple, or struct form.
#[derive(Debug, Clone, Any, Serialize, Deserialize)]
#[rune(item = ::code)]
pub struct Variant {
    pub name: String,
    /// Fields present for tuple/struct variants; empty for unit variants.
    pub fields: Vec<Field>,
    /// `true` → tuple variant, `false` → unit or struct variant.
    pub is_tuple: bool,
}

// ─── PatternKind ──────────────────────────────────────────────────────────────

/// Internal enum — the actual shape of a [`Pattern`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum PatternKind {
    Wildcard,
    Literal {
        value: LitValue,
    },
    Variable {
        name: String,
    },
    /// e.g. `Some(x)`, `Circle(r, theta)`
    EnumTuple {
        path: String,
        bindings: Vec<String>,
    },
    /// e.g. `Point { x, y }`
    EnumStruct {
        path: String,
        fields: Vec<PatternField>,
    },
    Tuple {
        elements: Vec<Pattern>,
    },
    Struct {
        name: String,
        fields: Vec<PatternField>,
    },
    Or {
        alternatives: Vec<Pattern>,
    },
    Ref {
        inner: Box<Pattern>,
    },
}

// ─── PatternField ─────────────────────────────────────────────────────────────

/// A `name: pattern` binding inside a struct or enum-struct pattern.
#[derive(Debug, Clone, Any, Serialize, Deserialize)]
#[rune(item = ::code)]
pub struct PatternField {
    pub name: String,
    pub pattern: Pattern,
}

// ─── Pattern ──────────────────────────────────────────────────────────────────

/// A match pattern.
#[derive(Debug, Clone, Any, Serialize, Deserialize)]
#[rune(item = ::code)]
pub struct Pattern {
    #[serde(flatten)]
    pub(crate) kind: PatternKind,
}

// ─── MatchArm ─────────────────────────────────────────────────────────────────

/// One arm of a `match` expression.
#[derive(Debug, Clone, Any, Serialize, Deserialize)]
#[rune(item = ::code)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
}

// ─── ExprKind ─────────────────────────────────────────────────────────────────

/// Internal enum — the actual shape of an [`Expr`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "expr", rename_all = "snake_case")]
pub(crate) enum ExprKind {
    Lit {
        value: LitValue,
    },
    Var {
        name: String,
    },
    Call {
        func: String,
        args: Vec<Expr>,
    },
    MethodCall {
        receiver: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    BinOp {
        op: String,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    UnOp {
        op: String,
        operand: Box<Expr>,
    },
    Block {
        stmts: Vec<Stmt>,
        trailing: Option<Box<Expr>>,
    },
    If {
        cond: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    Loop {
        body: Vec<Stmt>,
    },
    Array {
        elements: Vec<Expr>,
    },
    Tuple {
        elements: Vec<Expr>,
    },
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
    },
    FieldAccess {
        inner: Box<Expr>,
        field: String,
    },
    Index {
        inner: Box<Expr>,
        index: Box<Expr>,
    },
    StructExpr {
        name: String,
        fields: Vec<FieldInit>,
    },
    Closure {
        params: Vec<Param>,
        body: Box<Expr>,
    },
    Return {
        value: Option<Box<Expr>>,
    },
    Break {
        value: Option<Box<Expr>>,
    },
    Continue,
    Cast {
        inner: Box<Expr>,
        ty: CodeType,
    },
    Range {
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        inclusive: bool,
    },
    Await {
        inner: Box<Expr>,
    },
}

// ─── FieldInit ────────────────────────────────────────────────────────────────

/// A field initializer in a struct expression — `name: value`.
/// Defined after `ExprKind` but Rust resolves mutual references across items.
#[derive(Debug, Clone, Any, Serialize, Deserialize)]
#[rune(item = ::code)]
pub struct FieldInit {
    pub name: String,
    pub value: Expr,
}

// ─── Expr ─────────────────────────────────────────────────────────────────────

/// An expression node.
#[derive(Debug, Clone, Any, Serialize, Deserialize)]
#[rune(item = ::code)]
pub struct Expr {
    #[serde(flatten)]
    pub(crate) kind: ExprKind,
}

// ─── StmtKind ─────────────────────────────────────────────────────────────────

/// Internal enum — the actual shape of a [`Stmt`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "stmt", rename_all = "snake_case")]
pub(crate) enum StmtKind {
    Expr {
        expr: Expr,
    },
    Let {
        name: String,
        ty: Option<CodeType>,
        mutable: bool,
        value: Expr,
    },
    LetDestructure {
        pattern: Pattern,
        value: Expr,
    },
    Return {
        value: Option<Expr>,
    },
    If {
        cond: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    Loop {
        body: Vec<Stmt>,
    },
    For {
        var: String,
        iter: Expr,
        body: Vec<Stmt>,
    },
    Match {
        scrutinee: Expr,
        arms: Vec<MatchArm>,
    },
    Break {
        value: Option<Expr>,
    },
    Continue,
}

// ─── Stmt ─────────────────────────────────────────────────────────────────────

/// A statement node.
#[derive(Debug, Clone, Any, Serialize, Deserialize)]
#[rune(item = ::code)]
pub struct Stmt {
    #[serde(flatten)]
    pub(crate) kind: StmtKind,
}

// ─── ItemKind ─────────────────────────────────────────────────────────────────

/// Internal enum — the actual shape of an [`Item`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "item", rename_all = "snake_case")]
pub(crate) enum ItemKind {
    Struct {
        name: String,
        fields: Vec<Field>,
        derives: Vec<String>,
        is_pub: bool,
        is_tuple: bool,
    },
    Enum {
        name: String,
        variants: Vec<Variant>,
        derives: Vec<String>,
        is_pub: bool,
    },
    Fn {
        name: String,
        params: Vec<Param>,
        return_type: Option<CodeType>,
        body: Vec<Stmt>,
        is_async: bool,
        is_pub: bool,
    },
    TypeAlias {
        name: String,
        ty: CodeType,
        is_pub: bool,
    },
    Const {
        name: String,
        ty: CodeType,
        value: Expr,
        is_pub: bool,
    },
    Use {
        path: String,
        is_pub: bool,
    },
}

// ─── Item ─────────────────────────────────────────────────────────────────────

/// A top-level item — struct, enum, fn, type alias, const, or use declaration.
#[derive(Debug, Clone, Any, Serialize, Deserialize)]
#[rune(item = ::code)]
pub struct Item {
    #[serde(flatten)]
    pub(crate) kind: ItemKind,
}

impl Item {
    /// Append one derive to this item.  No-op for non-struct/enum items.
    pub fn with_derive(mut self, derive: String) -> Self {
        match &mut self.kind {
            ItemKind::Struct { derives, .. } | ItemKind::Enum { derives, .. } => {
                derives.push(derive);
            }
            _ => {}
        }
        self
    }

    /// Replace the entire derive list.  No-op for non-struct/enum items.
    pub fn with_derives(mut self, new_derives: Vec<String>) -> Self {
        match &mut self.kind {
            ItemKind::Struct { derives, .. } | ItemKind::Enum { derives, .. } => {
                *derives = new_derives;
            }
            _ => {}
        }
        self
    }

    /// Mark this item as `pub`.
    pub fn make_pub(mut self) -> Self {
        match &mut self.kind {
            ItemKind::Struct { is_pub, .. }
            | ItemKind::Enum { is_pub, .. }
            | ItemKind::Fn { is_pub, .. }
            | ItemKind::TypeAlias { is_pub, .. }
            | ItemKind::Const { is_pub, .. }
            | ItemKind::Use { is_pub, .. } => {
                *is_pub = true;
            }
        }
        self
    }

    /// Mark a function item as `async`.  No-op for non-fn items.
    pub fn make_async(mut self) -> Self {
        if let ItemKind::Fn { is_async, .. } = &mut self.kind {
            *is_async = true;
        }
        self
    }

    /// Return a human-readable one-line summary of this item.
    pub fn display(&self) -> String {
        match &self.kind {
            ItemKind::Struct {
                name,
                fields,
                is_pub,
                is_tuple,
                ..
            } => {
                let pub_kw = if *is_pub { "pub " } else { "" };
                if *is_tuple {
                    let fs = fields
                        .iter()
                        .map(|f| f.ty.repr.clone())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{}struct {}({})", pub_kw, name, fs)
                } else {
                    let fs = fields
                        .iter()
                        .map(|f| format!("{}: {}", f.name, f.ty.repr))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{}struct {} {{ {} }}", pub_kw, name, fs)
                }
            }
            ItemKind::Enum {
                name,
                variants,
                is_pub,
                ..
            } => {
                let pub_kw = if *is_pub { "pub " } else { "" };
                let vs = variants
                    .iter()
                    .map(|v| v.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}enum {} {{ {} }}", pub_kw, name, vs)
            }
            ItemKind::Fn {
                name,
                params,
                return_type,
                is_pub,
                is_async,
                ..
            } => {
                let pub_kw = if *is_pub { "pub " } else { "" };
                let async_kw = if *is_async { "async " } else { "" };
                let ps = params
                    .iter()
                    .map(|p| format!("{}: {}", p.name, p.ty.repr))
                    .collect::<Vec<_>>()
                    .join(", ");
                match return_type {
                    Some(rt) => {
                        format!("{}{}fn {}({}) -> {}", pub_kw, async_kw, name, ps, rt.repr)
                    }
                    None => format!("{}{}fn {}({})", pub_kw, async_kw, name, ps),
                }
            }
            ItemKind::TypeAlias { name, ty, is_pub } => {
                let pub_kw = if *is_pub { "pub " } else { "" };
                format!("{}type {} = {}", pub_kw, name, ty.repr)
            }
            ItemKind::Const {
                name, ty, is_pub, ..
            } => {
                let pub_kw = if *is_pub { "pub " } else { "" };
                format!("{}const {}: {}", pub_kw, name, ty.repr)
            }
            ItemKind::Use { path, is_pub } => {
                let pub_kw = if *is_pub { "pub " } else { "" };
                format!("{}use {}", pub_kw, path)
            }
        }
    }
}

// ─── CodeModule ───────────────────────────────────────────────────────────────

/// A named collection of top-level items representing a whole code module.
#[derive(Debug, Clone, Any, Serialize, Deserialize)]
#[rune(item = ::code)]
pub struct CodeModule {
    pub name: String,
    pub items: Vec<Item>,
}

impl CodeModule {
    /// Append one item and return the updated module (builder-style).
    pub fn add_item(mut self, item: Item) -> Self {
        self.items.push(item);
        self
    }

    /// Return a human-readable one-line summary of this module.
    pub fn display(&self) -> String {
        let n = self.items.len();
        format!(
            "module {} ({} item{})",
            self.name,
            n,
            if n == 1 { "" } else { "s" }
        )
    }
}
