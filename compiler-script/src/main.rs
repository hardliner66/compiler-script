use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

pub use rune;

use rune::{
    Context, ContextError, Diagnostics, Module, Sources, Vm,
    termcolor::{ColorChoice, StandardStream},
};

use rune::Source;

use rune::compile;
use rune::macros::{MacroContext, TokenStream, quote};

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
    let output_string = if pretty {
        serde_json::to_string_pretty(&result)?
    } else {
        serde_json::to_string(&result)?
    };
    if let Some(output_path) = output {
        std::fs::write(output_path, output_string)?;
    } else {
        println!("{output_string}");
    }
    Ok(())
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
