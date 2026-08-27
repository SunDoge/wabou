use std::{
    fs::OpenOptions,
    io::Write as _,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::Duration,
};

fn record(line: &str) {
    let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("hot-reload.log")
    else {
        return;
    };
    let _ = writeln!(file, "{line}");
}

fn capability_handler(state: Arc<AtomicU64>) {
    let tick = state.fetch_add(1, Ordering::Relaxed) + 1;
    let handler_version = 1_u64;
    let line = format!(
        "TICK pid={} tick={tick} handler={handler_version}",
        std::process::id(),
    );
    println!("{line}");
    record(&line);
}

fn main() {
    dioxus_devtools::connect_subsecond();
    println!("READY pid={}", std::process::id());
    let stable_state = Arc::new(AtomicU64::new(0));
    let mut handler = subsecond::HotFn::current(capability_handler as fn(Arc<AtomicU64>));

    loop {
        handler.call((stable_state.clone(),));
        thread::sleep(Duration::from_millis(250));
    }
}
