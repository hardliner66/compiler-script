use std::{
    io::{IsTerminal, Read},
    path::{Path, PathBuf},
    sync::Arc,
};

use rune::{
    runtime::Value,
    termcolor::{ColorChoice, StandardStream},
    Context, Diagnostics, Source, Sources, Vm,
};

fn run(
    pretty: bool,
    script: impl AsRef<Path>,
    input: Option<impl AsRef<Path>>,
    output: Option<impl AsRef<Path>>,
) -> anyhow::Result<()> {
    // ── Build Rune context (default modules only) ─────────────────────────────
    let context = Context::with_default_modules()?;

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

    // ── Read the JSON AST ─────────────────────────────────────────────────────
    let json_text: String = match input {
        Some(path) => std::fs::read_to_string(path)?,
        None => {
            let stdin = std::io::stdin();
            if stdin.is_terminal() {
                anyhow::bail!(
                    "no input provided\n\
                     \n\
                     Pipe the JSON output of parser-script, or pass --input <ast.json>:\n\
                     \n\
                     \tcargo run --bin parser-script -- --pretty --input src.c examples/clang.rn \\\n\
                     \t  | cargo run --bin run-script -- examples/clang.rn --input /dev/stdin\n\
                     \n\
                     \tcargo run --bin parser-script -- --pretty --input src.c examples/clang.rn \\\n\
                     \t  --output ast.json\n\
                     \tcargo run --bin run-script -- --input ast.json examples/clang.rn"
                );
            }
            let mut buf = String::new();
            stdin.lock().read_to_string(&mut buf)?;
            buf
        }
    };

    let json_val: serde_json::Value = serde_json::from_str(&json_text)?;
    let ast_val = json_to_rune(json_val)?;

    // ── Execute the interpreter script ────────────────────────────────────────
    let result = vm.call(rune::Hash::type_hash(["main"]), (ast_val,))?;

    // Serialise to --output if requested; otherwise side-effects are the output.
    if let Some(out_path) = output {
        let json = value_to_json(result, pretty)?;
        std::fs::write(out_path, json)?;
    }

    Ok(())
}

/// Recursively convert a `serde_json::Value` to a `rune::runtime::Value`.
///
/// JSON objects  → `rune::runtime::Object`  (accessible in scripts as `obj["key"]`)
/// JSON arrays   → `rune::runtime::Vec`      (iterable in scripts)
/// JSON strings  → Rune `String`
/// JSON numbers  → `i64` (if whole) or `f64`
/// JSON booleans → Rune `bool`
/// JSON null     → Rune unit `()`
fn json_to_rune(val: serde_json::Value) -> anyhow::Result<Value> {
    Ok(match val {
        serde_json::Value::Null => rune::to_value(())?,
        serde_json::Value::Bool(b) => rune::to_value(b)?,
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                rune::to_value(i)?
            } else {
                rune::to_value(n.as_f64().unwrap_or(0.0))?
            }
        }
        serde_json::Value::String(s) => rune::to_value(s)?,
        serde_json::Value::Array(arr) => {
            let mut vec = rune::runtime::Vec::new();
            for item in arr {
                vec.push(json_to_rune(item)?)?;
            }
            rune::to_value(vec)?
        }
        serde_json::Value::Object(map) => {
            let mut obj = rune::runtime::Object::new();
            for (k, v) in map {
                obj.insert(rune::alloc::String::try_from(k.as_str())?, json_to_rune(v)?)?;
            }
            rune::to_value(obj)?
        }
    })
}

fn value_to_json(value: Value, pretty: bool) -> anyhow::Result<String> {
    Ok(if pretty {
        serde_json::to_string_pretty(&value)?
    } else {
        serde_json::to_string(&value)?
    })
}

#[derive(Debug, clap::Parser)]
#[command(
    name = "run-script",
    about = "Run a Rune interpreter script against a JSON AST produced by parser-script"
)]
struct Cli {
    /// Pretty-print JSON output (only relevant with --output).
    #[clap(short, long)]
    pretty: bool,

    /// Write the script's return value as JSON to this file.
    #[clap(short, long)]
    output: Option<PathBuf>,

    /// JSON AST file to interpret (reads from stdin if omitted).
    #[clap(short, long)]
    input: Option<PathBuf>,

    /// The Rune interpreter script to run.
    script: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let Cli {
        pretty,
        output,
        input,
        script,
    } = clap::Parser::parse();
    run(pretty, script, input, output)
}
