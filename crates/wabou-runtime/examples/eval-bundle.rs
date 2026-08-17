//! Evaluate a browser-targeted JavaScript bundle in Wabou's QuickJS runtime.

use std::{env, fs, process};

use wabou_runtime::{JsRuntime, rquickjs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let Some(bundle_path) = args.next() else {
        eprintln!("usage: eval-bundle <bundle.js> [JavaScript expression]");
        process::exit(2);
    };
    let expression = args
        .next()
        .unwrap_or_else(|| "globalThis.__wabouRouterCoreExperiment".to_owned());
    let source = fs::read_to_string(bundle_path)?;
    let runtime = JsRuntime::new()?;
    runtime.with(|ctx| ctx.eval::<(), _>(source.as_str()))?;

    for _ in 0..32 {
        runtime.poll_async_runtime();
        let expression = format!(
            "typeof ({expression}) === 'undefined' ? undefined : JSON.stringify({expression})"
        );
        let result = runtime.with(|ctx| ctx.eval::<Option<String>, _>(expression.as_str()));
        match result {
            Ok(Some(value)) => {
                println!("{value}");
                return Ok(());
            }
            Ok(None) => {}
            Err(error) => return Err(Box::new(error)),
        }
    }

    Err(Box::new(rquickjs::Error::Unknown))
}
