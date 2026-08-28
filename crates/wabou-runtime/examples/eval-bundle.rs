//! Evaluate a browser-targeted JavaScript bundle in Wabou's QuickJS runtime.

use std::{env, fs, process, thread, time};

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
    let mut runtime = JsRuntime::new()?;
    runtime.boot(&source)?;

    let deadline = time::Instant::now() + time::Duration::from_secs(5);
    loop {
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

        if time::Instant::now() >= deadline {
            return Err(Box::new(rquickjs::Error::Unknown));
        }
        // Native async functions wake the real window event loop. This
        // standalone evaluator has no winit loop, so yield to Tokio and poll
        // again instead of treating the first parked job as completion.
        thread::sleep(time::Duration::from_millis(1));
    }
}
