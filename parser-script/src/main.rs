use std::{
    io::{IsTerminal, Read},
    path::{Path, PathBuf},
    sync::Arc,
};

use rune::Source;
use rune::{
    Context, Diagnostics, Sources, Vm,
    runtime::Value,
    termcolor::{ColorChoice, StandardStream},
};

mod ast_module;
mod ast_types;

fn parse(
    pretty: bool,
    script: impl AsRef<Path>,
    input: Option<impl AsRef<Path>>,
    output: Option<impl AsRef<Path>>,
) -> anyhow::Result<()> {
    let mut context = Context::with_default_modules()?;
    context.install(ast_module::module()?)?;

    let mut sources = Sources::new();
    sources.insert(Source::from_path(script)?)?;

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
    let mut vm = Vm::new(runtime, unit);

    // Read the text to parse from a file argument or from stdin.
    let input_text: String = match input {
        Some(path) => std::fs::read_to_string(path)?,
        None => {
            let stdin = std::io::stdin();
            if stdin.is_terminal() {
                anyhow::bail!(
                    "no input provided\n\
                     \n\
                     Pass a file with --input <file>, or pipe text to stdin:\n\
                     \n\
                     \techo '1 + 2 * 3' | cargo run --bin parser-script -- --pretty examples/arithmetic.rn\n\
                     \tcargo run --bin parser-script -- --pretty --input expr.txt examples/arithmetic.rn"
                );
            }
            let mut buf = String::new();
            stdin.lock().read_to_string(&mut buf)?;
            buf
        }
    };

    // Call the script's `main(input: String) -> AstNode`.
    let result = vm.call(rune::Hash::type_hash(["main"]), (input_text,))?;
    let output_string = value_to_json(result, pretty)?;

    match output {
        Some(path) => std::fs::write(path, output_string)?,
        None => println!("{output_string}"),
    }

    Ok(())
}

/// Serialize a Rune `Value` to JSON.
///
/// Our AST types cross the Rune FFI boundary as `Value::Any` (opaque external
/// references), which `serde_json` cannot reach into.  We try each concrete
/// type in turn — all implement `serde::Serialize` — and fall back to Rune's
/// own serialization for plain values (integers, strings, vecs, …).
fn value_to_json(value: Value, pretty: bool) -> anyhow::Result<String> {
    use ast_types::{AstNode, Attr, Span};

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

    try_as!(AstNode);
    try_as!(Span);
    try_as!(Attr);

    // Fallback: primitive Rune values (integer, float, bool, string, vec, …).
    Ok(if pretty {
        serde_json::to_string_pretty(&value)?
    } else {
        serde_json::to_string(&value)?
    })
}

#[derive(Debug, clap::Parser)]
#[command(
    name = "parser-script",
    about = "Run a Rune parser script against an input file and emit a generic AST as JSON"
)]
struct Cli {
    /// Pretty-print the JSON output.
    #[clap(short, long)]
    pretty: bool,

    /// Write output to this file instead of stdout.
    #[clap(short, long)]
    output: Option<PathBuf>,

    /// Text file to parse (reads from stdin if omitted).
    #[clap(short, long)]
    input: Option<PathBuf>,

    /// The Rune parser script to run.
    script: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let Cli {
        pretty,
        output,
        input,
        script,
    } = clap::Parser::parse();

    parse(pretty, script, input, output)
}
