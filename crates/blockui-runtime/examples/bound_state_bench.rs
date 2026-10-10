use blockui_runtime::{Actor, Message};
use std::{path::Path, time::Instant};

fn batch(actor: &mut Actor, message: Message) -> Result<f64, String> {
    let start = Instant::now();
    for _ in 0..1000 {
        actor.dispatch(message)?;
    }
    Ok(start.elapsed().as_secs_f64() * 1000.0)
}
fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().collect();
    if paths.len() != 3 {
        return Err("bound_state_bench bound unbound".into());
    }
    let mut bound = Actor::new(93, Path::new(".runtime"))?;
    let mut unbound = Actor::new(94, Path::new(".runtime"))?;
    bound.reload(Path::new(&paths[1]))?;
    unbound.reload(Path::new(&paths[2]))?;
    let message = Message {
        action: blockui_abi::key("Counter.increment"),
        value: 0,
    };
    for _ in 0..100 {
        bound.dispatch(message)?;
        unbound.dispatch(message)?;
    }
    let mut bound_samples = Vec::new();
    let mut unbound_samples = Vec::new();
    for round in 0..9 {
        if round % 2 == 0 {
            bound_samples.push(batch(&mut bound, message)?);
            unbound_samples.push(batch(&mut unbound, message)?);
        } else {
            unbound_samples.push(batch(&mut unbound, message)?);
            bound_samples.push(batch(&mut bound, message)?);
        }
    }
    for actor in [&bound, &unbound] {
        assert_eq!(actor.integer("counter.count"), 9100);
        assert_eq!(
            actor.view.bindings[&blockui_abi::key("counter.count")].text,
            "9100"
        );
        assert_eq!(
            actor.view.bindings[&blockui_abi::key("counter.double")].text,
            "18200"
        );
    }
    println!(
        "{{\"profile\":\"dev\",\"unit\":\"us_per_dispatch_including_text_projection\",\"bound\":{bound_samples:?},\"unbound\":{unbound_samples:?}}}"
    );
    Ok(())
}
