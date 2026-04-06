use std::{
    io::{IsTerminal, Read},
    path::{Path, PathBuf},
    sync::Arc,
};

use serde_json::Value as JsonValue;

pub use rune;

use rune::{
    Context, ContextError, Diagnostics, Module, Sources, Vm,
    runtime::Value,
    termcolor::{ColorChoice, StandardStream},
};

use rune::Source;

use rune::compile;
use rune::macros::{MacroContext, TokenStream, quote};

use crate::ast_types::{AstNode, Attr, Span};

mod ast_module;
mod ast_types;
mod code_module;
mod types;

#[rune::macro_]
fn span(cx: &mut MacroContext<'_, '_, '_>, _stream: &TokenStream) -> compile::Result<TokenStream> {
    let span = cx.macro_span();
    let pos = cx.lit(span.start.into_usize())?;
    Ok(quote!(#pos).into_token_stream(cx)?)
}

pub fn module() -> Result<Module, ContextError> {
    let mut m = Module::with_item(["common"])?;

    m.macro_meta(span)?;

    Ok(m)
}

/// Recursively convert a `serde_json::Value` into a `rune::runtime::Value` so
/// that Rune scripts can walk the JSON tree with `node["key"]` / `node.get(…)`.
fn json_to_rune(val: JsonValue) -> anyhow::Result<Value> {
    Ok(match val {
        JsonValue::Null => rune::to_value(())?,
        JsonValue::Bool(b) => rune::to_value(b)?,
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                rune::to_value(i)?
            } else {
                rune::to_value(n.as_f64().unwrap_or(0.0))?
            }
        }
        JsonValue::String(s) => rune::to_value(s)?,
        JsonValue::Array(arr) => {
            let mut vec = rune::runtime::Vec::new();
            for item in arr {
                vec.push(json_to_rune(item)?)?;
            }
            rune::to_value(vec)?
        }
        JsonValue::Object(map) => {
            let mut obj = rune::runtime::Object::new();
            for (k, v) in map {
                obj.insert(rune::alloc::String::try_from(k.as_str())?, json_to_rune(v)?)?;
            }
            rune::to_value(obj)?
        }
    })
}

fn generate(
    pretty: bool,
    text: bool,
    script: impl AsRef<Path>,
    input: Option<impl AsRef<Path>>,
    output: Option<impl AsRef<Path>>,
) -> anyhow::Result<()> {
    let mut context = Context::with_default_modules()?;
    let mut sources = Sources::new();
    let source = Source::from_path(script)?;
    context.install(module()?)?;
    context.install(code_module::module()?)?;
    context.install(ast_module::module()?)?;
    sources.insert(source)?;
    let mut diagnostics = Diagnostics::new();

    let result = rune::prepare(&mut sources)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .build();

    if !diagnostics.is_empty() {
        let mut writer = StandardStream::stderr(ColorChoice::Always);
        diagnostics.emit(&mut writer, &sources)?;
    }

    let unit = Arc::new(result?);
    let runtime = Arc::new(context.runtime()?);

    let mut vm = Vm::new(runtime.clone(), unit);

    // Read the text to parse from a file argument or from stdin.
    let input_text: Option<String> = match input {
        Some(path) => Some(std::fs::read_to_string(path)?),
        None => {
            let stdin = std::io::stdin();
            if stdin.is_terminal() {
                None
            } else {
                let mut buf = String::new();
                stdin.lock().read_to_string(&mut buf)?;
                Some(buf)
            }
        }
    };
    let result = if let Some(input_text) = input_text {
        let input = if text {
            rune::to_value(input_text)?
        } else {
            if let Ok(json) = serde_json::from_str(&input_text) {
                json_to_rune(json)?
            } else {
                rune::to_value(input_text)?
            }
        };
        vm.call(rune::Hash::type_hash(["main"]), (dbg!(input),))?
    } else {
        vm.call(rune::Hash::type_hash(["main"]), ())?
    };

    let output_string = value_to_json(result, pretty)?;
    if let Some(output_path) = output {
        std::fs::write(output_path, output_string)?;
    } else {
        println!("{output_string}");
    }
    Ok(())
}

/// Serialize a Rune `Value` to JSON.
///
/// Rune wraps our `Any` types in `Value::Any` (an "external reference"), which
/// `serde_json` cannot reach into directly.  We try to downcast to each of our
/// concrete code-AST types — all of which implement `serde::Serialize` — and
/// serialize the inner Rust value.  If none match (e.g. the script returned a
/// plain integer or string), we fall back to Rune's own `Serialize` impl which
/// handles the primitive `Value` variants.
fn value_to_json(value: Value, pretty: bool) -> anyhow::Result<String> {
    use types::{
        CodeModule, CodeType, Expr, Field, FieldInit, Item, MatchArm, Param, Pattern, PatternField,
        Stmt, Variant,
    };

    macro_rules! try_as {
        ($ty:ty) => {
            if let Ok(v) = value.borrow_ref::<$ty>() {
                return Ok(if pretty {
                    serde_json::to_string_pretty(&*v)?
                } else {
                    serde_json::to_string(&*v)?
                });
            }
        };
    }

    // Most-likely return types first.
    try_as!(CodeModule);
    try_as!(Item);
    try_as!(Expr);
    try_as!(Stmt);
    try_as!(Pattern);
    try_as!(CodeType);
    try_as!(Field);
    try_as!(Param);
    try_as!(Variant);
    try_as!(FieldInit);
    try_as!(MatchArm);
    try_as!(PatternField);

    try_as!(AstNode);
    try_as!(Span);
    try_as!(Attr);

    // Fallback: let Rune serialize primitive Value variants (integers, strings,
    // booleans, vecs, objects, …).
    Ok(if pretty {
        serde_json::to_string_pretty(&value)?
    } else {
        serde_json::to_string(&value)?
    })
}

#[derive(Debug, clap::Parser)]
struct Cli {
    /// Pretty-print the JSON output.
    #[clap(short, long)]
    pretty: bool,

    /// Force input to be treated as plain text.
    #[clap(short, long)]
    text: bool,

    /// Write output to this file instead of stdout.
    #[clap(short, long)]
    output: Option<PathBuf>,

    /// JSON AST file to pass as the first argument to the script's `main(ast)`.
    /// When omitted the script is called as `main()` with no arguments.
    #[clap(short, long)]
    input: Option<PathBuf>,

    /// The Rune compiler script to run.
    script: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let Cli {
        pretty,
        text,
        output,
        input,
        script,
    } = clap::Parser::parse();
    generate(pretty, text, script, input, output)
}
