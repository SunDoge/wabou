use std::net::{TcpStream, ToSocketAddrs};
use std::process::{Command, ExitStatus};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(windows)]
use process_wrap::std::JobObject;
#[cfg(unix)]
use process_wrap::std::ProcessGroup;
use process_wrap::std::{ChildWrapper, CommandWrap};

use super::{Result, ensure};

pub(super) struct ManagedChild {
    pub(super) child: Box<dyn ChildWrapper>,
}

impl ManagedChild {
    pub(super) fn spawn(command: Command) -> std::io::Result<Self> {
        let mut command = CommandWrap::from(command);
        #[cfg(unix)]
        command.wrap(ProcessGroup::leader());
        #[cfg(windows)]
        command.wrap(JobObject);
        command.spawn().map(|child| Self { child })
    }

    pub(super) fn terminate(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for ManagedChild {
    fn drop(&mut self) {
        self.terminate();
    }
}

pub(super) fn configure_test_backend(command: &mut Command, native: bool) {
    if native {
        // `--native` must win over an inherited shell/CI variable.
        command.env_remove("WABOU_TEST_HEADLESS");
    } else {
        command.env("WABOU_TEST_HEADLESS", "1");
    }
}

pub(super) fn wait_for_managed_child(
    command: Command,
    timeout: Duration,
    stopped: &AtomicBool,
) -> Result<ExitStatus> {
    let mut child = ManagedChild::spawn(command)?;
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.child.try_wait()? {
            return Ok(status);
        }
        if stopped.load(Ordering::Acquire) {
            child.terminate();
            return Err("Wabou behavior test interrupted".into());
        }
        if Instant::now() >= deadline {
            child.terminate();
            return Err(format!(
                "Wabou behavior test host exceeded its final {}s watchdog",
                timeout.as_secs()
            )
            .into());
        }
        thread::sleep(Duration::from_millis(10));
    }
}

pub(super) fn wait_for_vite(url: &str, child: &mut dyn ChildWrapper) -> Result<()> {
    let authority = url.trim_start_matches("http://");
    let address = authority
        .to_socket_addrs()?
        .next()
        .ok_or("Vite address did not resolve")?;
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait()? {
            return Err(format!("Vite exited before startup: {status}").into());
        }
        if TcpStream::connect_timeout(&address, Duration::from_millis(100)).is_ok() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }
    Err("timed out waiting for Vite".into())
}

pub(super) fn supervise(
    host: &mut ManagedChild,
    vite: &mut ManagedChild,
    inspector: Option<&mut ManagedChild>,
) -> Result<()> {
    let stopped = Arc::new(AtomicBool::new(false));
    let signal = stopped.clone();
    ctrlc::set_handler(move || signal.store(true, Ordering::Release))?;
    let mut inspector = inspector;
    let result = loop {
        if stopped.load(Ordering::Acquire) {
            break Ok(());
        }
        if let Some(status) = host.child.try_wait()? {
            break ensure(status, "Rust host");
        }
        if let Some(status) = vite.child.try_wait()? {
            break ensure(status, "Vite dev server");
        }
        if let Some(child) = inspector.as_mut()
            && let Some(status) = child.child.try_wait()?
        {
            eprintln!("[wabou] DevTools exited: {status}");
            inspector = None;
        }
        thread::sleep(Duration::from_millis(50));
    };
    host.terminate();
    vite.terminate();
    if let Some(child) = inspector {
        child.terminate();
    }
    result
}
