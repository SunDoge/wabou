//! Stable executable-owned business services. Only owned bytes are returned to UI.
use blockui_abi::{Bytes, ERROR, MAX_MESSAGE_TEXT_BYTES};
use std::{collections::BTreeMap, rc::Rc};

type Handler = dyn Fn(&[u8]) -> Result<Vec<u8>, String>;

/// Cloneable service catalogue shared by UI roots. Captures must be compiled into
/// the stable host; registering handlers from reloadable libraries is unsupported.
#[derive(Clone, Default)]
pub struct Services {
    handlers: BTreeMap<u64, Rc<Handler>>,
}
impl Services {
    /// Register a typed operation with a regular host-side Rust closure.
    pub fn handle<R: ui_contract::Request>(
        &mut self,
        handler: impl Fn(R) -> Result<R::Response, String> + 'static,
    ) -> Result<(), String> {
        self.register(R::id(), move |bytes| {
            let request: R = ui_contract::decode(bytes)?;
            ui_contract::encode(&handler(request))
        })
    }

    pub fn register(
        &mut self,
        id: u64,
        handler: impl Fn(&[u8]) -> Result<Vec<u8>, String> + 'static,
    ) -> Result<(), String> {
        if self.handlers.contains_key(&id) {
            return Err(format!("application service {id:#018x} already registered"));
        }
        self.handlers.insert(id, Rc::new(handler));
        Ok(())
    }
}

pub(crate) unsafe extern "C" fn call(
    context: *mut std::ffi::c_void,
    id: u64,
    request: Bytes,
    output: *mut u8,
    capacity: usize,
) -> isize {
    if request.len > MAX_MESSAGE_TEXT_BYTES
        || capacity > MAX_MESSAGE_TEXT_BYTES
        || (request.len != 0 && request.ptr.is_null())
        || output.is_null()
    {
        return ERROR as isize;
    }
    let context = unsafe { &*context.cast::<super::Context>() };
    let turn = unsafe { &*context.turn };
    if turn.read_only || turn.previous.is_some() {
        return ERROR as isize;
    }
    let Some(handler) = turn.services.handlers.get(&id).cloned() else {
        return ERROR as isize;
    };
    let input = if request.len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(request.ptr, request.len) }
    };
    // Prevent host panics from unwinding through C. With panic=abort the process
    // aborts normally; recoverable application failures should use Result.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handler(input)));
    let Ok(Ok(bytes)) = result else {
        return ERROR as isize;
    };
    if bytes.len() > capacity {
        return ERROR as isize;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), output, bytes.len());
    }
    bytes.len() as isize
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn projection_migration_and_oversized_inputs_never_invoke_business_code() {
        let calls = Rc::new(std::cell::Cell::new(0));
        let captured = calls.clone();
        let mut turn = super::super::Turn::default();
        turn.services
            .register(9, move |_| {
                captured.set(captured.get() + 1);
                Ok(vec![7])
            })
            .unwrap();
        let mut output = [0; 8];
        let mut invoke = |turn: &mut super::super::Turn, request: Bytes| {
            super::super::with_api(turn, std::ptr::null(), |api| unsafe {
                (api.call_service)(api.context, 9, request, output.as_mut_ptr(), output.len())
            })
        };
        let empty = Bytes {
            ptr: std::ptr::null(),
            len: 0,
        };
        turn.read_only = true;
        assert_eq!(invoke(&mut turn, empty), ERROR as isize);
        turn.read_only = false;
        turn.previous = Some(Default::default());
        assert_eq!(invoke(&mut turn, empty), ERROR as isize);
        turn.previous = None;
        assert_eq!(
            invoke(
                &mut turn,
                Bytes {
                    ptr: std::ptr::null(),
                    len: MAX_MESSAGE_TEXT_BYTES + 1
                }
            ),
            ERROR as isize
        );
        assert_eq!(calls.get(), 0);
        assert_eq!(invoke(&mut turn, empty), 1);
        assert_eq!(calls.get(), 1);
        assert_eq!(output[0], 7);
    }
}
