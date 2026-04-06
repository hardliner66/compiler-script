use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

pub use rune;

use rune::{
    Context, ContextError, Diagnostics, Module, Sources, Vm,
    runtime::Value,
    termcolor::{ColorChoice, StandardStream},
};

use rune::Source;

use rune::compile;
use rune::macros::{MacroContext, TokenStream, quote};

mod code_module;
mod types;

#[rune::macro_]
fn span(cx: &mut MacroContext<'_, '_, '_>, _stream: &TokenStream) -> compile::Result<TokenStream> {
    let span = cx.macro_span();
    let pos = cx.lit(span.start.into_usize())?;
    Ok(quote!(#pos).into_token_stream(cx)?)
}

pub fn module() -> Result<Module, ContextError> {
    let mut m = Module::with_item(["rugen"])?;

    m.macro_meta(span)?;

    Ok(m)
}

fn generate(
    pretty: bool,
    script: impl AsRef<Path>,
    output: Option<impl AsRef<Path>>,
) -> anyhow::Result<()> {
    let mut context = Context::with_default_modules()?;
    let mut sources = Sources::new();
    let source = Source::from_path(script)?;
    context.install(module()?)?;
    context.install(code_module::module()?)?;
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

    let result = vm.call(rune::Hash::type_hash(["main"]), ())?;
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
    #[clap(short, long)]
    pretty: bool,
    #[clap(short, long)]
    output: Option<PathBuf>,
    script: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let Cli {
        pretty,
        output,
        script,
    } = clap::Parser::parse();
    generate(pretty, script, output)?;
    println!("Hello, world!");
    Ok(())
}
