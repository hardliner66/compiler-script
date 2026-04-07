use rune::{ContextError, Module};

use crate::types::{
    CodeModule, CodeType, Expr, ExprKind, Field, FieldInit, Item, ItemKind, LitValue, MatchArm,
    Param, Pattern, PatternField, PatternKind, Stmt, StmtKind, Variant,
};

/// Build and return the `code` Rune module.
///
/// All constructor functions live at the `code::` path in Rune scripts, e.g.
/// `code::struct_def(...)`, `code::lit_int(42)`, `code::expr_stmt(...)`.
pub fn module() -> Result<Module, ContextError> {
    let mut m = Module::with_item(["code"])?;

    // ── Register types ────────────────────────────────────────────────────────
    m.ty::<CodeType>()?;
    m.ty::<Field>()?;
    m.ty::<Param>()?;
    m.ty::<Variant>()?;
    m.ty::<FieldInit>()?;
    m.ty::<PatternField>()?;
    m.ty::<Pattern>()?;
    m.ty::<MatchArm>()?;
    m.ty::<Expr>()?;
    m.ty::<Stmt>()?;
    m.ty::<Item>()?;
    m.ty::<CodeModule>()?;

    // ── Type constructors ─────────────────────────────────────────────────────

    m.function("type_of", |name: &str| CodeType {
        repr: name.to_owned(),
    })
    .build()?;

    m.function("generic_type", |name: &str, params: Vec<CodeType>| {
        let ps = params
            .iter()
            .map(|p| p.repr.clone())
            .collect::<Vec<_>>()
            .join(", ");
        CodeType {
            repr: format!("{name}<{ps}>"),
        }
    })
    .build()?;

    m.function("array_type", |inner: CodeType, size: i64| CodeType {
        repr: format!("[{}; {}]", inner.repr, size),
    })
    .build()?;

    m.function("slice_type", |inner: CodeType| CodeType {
        repr: format!("[{}]", inner.repr),
    })
    .build()?;

    m.function("optional_type", |inner: CodeType| CodeType {
        repr: format!("Option<{}>", inner.repr),
    })
    .build()?;

    m.function("result_type", |ok: CodeType, err: CodeType| CodeType {
        repr: format!("Result<{}, {}>", ok.repr, err.repr),
    })
    .build()?;

    m.function("tuple_type", |parts: Vec<CodeType>| {
        let ps = parts
            .iter()
            .map(|p| p.repr.clone())
            .collect::<Vec<_>>()
            .join(", ");
        CodeType {
            repr: format!("({ps})"),
        }
    })
    .build()?;

    m.function("ref_type", |inner: CodeType| CodeType {
        repr: format!("&{}", inner.repr),
    })
    .build()?;

    m.function("mut_ref_type", |inner: CodeType| CodeType {
        repr: format!("&mut {}", inner.repr),
    })
    .build()?;

    // ── Component constructors ────────────────────────────────────────────────

    m.function("field", |name: &str, ty: CodeType| Field {
        name: name.to_owned(),
        ty,
        optional: false,
    })
    .build()?;

    m.function("field_opt", |name: &str, ty: CodeType| Field {
        name: name.to_owned(),
        ty,
        optional: true,
    })
    .build()?;

    m.function("param", |name: &str, ty: CodeType| Param {
        name: name.to_owned(),
        ty,
    })
    .build()?;

    m.function("variant", |name: &str| Variant {
        name: name.to_owned(),
        fields: vec![],
        is_tuple: false,
    })
    .build()?;

    m.function("variant_tuple", |name: &str, fields: Vec<Field>| Variant {
        name: name.to_owned(),
        fields,
        is_tuple: true,
    })
    .build()?;

    m.function("variant_struct", |name: &str, fields: Vec<Field>| Variant {
        name: name.to_owned(),
        fields,
        is_tuple: false,
    })
    .build()?;

    m.function("field_init", |name: &str, value: Expr| FieldInit {
        name: name.to_owned(),
        value,
    })
    .build()?;

    // ── Pattern constructors ──────────────────────────────────────────────────

    m.function("pat_wildcard", || Pattern {
        kind: PatternKind::Wildcard,
    })
    .build()?;

    m.function("pat_var", |name: &str| Pattern {
        kind: PatternKind::Variable {
            name: name.to_owned(),
        },
    })
    .build()?;

    m.function("pat_lit_int", |n: i128| Pattern {
        kind: PatternKind::Literal {
            value: LitValue::Int { value: n },
        },
    })
    .build()?;

    m.function("pat_lit_str", |s: &str| Pattern {
        kind: PatternKind::Literal {
            value: LitValue::Str {
                value: s.to_owned(),
            },
        },
    })
    .build()?;

    m.function("pat_lit_bool", |b: bool| Pattern {
        kind: PatternKind::Literal {
            value: LitValue::Bool { value: b },
        },
    })
    .build()?;

    m.function("pat_enum", |path: &str, bindings: Vec<String>| Pattern {
        kind: PatternKind::EnumTuple {
            path: path.to_owned(),
            bindings,
        },
    })
    .build()?;

    m.function("pat_or", |alternatives: Vec<Pattern>| Pattern {
        kind: PatternKind::Or { alternatives },
    })
    .build()?;

    m.function("pat_tuple", |elements: Vec<Pattern>| Pattern {
        kind: PatternKind::Tuple { elements },
    })
    .build()?;

    m.function("pat_ref", |inner: Pattern| Pattern {
        kind: PatternKind::Ref {
            inner: Box::new(inner),
        },
    })
    .build()?;

    m.function("match_arm", |pattern: Pattern, body: Expr| MatchArm {
        pattern,
        guard: None,
        body,
    })
    .build()?;

    m.function(
        "guarded_arm",
        |pattern: Pattern, guard: Expr, body: Expr| MatchArm {
            pattern,
            guard: Some(guard),
            body,
        },
    )
    .build()?;

    // ── Item constructors ─────────────────────────────────────────────────────

    m.function("struct_def", |name: &str, fields: Vec<Field>| Item {
        kind: ItemKind::Struct {
            name: name.to_owned(),
            fields,
            derives: vec![],
            is_pub: false,
            is_tuple: false,
        },
    })
    .build()?;

    m.function("tuple_struct_def", |name: &str, fields: Vec<Field>| Item {
        kind: ItemKind::Struct {
            name: name.to_owned(),
            fields,
            derives: vec![],
            is_pub: false,
            is_tuple: true,
        },
    })
    .build()?;

    m.function("enum_def", |name: &str, variants: Vec<Variant>| Item {
        kind: ItemKind::Enum {
            name: name.to_owned(),
            variants,
            derives: vec![],
            is_pub: false,
        },
    })
    .build()?;

    // fn_def: function with an explicit return type (passes CodeType directly, no Option wrapping)
    m.function(
        "fn_def",
        |name: &str, params: Vec<Param>, return_type: CodeType, body: Vec<Stmt>| Item {
            kind: ItemKind::Fn {
                name: name.to_owned(),
                params,
                return_type: Some(return_type),
                body,
                is_async: false,
                is_pub: false,
            },
        },
    )
    .build()?;

    // fn_def_void: function that returns nothing / unit
    m.function(
        "fn_def_void",
        |name: &str, params: Vec<Param>, body: Vec<Stmt>| Item {
            kind: ItemKind::Fn {
                name: name.to_owned(),
                params,
                return_type: None,
                body,
                is_async: false,
                is_pub: false,
            },
        },
    )
    .build()?;

    m.function("type_alias", |name: &str, ty: CodeType| Item {
        kind: ItemKind::TypeAlias {
            name: name.to_owned(),
            ty,
            is_pub: false,
        },
    })
    .build()?;

    m.function("const_def", |name: &str, ty: CodeType, value: Expr| Item {
        kind: ItemKind::Const {
            name: name.to_owned(),
            ty,
            value,
            is_pub: false,
        },
    })
    .build()?;

    m.function("use_item", |path: &str| Item {
        kind: ItemKind::Use {
            path: path.to_owned(),
            is_pub: false,
        },
    })
    .build()?;

    m.function("code_module", |name: &str, items: Vec<Item>| CodeModule {
        name: name.to_owned(),
        items,
    })
    .build()?;

    // ── Instance methods on Item ──────────────────────────────────────────────

    m.associated_function("with_derive", Item::with_derive)?;
    m.associated_function("with_derives", Item::with_derives)?;
    m.associated_function("make_pub", Item::make_pub)?;
    m.associated_function("make_async", Item::make_async)?;
    m.associated_function("display", Item::display)?;

    // ── Instance methods on CodeModule ────────────────────────────────────────

    m.associated_function("add_item", CodeModule::add_item)?;
    m.associated_function("display", CodeModule::display)?;

    // ── Expression constructors ───────────────────────────────────────────────

    m.function("lit_int", |n: i128| Expr {
        kind: ExprKind::Lit {
            value: LitValue::Int { value: n },
        },
    })
    .build()?;

    m.function("lit_float", |n: f64| Expr {
        kind: ExprKind::Lit {
            value: LitValue::Float { value: n },
        },
    })
    .build()?;

    m.function("lit_str", |s: &str| Expr {
        kind: ExprKind::Lit {
            value: LitValue::Str {
                value: s.to_owned(),
            },
        },
    })
    .build()?;

    m.function("lit_bool", |b: bool| Expr {
        kind: ExprKind::Lit {
            value: LitValue::Bool { value: b },
        },
    })
    .build()?;

    m.function("lit_null", || Expr {
        kind: ExprKind::Lit {
            value: LitValue::Null,
        },
    })
    .build()?;

    m.function("var", |name: &str| Expr {
        kind: ExprKind::Var {
            name: name.to_owned(),
        },
    })
    .build()?;

    m.function("call", |func: &str, args: Vec<Expr>| Expr {
        kind: ExprKind::Call {
            func: func.to_owned(),
            args,
        },
    })
    .build()?;

    m.function(
        "method_call",
        |receiver: Expr, method: &str, args: Vec<Expr>| Expr {
            kind: ExprKind::MethodCall {
                receiver: Box::new(receiver),
                method: method.to_owned(),
                args,
            },
        },
    )
    .build()?;

    m.function("bin_op", |op: &str, lhs: Expr, rhs: Expr| Expr {
        kind: ExprKind::BinOp {
            op: op.to_owned(),
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
    })
    .build()?;

    m.function("un_op", |op: &str, operand: Expr| Expr {
        kind: ExprKind::UnOp {
            op: op.to_owned(),
            operand: Box::new(operand),
        },
    })
    .build()?;

    m.function("block_expr", |stmts: Vec<Stmt>| Expr {
        kind: ExprKind::Block {
            stmts,
            trailing: None,
        },
    })
    .build()?;

    m.function("block_with_trailing", |stmts: Vec<Stmt>, trailing: Expr| {
        Expr {
            kind: ExprKind::Block {
                stmts,
                trailing: Some(Box::new(trailing)),
            },
        }
    })
    .build()?;

    m.function("if_expr", |cond: Expr, then_branch: Expr| Expr {
        kind: ExprKind::If {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: None,
        },
    })
    .build()?;

    m.function(
        "if_else_expr",
        |cond: Expr, then_branch: Expr, else_branch: Expr| Expr {
            kind: ExprKind::If {
                cond: Box::new(cond),
                then_branch: Box::new(then_branch),
                else_branch: Some(Box::new(else_branch)),
            },
        },
    )
    .build()?;

    m.function("match_expr", |scrutinee: Expr, arms: Vec<MatchArm>| Expr {
        kind: ExprKind::Match {
            scrutinee: Box::new(scrutinee),
            arms,
        },
    })
    .build()?;

    m.function("loop_expr", |body: Vec<Stmt>| Expr {
        kind: ExprKind::Loop { body },
    })
    .build()?;

    m.function("array_expr", |elements: Vec<Expr>| Expr {
        kind: ExprKind::Array { elements },
    })
    .build()?;

    m.function("tuple_expr", |elements: Vec<Expr>| Expr {
        kind: ExprKind::Tuple { elements },
    })
    .build()?;

    m.function("assign", |target: Expr, value: Expr| Expr {
        kind: ExprKind::Assign {
            target: Box::new(target),
            value: Box::new(value),
        },
    })
    .build()?;

    m.function("field_access", |expr: Expr, field: &str| Expr {
        kind: ExprKind::FieldAccess {
            inner: Box::new(expr),
            field: field.to_owned(),
        },
    })
    .build()?;

    m.function("index_expr", |expr: Expr, index: Expr| Expr {
        kind: ExprKind::Index {
            inner: Box::new(expr),
            index: Box::new(index),
        },
    })
    .build()?;

    m.function("struct_expr", |name: &str, fields: Vec<FieldInit>| Expr {
        kind: ExprKind::StructExpr {
            name: name.to_owned(),
            fields,
        },
    })
    .build()?;

    m.function("closure", |params: Vec<Param>, body: Expr| Expr {
        kind: ExprKind::Closure {
            params,
            body: Box::new(body),
        },
    })
    .build()?;

    m.function("return_expr", |value: Expr| Expr {
        kind: ExprKind::Return {
            value: Some(Box::new(value)),
        },
    })
    .build()?;

    m.function("return_unit", || Expr {
        kind: ExprKind::Return { value: None },
    })
    .build()?;

    m.function("break_expr", || Expr {
        kind: ExprKind::Break { value: None },
    })
    .build()?;

    m.function("break_with", |value: Expr| Expr {
        kind: ExprKind::Break {
            value: Some(Box::new(value)),
        },
    })
    .build()?;

    m.function("continue_expr", || Expr {
        kind: ExprKind::Continue,
    })
    .build()?;

    m.function("cast", |expr: Expr, ty: CodeType| Expr {
        kind: ExprKind::Cast {
            inner: Box::new(expr),
            ty,
        },
    })
    .build()?;

    m.function("range", |start: Expr, end: Expr| Expr {
        kind: ExprKind::Range {
            start: Some(Box::new(start)),
            end: Some(Box::new(end)),
            inclusive: false,
        },
    })
    .build()?;

    m.function("range_inclusive", |start: Expr, end: Expr| Expr {
        kind: ExprKind::Range {
            start: Some(Box::new(start)),
            end: Some(Box::new(end)),
            inclusive: true,
        },
    })
    .build()?;

    m.function("await_expr", |expr: Expr| Expr {
        kind: ExprKind::Await {
            inner: Box::new(expr),
        },
    })
    .build()?;

    // ── Statement constructors ────────────────────────────────────────────────

    m.function("expr_stmt", |expr: Expr| Stmt {
        kind: StmtKind::Expr { expr },
    })
    .build()?;

    m.function("let_stmt", |name: &str, value: Expr| Stmt {
        kind: StmtKind::Let {
            name: name.to_owned(),
            ty: None,
            mutable: false,
            value,
        },
    })
    .build()?;

    m.function("let_mut_stmt", |name: &str, value: Expr| Stmt {
        kind: StmtKind::Let {
            name: name.to_owned(),
            ty: None,
            mutable: true,
            value,
        },
    })
    .build()?;

    m.function("let_typed", |name: &str, ty: CodeType, value: Expr| Stmt {
        kind: StmtKind::Let {
            name: name.to_owned(),
            ty: Some(ty),
            mutable: false,
            value,
        },
    })
    .build()?;

    m.function("let_typed_mut", |name: &str, ty: CodeType, value: Expr| {
        Stmt {
            kind: StmtKind::Let {
                name: name.to_owned(),
                ty: Some(ty),
                mutable: true,
                value,
            },
        }
    })
    .build()?;

    m.function("let_destructure", |pattern: Pattern, value: Expr| Stmt {
        kind: StmtKind::LetDestructure { pattern, value },
    })
    .build()?;

    m.function("return_stmt", |value: Expr| Stmt {
        kind: StmtKind::Return { value: Some(value) },
    })
    .build()?;

    m.function("return_unit_stmt", || Stmt {
        kind: StmtKind::Return { value: None },
    })
    .build()?;

    m.function("if_stmt", |cond: Expr, then_branch: Vec<Stmt>| Stmt {
        kind: StmtKind::If {
            cond,
            then_branch,
            else_branch: None,
        },
    })
    .build()?;

    m.function(
        "if_else_stmt",
        |cond: Expr, then_branch: Vec<Stmt>, else_branch: Vec<Stmt>| Stmt {
            kind: StmtKind::If {
                cond,
                then_branch,
                else_branch: Some(else_branch),
            },
        },
    )
    .build()?;

    m.function("while_stmt", |cond: Expr, body: Vec<Stmt>| Stmt {
        kind: StmtKind::While { cond, body },
    })
    .build()?;

    m.function("loop_stmt", |body: Vec<Stmt>| Stmt {
        kind: StmtKind::Loop { body },
    })
    .build()?;

    m.function("for_stmt", |var: &str, iter: Expr, body: Vec<Stmt>| Stmt {
        kind: StmtKind::For {
            var: var.to_owned(),
            iter,
            body,
        },
    })
    .build()?;

    m.function("match_stmt", |scrutinee: Expr, arms: Vec<MatchArm>| Stmt {
        kind: StmtKind::Match { scrutinee, arms },
    })
    .build()?;

    m.function("break_stmt", || Stmt {
        kind: StmtKind::Break { value: None },
    })
    .build()?;

    m.function("break_with_stmt", |value: Expr| Stmt {
        kind: StmtKind::Break { value: Some(value) },
    })
    .build()?;

    m.function("continue_stmt", || Stmt {
        kind: StmtKind::Continue,
    })
    .build()?;

    Ok(m)
}
