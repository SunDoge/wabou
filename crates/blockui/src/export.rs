//! Shared implementation-local ABI adapters.
//!
//! Keep the substantial execution paths out of Export<A>: UI crates should only
//! monomorphize thin Actor-specific entry points, then reuse compiled SDK code.
//! Slices and Rust callbacks below stay inside this dylib; they are never ABI data.
use super::*;

pub(super) unsafe extern "C" fn events(
    owner: u64,
    operation: u32,
    ids: *const u64,
    len: usize,
) -> i32 {
    guard(|| {
        let max_ids = abi::MAX_VIEW_NODES * if operation == 7 { 2 } else { 1 };
        if len > max_ids || (len != 0 && ids.is_null()) {
            return Err(Error::InvalidView("invalid event lifecycle identities"));
        }
        let ids = if len == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(ids, len) }
        };
        __events::lifecycle(owner, operation, ids)
    })
}

pub(super) unsafe extern "C" fn render_binding(
    api: *const HostApi,
    kind: u32,
    slot: u64,
    token: usize,
    context: *mut std::ffi::c_void,
    write: abi::Write,
) -> i32 {
    guard_with_api(unsafe { &*api }, || {
        if token == 0 {
            return Err(Error::InvalidView("null binding token"));
        }
        // The trusted host returns only a token emitted by this loaded DLL.
        // Rust callbacks never cross into host execution or survive owner unload.
        let cx = Cx {
            api: unsafe { &*api },
            instance: InstancePath::EMPTY,
        };
        if kind == abi::BIND_STYLE && slot == 0 {
            let compute = __events::lookup_style(cx.api.event_owner, token)
                .ok_or(Error::InvalidView("unknown style binding token"))?;
            let bytes = compute(&cx)?.presentation().encode();
            let status = unsafe {
                write(
                    context,
                    abi::Bytes {
                        ptr: bytes.as_ptr(),
                        len: bytes.len(),
                    },
                )
            };
            if status != abi::OK {
                return Err(status.into());
            }
            return Ok(());
        }
        let text = match kind {
            abi::BIND_CAPTURED_TEXT if slot == 0 => {
                let compute = __events::lookup_text(cx.api.event_owner, token)
                    .ok_or(Error::InvalidView("unknown captured text token"))?;
                compute(&cx)?
            }
            abi::BIND_REACTIVE if slot == 0 => {
                let compute: BindingFn = unsafe { std::mem::transmute(token) };
                compute(&cx)?
            }
            abi::BIND_MAP_F64 => {
                let status = unsafe { (cx.api.validate_state)(cx.api.context, slot, abi::F64) };
                if status != abi::OK {
                    return Err(status.into());
                }
                let format: fn(f64) -> String = unsafe { std::mem::transmute(token) };
                let mut state = State::<f64>::new("", 0.0);
                state.spec.key = slot;
                format(state.get(&cx)?)
            }
            abi::BIND_MAP_BOOL => {
                let status = unsafe { (cx.api.validate_state)(cx.api.context, slot, abi::BOOLEAN) };
                if status != abi::OK {
                    return Err(status.into());
                }
                let format: fn(bool) -> String = unsafe { std::mem::transmute(token) };
                let mut state = State::<bool>::new("", false);
                state.spec.key = slot;
                format(state.get(&cx)?)
            }
            abi::BIND_MAP_TEXT => {
                let status = unsafe { (cx.api.validate_state)(cx.api.context, slot, abi::TEXT) };
                if status != abi::OK {
                    return Err(status.into());
                }
                let format: fn(String) -> String = unsafe { std::mem::transmute(token) };
                let mut state = State::<String>::new("");
                state.spec.key = slot;
                format(state.get(&cx)?)
            }
            abi::BIND_MAP_LIST_I64 => {
                let status =
                    unsafe { (cx.api.validate_state)(cx.api.context, slot, abi::I64_LIST) };
                if status != abi::OK {
                    return Err(status.into());
                }
                let format: fn(Vec<i64>) -> String = unsafe { std::mem::transmute(token) };
                let mut state = State::<Vec<i64>>::new("");
                state.spec.key = slot;
                format(state.get(&cx)?)
            }
            abi::BIND_MAP_TEXT_LIST => {
                let status =
                    unsafe { (cx.api.validate_state)(cx.api.context, slot, abi::TEXT_LIST) };
                if status != abi::OK {
                    return Err(status.into());
                }
                let format: fn(Vec<String>) -> String = unsafe { std::mem::transmute(token) };
                let mut state = State::<Vec<String>>::new("");
                state.spec.key = slot;
                format(state.get(&cx)?)
            }
            abi::BIND_MAP_I64 => {
                let status = unsafe { (cx.api.validate_state)(cx.api.context, slot, abi::I64) };
                if status != abi::OK {
                    return Err(status.into());
                }
                let format: fn(i64) -> String = unsafe { std::mem::transmute(token) };
                format(cx.api.state_key::<i64>(slot).get().map_err(Error::from)?)
            }
            _ => return Err(Error::InvalidView("invalid binding callback kind")),
        };
        let status = unsafe { write(context, abi::Bytes::new(&text)) };
        if status != abi::OK {
            return Err(status.into());
        }
        Ok(())
    })
}

pub(super) unsafe fn scope_ids(
    scopes: &[Scope],
    context: *mut std::ffi::c_void,
    emit: abi::EmitScope,
) -> i32 {
    guard(|| {
        if scopes.len() > 256 {
            return Err(Error::InvalidView("too many structural scopes"));
        }
        for scope in scopes {
            let status = unsafe { emit(context, scope.id) };
            if status != abi::OK {
                return Err(status.into());
            }
        }
        Ok(())
    })
}

// Preserve leaf identity, events and observers beneath a stable scope container.
// This stays in shared SDK code rather than expanding in every actor macro.
fn scope_root(view: View) -> View {
    if matches!(
        view.node.kind,
        abi::COLUMN | abi::ROW | abi::DIALOG | abi::SCROLL
    ) {
        view
    } else {
        Column {
            key: Key::default(),
            gap: 0.0,
            padding: 0.0,
        }
        .children(vec![view])
    }
}

pub(super) unsafe fn scope_view(
    scopes: &[Scope],
    actions: &[Action],
    api: *const HostApi,
    id: u64,
    context: *mut std::ffi::c_void,
    emit: abi::Emit,
    emit_binding: abi::EmitBinding,
) -> i32 {
    guard_with_api(unsafe { &*api }, || {
        let captured = __events::lookup_scope(unsafe { &*api }.event_owner, id);
        let declared = scopes.iter().find(|scope| scope.id == id);
        if captured.is_some() && declared.is_some() {
            return Err(Error::InvalidView(
                "captured scope conflicts with declared scope",
            ));
        }
        let cx = Cx {
            api: unsafe { &*api },
            instance: declared.map_or(InstancePath::EMPTY, |scope| scope.instance),
        };
        if cx.api.event_owner != 0 {
            __events::lifecycle(cx.api.event_owner, 5, &[id])?;
        }
        let view = if let Some(build) = captured {
            let view = scope_root(build(&cx)?);
            __component_view(view, InstancePath::EMPTY.scoped(Key(id)), Key(id))
        } else if let Some(scope) = declared {
            scope_root((scope.build)(&cx)?)
        } else {
            return Err(Error::InvalidView("unknown structural scope"));
        }
        .key(Key(id))
        .actor_root();
        let mut nodes = Vec::new();
        let mut events = Vec::new();
        view.flatten_events(0, &mut nodes, &mut events)?;
        for (event, callback) in events {
            if actions.iter().any(|action| action.id == event) {
                return Err(Error::InvalidView(
                    "closure event conflicts with declared action",
                ));
            }
            __events::register(cx.api.event_owner, id, event, callback)?;
        }
        for node in nodes {
            let status = unsafe { emit(context, node) };
            if status != abi::OK {
                return Err(status.into());
            }
        }
        view.emit_bindings(context, emit_binding, cx.api.event_owner, id)
    })
}

pub(super) unsafe fn computed_specs(
    specs: &[ComputedSpec],
    context: *mut std::ffi::c_void,
    emit: abi::EmitComputed,
) -> i32 {
    guard(|| {
        for spec in specs {
            let kind = match spec.compute {
                Compute::Float(_) => abi::F64,
                Compute::Integer(_) => abi::I64,
                Compute::Boolean(_) => abi::BOOLEAN,
                Compute::Text(_) => abi::TEXT,
                Compute::List(_) => abi::I64_LIST,
                Compute::TextList(_) => abi::TEXT_LIST,
            };
            let status = unsafe {
                emit(
                    context,
                    abi::ComputedSpec {
                        key: spec.key,
                        kind,
                    },
                )
            };
            if status != abi::OK {
                return Err(status.into());
            }
        }
        Ok(())
    })
}

pub(super) unsafe fn compute(
    specs: &[ComputedSpec],
    api: *const HostApi,
    key: u64,
    context: *mut std::ffi::c_void,
    write: abi::WriteValue,
) -> i32 {
    guard_with_api(unsafe { &*api }, || {
        let cx = Cx {
            api: unsafe { &*api },
            instance: InstancePath::EMPTY,
        };
        let spec = specs
            .iter()
            .find(|s| s.key == key)
            .ok_or(Error::InvalidView("missing computed"))?;
        let cx = Cx {
            instance: spec.instance,
            ..cx
        };
        let text;
        let list;
        let text_list;
        let descriptors;
        let output = match spec.compute {
            Compute::Float(compute) => abi::Output {
                kind: abi::F64,
                integer: compute(&cx)?.to_bits() as i64,
                text: abi::Bytes::new(""),
            },
            Compute::Boolean(compute) => abi::Output {
                kind: abi::BOOLEAN,
                integer: i64::from(compute(&cx)?),
                text: abi::Bytes::new(""),
            },
            Compute::Integer(compute) => abi::Output {
                kind: abi::I64,
                integer: compute(&cx)?,
                text: abi::Bytes::new(""),
            },
            Compute::List(compute) => {
                list = compute(&cx)?;
                if list.len() > abi::MAX_LIST_ITEMS {
                    return Err(abi::ERROR.into());
                }
                abi::Output {
                    kind: abi::I64_LIST,
                    integer: 0,
                    text: abi::Bytes {
                        ptr: list.as_ptr().cast(),
                        len: list.len() * 8,
                    },
                }
            }
            Compute::TextList(compute) => {
                text_list = compute(&cx)?;
                if text_list.len() > abi::MAX_LIST_ITEMS {
                    return Err(abi::ERROR.into());
                }
                let total = text_list
                    .iter()
                    .try_fold(0_usize, |sum, item| sum.checked_add(item.len()))
                    .ok_or(Error::Overflow)?;
                if total > abi::MAX_TEXT_LIST_BYTES {
                    return Err(abi::ERROR.into());
                }
                descriptors = text_list
                    .iter()
                    .map(|item| abi::Bytes::new(item))
                    .collect::<Vec<_>>();
                abi::Output {
                    kind: abi::TEXT_LIST,
                    integer: 0,
                    text: abi::Bytes {
                        ptr: descriptors.as_ptr().cast(),
                        len: std::mem::size_of_val(descriptors.as_slice()),
                    },
                }
            }
            Compute::Text(compute) => {
                text = compute(&cx)?;
                abi::Output {
                    kind: abi::TEXT,
                    integer: 0,
                    text: abi::Bytes::new(&text),
                }
            }
        };
        let status = unsafe { write(context, output) };
        if status != abi::OK {
            return Err(status.into());
        }
        Ok(())
    })
}

pub(super) unsafe fn handle(actions: &[Action], api: *const HostApi, id: u64, value: i64) -> i32 {
    guard_with_api(unsafe { &*api }, || {
        let cx = Cx {
            api: unsafe { &*api },
            instance: InstancePath::EMPTY,
        };
        if let Some(callback) = __events::lookup(cx.api.event_owner, id) {
            return callback(&cx, value);
        }
        let action = actions
            .iter()
            .find(|action| action.id == id)
            .ok_or(Error::UnknownAction)?;
        let cx = Cx {
            instance: action.instance,
            ..cx
        };
        action.handler.call(&cx, value)
    })
}

pub(super) unsafe fn view(
    actions: &[Action],
    build: for<'a> fn(&Cx<'a>) -> Result<View>,
    api: *const HostApi,
    context: *mut std::ffi::c_void,
    emit: abi::Emit,
    emit_binding: abi::EmitBinding,
) -> i32 {
    guard_with_api(unsafe { &*api }, || {
        let mut ids = std::collections::BTreeSet::new();
        if actions.len() > 256
            || actions
                .iter()
                .any(|action| action.id == 0 || !ids.insert(action.id))
        {
            return Err(Error::InvalidView("duplicate/zero action identity"));
        }
        let cx = Cx {
            api: unsafe { &*api },
            instance: InstancePath::EMPTY,
        };
        if cx.api.event_owner != 0 {
            __events::lifecycle(cx.api.event_owner, 5, &[0])?;
        }
        let view = build(&cx)?.actor_root();
        let mut nodes = Vec::new();
        let mut events = Vec::new();
        view.flatten_events(0, &mut nodes, &mut events)?;
        for (id, callback) in events {
            if actions.iter().any(|action| action.id == id) {
                return Err(Error::InvalidView(
                    "closure event conflicts with declared action",
                ));
            }
            __events::register(cx.api.event_owner, 0, id, callback)?;
        }
        for node in nodes {
            let status = unsafe { emit(context, node) };
            if status != abi::OK {
                return Err(status.into());
            }
        }
        view.emit_bindings(context, emit_binding, cx.api.event_owner, 0)?;
        Ok(())
    })
}

// Keep panic capture and status conversion out of each actor's monomorphized shim.
// The function pointer is used synchronously and stays in the implementation.
pub(super) unsafe fn migrate(
    migration: for<'a> fn(&Cx<'a>, u64) -> Result<()>,
    api: *const HostApi,
    old_schema: u64,
) -> i32 {
    guard_with_api(unsafe { &*api }, || {
        migration(
            &Cx {
                api: unsafe { &*api },
                instance: InstancePath::EMPTY,
            },
            old_schema,
        )
    })
}
