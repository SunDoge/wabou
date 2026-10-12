//! Host-owned recipes; code tokens are owned only by their Loaded implementation.
use super::*;
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Recipe {
    /// Resolved only through the owning Loaded implementation.
    Style(usize),
    Icon(Box<Recipe>, std::sync::Arc<blockui_abi::icon::IconData>),
    IconResource(std::sync::Arc<blockui_abi::icon::IconData>),
    Styled(Box<Recipe>, usize),
    Static(std::sync::Arc<str>),
    Reactive(usize),
    CapturedText(usize),
    Mapped(u32, u64, usize),
    Text(u64, Option<std::sync::Arc<str>>),
    ComputedText(u64),
    ComputedEditor(u64, Option<std::sync::Arc<str>>),
    RecordText(u64, std::sync::Arc<str>, Option<std::sync::Arc<str>>),
    RecordMapText {
        keyed: bool,
        slot: u64,
        key: std::sync::Arc<str>,
        format: std::sync::Arc<str>,
        index: usize,
        label: Option<std::sync::Arc<str>>,
    },
    RecordScalar(u64, std::sync::Arc<str>, usize),
    RecordSlider(u64, std::sync::Arc<str>, std::sync::Arc<str>, IntegerRange),
    RecordCheckbox(u64, std::sync::Arc<str>, std::sync::Arc<str>),
    Scope(u64),
    Scoped(Box<Recipe>, Box<Recipe>),
    CapturedScope {
        id: u64,
        token: usize,
    },
    ComputedScalar(u32, u64),
    Checkbox(u64, std::sync::Arc<str>),
    Slider(u64, std::sync::Arc<str>, IntegerRange),
    Select(u64, std::sync::Arc<str>),
    RecordSelect(u64, std::sync::Arc<str>, std::sync::Arc<str>),
    Progress(u64, std::sync::Arc<str>),
    RecordProgress(u64, std::sync::Arc<str>, std::sync::Arc<str>),
    Canvas(blockui_abi::canvas::CanvasScene),
}
impl Recipe {
    pub(super) fn scope_marker(&self) -> &Self {
        match self {
            Self::Styled(inner, _) | Self::Icon(inner, _) | Self::Scoped(inner, _) => {
                inner.scope_marker()
            }
            other => other,
        }
    }
    pub(super) fn scoped_recipe(&self) -> Option<&Self> {
        match self {
            Self::Styled(inner, _) | Self::Icon(inner, _) => inner.scoped_recipe(),
            Self::Scoped(..) => Some(self),
            _ => None,
        }
    }
    pub(super) fn scope_id(&self) -> Option<u64> {
        match self.scope_marker() {
            Self::Scope(id) | Self::CapturedScope { id, .. } => Some(*id),
            _ => None,
        }
    }
    pub(super) fn scope_token(&self) -> Option<usize> {
        match self.scope_marker() {
            Self::CapturedScope { token, .. } => Some(*token),
            _ => None,
        }
    }
    pub(super) fn captured_tokens(&self, tokens: &mut Vec<u64>) {
        match self {
            Self::Styled(text, token) => {
                text.captured_tokens(tokens);
                tokens.push(*token as u64);
            }
            Self::Style(token) | Self::CapturedText(token) | Self::CapturedScope { token, .. } => {
                tokens.push(*token as u64)
            }
            Self::Icon(text, _) => text.captured_tokens(tokens),
            Self::Scoped(scope, text) => {
                scope.captured_tokens(tokens);
                text.captured_tokens(tokens);
            }
            _ => {}
        }
    }

    pub(super) fn text(&self) -> &Self {
        match self {
            Self::Styled(text, _) | Self::Icon(text, _) | Self::Scoped(_, text) => text.text(),
            other => other,
        }
    }
    pub(super) fn icon(&self) -> Option<std::sync::Arc<blockui_abi::icon::IconData>> {
        match self {
            Self::Icon(_, icon) => Some(icon.clone()),
            Self::Styled(text, _) => text.icon(),
            _ => None,
        }
    }
    fn text_mut(&mut self) -> &mut Self {
        match self {
            Self::Styled(text, _) | Self::Icon(text, _) | Self::Scoped(_, text) => text.text_mut(),
            other => other,
        }
    }
}
pub(super) type Registry = BTreeMap<u64, Recipe>;
#[derive(Default)]
pub(super) struct Collection {
    pub nodes: Vec<Node>,
    pub recipes: Vec<(u64, Recipe)>,
    pub error: Option<String>,
}
pub(super) unsafe extern "C" fn emit_recipe(ctx: *mut c_void, spec: BindingSpec) -> i32 {
    let collection = unsafe { &mut *ctx.cast::<Collection>() };
    let result = (|| {
        let limit = blockui_abi::MAX_VIEW_NODES;
        let class = |recipe: &Recipe| match recipe {
            Recipe::Style(_) => 1,
            Recipe::IconResource(_) => 2,
            _ => 0,
        };
        let incoming = match spec.kind {
            blockui_abi::BIND_STYLE => 1,
            blockui_abi::BIND_ICON => 2,
            _ => 0,
        };
        if collection.recipes.len() >= limit * 3
            || (collection.recipes.len() >= limit
                && collection
                    .recipes
                    .iter()
                    .filter(|(_, recipe)| class(recipe) == incoming)
                    .count()
                    >= limit)
        {
            return Err("view exceeds binding recipe budget".into());
        }
        let recipe = match spec.kind {
            blockui_abi::BIND_STYLE if spec.token != 0 && spec.slot == 0 && spec.text.len == 0 => {
                Recipe::Style(spec.token)
            }
            blockui_abi::BIND_ICON if spec.token == 0 && spec.slot == 0 => {
                let wire = unsafe { copy_bytes(spec.text) }?;
                Recipe::IconResource(
                    blockui_abi::icon::IconData::decode(&wire)
                        .map_err(str::to_owned)?
                        .into(),
                )
            }
            BIND_STATIC if spec.token == 0 && spec.slot == 0 => {
                Recipe::Static(unsafe { copy_bytes(spec.text) }?.into())
            }
            blockui_abi::BIND_CAPTURED_TEXT
                if spec.token != 0 && spec.slot == 0 && spec.text.len == 0 =>
            {
                Recipe::CapturedText(spec.token)
            }
            BIND_REACTIVE if spec.token != 0 && spec.slot == 0 && spec.text.len == 0 => {
                Recipe::Reactive(spec.token)
            }
            kind @ (blockui_abi::BIND_MAP_I64
            | blockui_abi::BIND_MAP_F64
            | blockui_abi::BIND_MAP_BOOL
            | blockui_abi::BIND_MAP_TEXT
            | blockui_abi::BIND_MAP_LIST_I64
            | blockui_abi::BIND_MAP_TEXT_LIST)
                if spec.token != 0 && spec.text.len == 0 =>
            {
                Recipe::Mapped(kind, spec.slot, spec.token)
            }
            blockui_abi::BIND_COMPUTED_TEXT if spec.token == 0 && spec.text.len == 0 => {
                Recipe::ComputedText(spec.slot)
            }
            kind @ (blockui_abi::BIND_COMPUTED_I64
            | blockui_abi::BIND_COMPUTED_F64
            | blockui_abi::BIND_COMPUTED_BOOL)
                if spec.token == 0 && spec.text.len == 0 =>
            {
                Recipe::ComputedScalar(kind, spec.slot)
            }
            blockui_abi::BIND_CAPTURED_SCOPE
                if spec.token != 0 && spec.slot != 0 && spec.text.len == 0 =>
            {
                Recipe::CapturedScope {
                    id: spec.slot,
                    token: spec.token,
                }
            }
            blockui_abi::BIND_SCOPE if spec.token == 0 && spec.text.len == 0 => {
                Recipe::Scope(spec.slot)
            }
            BIND_TEXT_SLOT if spec.token == 0 => {
                let label = unsafe { copy_bytes(spec.text) }?;
                Recipe::Text(spec.slot, (!label.is_empty()).then(|| label.into()))
            }
            BIND_COMPUTED_EDITOR if spec.token == 0 && spec.slot != 0 => {
                let label = unsafe { copy_bytes(spec.text) }?;
                Recipe::ComputedEditor(spec.slot, (!label.is_empty()).then(|| label.into()))
            }
            kind @ (BIND_RECORD_MAP_TEXT | BIND_RECORD_MAP_KEYED_TEXT)
                if spec.slot != 0 && spec.token == 0 =>
            {
                let wire = unsafe { copy_bytes(spec.text) }?;
                let mut parts = wire.splitn(4, '\n');
                let format = parts.next().ok_or("missing record map format")?;
                let ordinal = parts.next().ok_or("missing record map ordinal")?;
                let length = parts.next().ok_or("missing record map key length")?;
                let tail = parts.next().ok_or("missing record map key")?;
                let index: usize = ordinal.parse().map_err(|_| "invalid record map ordinal")?;
                let size: usize = length
                    .parse()
                    .map_err(|_| "invalid record map key length")?;
                if format.is_empty()
                    || format.len() > 128
                    || index.to_string() != ordinal
                    || size == 0
                    || size.to_string() != length
                {
                    return Err("invalid record map text selector".into());
                }
                let key = tail.get(..size).ok_or("invalid record map key bytes")?;
                let label = tail.get(size..).ok_or("invalid record map label bytes")?;
                Recipe::RecordMapText {
                    keyed: kind == BIND_RECORD_MAP_KEYED_TEXT,
                    slot: spec.slot,
                    key: key.into(),
                    format: format.into(),
                    index,
                    label: (!label.is_empty()).then(|| label.into()),
                }
            }
            kind @ (BIND_RECORD_TEXT | BIND_RECORD_SCALAR | BIND_RECORD_CHECKBOX
            | BIND_RECORD_SLIDER | BIND_RECORD_PROGRESS | BIND_RECORD_SELECT)
                if spec.slot != 0 =>
            {
                if (kind != BIND_RECORD_SCALAR && spec.token != 0)
                    || (kind == BIND_RECORD_SCALAR
                        && !matches!(
                            spec.token,
                            RECORD_DISPLAY_I64
                                | RECORD_DISPLAY_U64
                                | RECORD_DISPLAY_F64
                                | RECORD_DISPLAY_BOOL
                        ))
                {
                    return Err("invalid record display kind".into());
                }
                if kind == BIND_RECORD_SELECT
                    && spec.text.len
                        > blockui_abi::select::MAX_WIRE_BYTES
                            + blockui_abi::select::MAX_LABEL_BYTES
                            + 256
                {
                    return Err("record select control exceeds limit".into());
                }
                let payload = unsafe { copy_bytes(spec.text) }?;
                if let Some(wire) = payload.strip_prefix("\n@record-list-binding1\n") {
                    if !matches!(kind, BIND_RECORD_TEXT | BIND_RECORD_CHECKBOX) {
                        return Err("list selector used for an unsupported control".into());
                    }
                    let (length, tail) = wire
                        .split_once('\n')
                        .ok_or("missing list selector length")?;
                    let size: usize = length.parse().map_err(|_| "invalid list selector length")?;
                    if size == 0 || size > 1024 || size.to_string() != length {
                        return Err("invalid list text selector size".into());
                    }
                    let selector = tail.get(..size).ok_or("invalid list selector bytes")?;
                    if !selector.starts_with("\n@record-list1\n") {
                        return Err("invalid list selector prefix".into());
                    }
                    let label = tail.get(size..).ok_or("invalid list field label")?;
                    if kind == BIND_RECORD_CHECKBOX {
                        Recipe::RecordCheckbox(spec.slot, selector.into(), label.into())
                    } else {
                        Recipe::RecordText(
                            spec.slot,
                            selector.into(),
                            (!label.is_empty()).then(|| label.into()),
                        )
                    }
                } else {
                    let mut parts = payload.splitn(3, '\n');
                    let format = parts.next().ok_or("missing record format")?;
                    let ordinal = parts.next().ok_or("missing record field ordinal")?;
                    let label = parts.next().ok_or("missing record field label")?;
                    let index: usize = ordinal
                        .parse()
                        .map_err(|_| "invalid record field ordinal")?;
                    if format.is_empty() || format.len() > 128 || index.to_string() != ordinal {
                        return Err("invalid record field selector".into());
                    }
                    if kind == BIND_RECORD_SCALAR {
                        if !label.is_empty() {
                            return Err("scalar record binding cannot have an editor label".into());
                        }
                        Recipe::RecordScalar(
                            spec.slot,
                            format!("{format}\n{index}").into(),
                            spec.token,
                        )
                    } else if kind == BIND_RECORD_SELECT {
                        blockui_abi::select::decode_control(label).map_err(str::to_owned)?;
                        Recipe::RecordSelect(
                            spec.slot,
                            format!("{format}\n{index}").into(),
                            label.into(),
                        )
                    } else if kind == BIND_RECORD_PROGRESS {
                        Recipe::RecordProgress(
                            spec.slot,
                            format!("{format}\n{index}").into(),
                            label.into(),
                        )
                    } else if kind == BIND_RECORD_SLIDER {
                        Recipe::RecordSlider(
                            spec.slot,
                            format!("{format}\n{index}").into(),
                            label.into(),
                            IntegerRange::default(),
                        )
                    } else if kind == BIND_RECORD_CHECKBOX {
                        Recipe::RecordCheckbox(
                            spec.slot,
                            format!("{format}\n{index}").into(),
                            label.into(),
                        )
                    } else {
                        Recipe::RecordText(
                            spec.slot,
                            format!("{format}\n{index}").into(),
                            (!label.is_empty()).then(|| label.into()),
                        )
                    }
                }
            }
            BIND_CANVAS if spec.token == 0 && spec.slot == 0 => {
                if spec.text.len > blockui_abi::canvas::MAX_SCENE_BYTES {
                    return Err("canvas scene too large".into());
                }
                let wire = unsafe { copy_bytes(spec.text) }?;
                Recipe::Canvas(
                    blockui_abi::canvas::CanvasScene::from_wire(&wire).map_err(str::to_owned)?,
                )
            }
            BIND_PROGRESS if spec.token == 0 => {
                Recipe::Progress(spec.slot, unsafe { copy_bytes(spec.text) }?.into())
            }
            BIND_SELECT if spec.token == 0 => {
                if spec.text.len
                    > blockui_abi::select::MAX_WIRE_BYTES
                        + blockui_abi::select::MAX_LABEL_BYTES
                        + 64
                {
                    return Err("select control exceeds limit".into());
                }
                let wire = unsafe { copy_bytes(spec.text) }?;
                blockui_abi::select::decode_control(&wire).map_err(str::to_owned)?;
                Recipe::Select(spec.slot, wire.into())
            }
            BIND_SLIDER if spec.token == 0 => Recipe::Slider(
                spec.slot,
                unsafe { copy_bytes(spec.text) }?.into(),
                IntegerRange::default(),
            ),
            BIND_CHECKBOX if spec.token == 0 => {
                Recipe::Checkbox(spec.slot, unsafe { copy_bytes(spec.text) }?.into())
            }
            _ => return Err("invalid binding recipe".into()),
        };
        collection.recipes.push((spec.key, recipe));
        Ok(())
    })();
    match result {
        Ok(()) => OK,
        Err(error) => {
            collection.error = Some(error);
            ERROR
        }
    }
}
pub(super) fn registry(records: Vec<(u64, Recipe)>, nodes: &[Node]) -> Result<Registry, String> {
    let mut recipes = Registry::new();
    for (key, recipe) in records {
        if let Recipe::IconResource(icon) = recipe {
            let text = recipes.remove(&key).ok_or("icon must follow text")?;
            if matches!(
                text,
                Recipe::Icon(..)
                    | Recipe::Styled(..)
                    | Recipe::Scope(_)
                    | Recipe::CapturedScope { .. }
            ) {
                return Err("duplicate or misplaced icon".into());
            }
            recipes.insert(key, Recipe::Icon(Box::new(text), icon));
            continue;
        }
        if let Recipe::Style(token) = recipe {
            let text = recipes
                .remove(&key)
                .ok_or("style must follow its text recipe")?;
            if matches!(text, Recipe::Styled(..)) {
                return Err("duplicate style".into());
            }
            recipes.insert(key, Recipe::Styled(Box::new(text), token));
            continue;
        }
        if recipes.insert(key, recipe).is_some() {
            return Err("duplicate binding recipe".into());
        }
    }
    if recipes.len() != nodes.len() {
        return Err("binding recipes must match view nodes".into());
    }
    for node in nodes {
        if let Some(id) = recipes.get(&node.key).and_then(Recipe::scope_id)
            && (node.key != id || !matches!(node.kind, COLUMN | ROW | DIALOG | SCROLL))
        {
            return Err("scope identity requires a container root".into());
        }
        let icon = recipes.get(&node.key).and_then(Recipe::icon);
        if (node.kind == blockui_abi::ICON && icon.is_none())
            || (icon.is_some() && !matches!(node.kind, BUTTON | blockui_abi::ICON))
        {
            return Err("icon requires an icon or button widget".into());
        }
        if node.kind == BUTTON
            && icon.as_ref().is_some_and(|i| i.only)
            && matches!(recipes.get(&node.key).map(Recipe::text), Some(Recipe::Static(text)) if text.trim().is_empty())
        {
            return Err("icon button requires an accessible label".into());
        }
        let valid = match recipes.get(&node.key).map(Recipe::text) {
            Some(Recipe::Scope(id) | Recipe::CapturedScope { id, .. }) => {
                matches!(node.kind, COLUMN | ROW) && node.key == *id
            }
            Some(Recipe::Text(slot, _)) => {
                matches!(node.kind, ENTRY | TEXTAREA) && *slot == node.slot && *slot != 0
            }
            Some(Recipe::ComputedEditor(_, _)) => {
                matches!(node.kind, ENTRY | TEXTAREA) && node.slot == 0 && node.action != 0
            }
            Some(Recipe::RecordMapText { keyed: true, .. }) => {
                matches!(node.kind, ENTRY | TEXTAREA) && node.slot == 0 && node.action != 0
            }
            Some(Recipe::RecordText(_, _, _) | Recipe::RecordMapText { keyed: false, .. }) => {
                if matches!(node.kind, ENTRY | TEXTAREA) {
                    node.slot == 0 && node.action != 0
                } else {
                    !matches!(node.kind, CHECKBOX | SELECT | SLIDER | PROGRESS | CANVAS)
                }
            }
            Some(Recipe::RecordCheckbox(_, _, _)) => {
                node.kind == CHECKBOX && node.slot == 0 && node.action != 0
            }
            Some(Recipe::RecordSlider(_, _, _, _)) => {
                node.kind == SLIDER && node.slot == 0 && node.action != 0 && node.range.is_valid()
            }
            Some(Recipe::RecordProgress(_, _, _)) => {
                node.kind == PROGRESS && node.slot == 0 && node.action == 0
            }
            Some(Recipe::RecordSelect(_, _, _)) => {
                node.kind == SELECT && node.slot == 0 && node.action != 0
            }
            Some(Recipe::Canvas(_)) => node.kind == CANVAS && node.slot == 0,
            Some(Recipe::Progress(slot, _)) => node.kind == PROGRESS && *slot == node.slot,
            Some(Recipe::Select(slot, _)) => node.kind == SELECT && *slot == node.slot,
            Some(Recipe::Slider(slot, _, _)) => {
                node.kind == SLIDER
                    && *slot == node.slot
                    && *slot != 0
                    && node.range.is_valid()
                    && node.action != 0
            }
            Some(Recipe::Checkbox(slot, _)) => node.kind == CHECKBOX && *slot == node.slot,
            Some(
                Recipe::Static(_)
                | Recipe::Reactive(_)
                | Recipe::CapturedText(_)
                | Recipe::Mapped(_, _, _)
                | Recipe::ComputedText(_)
                | Recipe::ComputedScalar(_, _)
                | Recipe::RecordScalar(_, _, _),
            ) => !matches!(
                node.kind,
                ENTRY | TEXTAREA | CHECKBOX | SELECT | SLIDER | PROGRESS | CANVAS
            ),
            Some(
                Recipe::Style(_)
                | Recipe::Styled(..)
                | Recipe::Icon(..)
                | Recipe::IconResource(_)
                | Recipe::Scoped(..),
            )
            | None => false,
        };
        if !valid {
            return Err("binding recipe does not match widget".into());
        }
        if let Some(Recipe::RecordSlider(_, _, _, range) | Recipe::Slider(_, _, range)) =
            recipes.get_mut(&node.key).map(Recipe::text_mut)
        {
            *range = node.range;
        }
    }
    Ok(recipes)
}
pub(super) fn evaluate(
    recipe: &Recipe,
    loaded: &Loaded,
    turn: &mut Turn,
    output: &mut String,
) -> i32 {
    match recipe {
        Recipe::Style(_) | Recipe::IconResource(_) => ERROR,
        Recipe::Styled(text, _) | Recipe::Icon(text, _) | Recipe::Scoped(_, text) => {
            evaluate(text, loaded, turn, output)
        }
        Recipe::Scope(_) | Recipe::CapturedScope { .. } => OK,
        Recipe::Static(text) => {
            output.push_str(text);
            OK
        }
        Recipe::RecordMapText {
            slot,
            key,
            format,
            index,
            ..
        } => match crate::keyed_map::selected_field(turn, *slot, key, format, *index) {
            Ok(Some(field)) => {
                output.push_str(field);
                OK
            }
            _ => {
                turn.graph.fault =
                    Some("invalid or missing record collection text field binding".into());
                ERROR
            }
        },
        Recipe::RecordText(slot, selector, _) => {
            match crate::record_fields::selected(turn, *slot, selector) {
                Ok(field) => {
                    output.push_str(field);
                    OK
                }
                Err(()) => {
                    turn.graph.fault = Some("invalid record text field binding".into());
                    ERROR
                }
            }
        }
        Recipe::RecordSlider(slot, selector, label, range) => {
            match crate::record_fields::selected(turn, *slot, selector)
                .and_then(crate::record_fields::decode_integer)
            {
                Ok(value) if range.contains(value) => {
                    output.push_str(label);
                    OK
                }
                _ => {
                    turn.graph.fault = Some(
                        "record slider needs an aligned integer field within its range".into(),
                    );
                    ERROR
                }
            }
        }
        Recipe::RecordProgress(slot, selector, _) => {
            match crate::record_fields::selected(turn, *slot, selector)
                .and_then(crate::record_fields::decode_float)
            {
                Ok(value) if value.is_finite() && (0.0..=1.0).contains(&value) => {
                    output.push_str(&value.to_string());
                    OK
                }
                _ => {
                    turn.graph.fault =
                        Some("record progress needs a finite fraction within [0, 1]".into());
                    ERROR
                }
            }
        }
        Recipe::RecordSelect(slot, selector, wire) => {
            let value = crate::record_fields::selected(turn, *slot, selector)
                .and_then(crate::record_fields::decode_integer);
            let options = blockui_abi::select::decode_control(wire);
            match (value, options) {
                (Ok(value), Ok((_, options)))
                    if options.iter().any(|option| option.value == value) =>
                {
                    output.push_str(wire);
                    OK
                }
                _ => {
                    turn.graph.fault =
                        Some("record select needs an integer field matching an option".into());
                    ERROR
                }
            }
        }
        Recipe::RecordCheckbox(slot, selector, label) => {
            match crate::record_fields::selected(turn, *slot, selector) {
                Ok(field @ ("0" | "1")) => {
                    output.push_str(if field == "1" { "☑ " } else { "☐ " });
                    output.push_str(label);
                    OK
                }
                _ => {
                    turn.graph.fault = Some("invalid record checkbox field".into());
                    ERROR
                }
            }
        }
        Recipe::RecordScalar(slot, selector, kind) => {
            let result = crate::record_fields::selected(turn, *slot, selector)
                .and_then(|field| display_record_scalar(field, *kind));
            match result {
                Ok(text) => {
                    output.push_str(&text);
                    OK
                }
                Err(()) => {
                    turn.graph.fault = Some("invalid scalar record field binding".into());
                    ERROR
                }
            }
        }
        Recipe::CapturedText(token) => with_api(turn, loaded.table, |api| unsafe {
            (loaded.table().render_binding)(
                api,
                blockui_abi::BIND_CAPTURED_TEXT,
                0,
                *token,
                (output as *mut String).cast(),
                write,
            )
        }),
        Recipe::Reactive(token) => with_api(turn, loaded.table, |api| unsafe {
            (loaded.table().render_binding)(
                api,
                BIND_REACTIVE,
                0,
                *token,
                (output as *mut String).cast(),
                write,
            )
        }),
        Recipe::Mapped(kind, slot, token) => with_api(turn, loaded.table, |api| unsafe {
            (loaded.table().render_binding)(
                api,
                *kind,
                *slot,
                *token,
                (output as *mut String).cast(),
                write,
            )
        }),
        Recipe::ComputedText(key) | Recipe::ComputedEditor(key, _) => {
            with_api(turn, loaded.table, |api| {
                let context = unsafe { &*api.context.cast::<Context>() };
                match graph::evaluate(context, *key) {
                    Ok(value) => {
                        unsafe { &mut *context.turn }
                            .reads
                            .insert(Dependency::Computed(*key));
                        match value.as_ref() {
                            Value::Text(text) => {
                                output.push_str(text);
                                OK
                            }
                            _ => {
                                graph::fail(context, "computed text binding type mismatch");
                                ERROR
                            }
                        }
                    }
                    Err(_) => ERROR,
                }
            })
        }
        Recipe::ComputedScalar(kind, key) => with_api(turn, loaded.table, |api| {
            let context = unsafe { &*api.context.cast::<Context>() };
            let Ok(value) = graph::evaluate(context, *key) else {
                return ERROR;
            };
            unsafe { &mut *context.turn }
                .reads
                .insert(Dependency::Computed(*key));
            use std::fmt::Write;
            let result = match (*kind, value.as_ref()) {
                (blockui_abi::BIND_COMPUTED_I64, Value::Integer(value)) => {
                    write!(output, "{value}")
                }
                (blockui_abi::BIND_COMPUTED_F64, Value::Float(value)) => write!(output, "{value}"),
                (blockui_abi::BIND_COMPUTED_BOOL, Value::Boolean(value)) => {
                    write!(output, "{value}")
                }
                _ => {
                    graph::fail(context, "computed scalar binding type mismatch");
                    return ERROR;
                }
            };
            if result.is_ok() { OK } else { ERROR }
        }),
        Recipe::Text(slot, _) => {
            turn.reads.insert(Dependency::State(*slot));
            match turn.storage.slots.get(slot).map(Box::as_ref) {
                Some(Value::Text(value)) => {
                    output.push_str(value);
                    OK
                }
                _ => ERROR,
            }
        }
        Recipe::Canvas(scene) => {
            output.push_str(&scene.to_wire());
            OK
        }
        Recipe::Progress(slot, _) => {
            turn.reads.insert(Dependency::State(*slot));
            match turn.storage.slots.get(slot).map(Box::as_ref) {
                Some(Value::Float(value)) if value.is_finite() && (0.0..=1.0).contains(value) => {
                    output.push_str(&value.to_string());
                    OK
                }
                _ => ERROR,
            }
        }
        Recipe::Select(slot, wire) => {
            turn.reads.insert(Dependency::State(*slot));
            let Ok((_, options)) = blockui_abi::select::decode_control(wire) else {
                return ERROR;
            };
            if matches!(turn.storage.slots.get(slot).map(Box::as_ref), Some(Value::Integer(value)) if options.iter().any(|option| option.value == *value))
            {
                output.push_str(wire);
                OK
            } else {
                ERROR
            }
        }
        Recipe::Slider(slot, label, range) => {
            turn.reads.insert(Dependency::State(*slot));
            if matches!(
                turn.storage.slots.get(slot).map(Box::as_ref),
                Some(Value::Integer(value)) if range.contains(*value)
            ) {
                output.push_str(label);
                OK
            } else {
                turn.graph.fault =
                    Some("slider needs an aligned integer state within its range".into());
                ERROR
            }
        }
        Recipe::Checkbox(slot, label) => {
            turn.reads.insert(Dependency::State(*slot));
            match turn.storage.slots.get(slot).map(Box::as_ref) {
                Some(Value::Boolean(value)) => {
                    output.push_str(if *value { "☑ " } else { "☐ " });
                    output.push_str(label);
                    OK
                }
                _ => ERROR,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn node(kind: u32, slot: u64) -> Node {
        Node {
            key: 1,
            parent: 0,
            kind,
            binding: 0,
            shortcut: 0,
            disabled: 0,
            read_only: 0,
            button_variant: 0,
            presentation: Default::default(),
            semantics: Default::default(),
            action: 0,
            secondary_action: 0,
            submit_action: 0,
            cancel_action: 0,
            grid_rows: 0,
            grid_columns: 0,
            navigate_action: 0,
            scroll_action: 0,
            viewport_action: 0,
            focus_boundary: 0,
            action_payload: 0,
            slot,
            gap: 0.0,
            padding: 0.0,
            height: 0.0,
            content_width: 0.0,
            range: Default::default(),
            overlay_position: 0,
            tooltip_anchor: 0,
            tooltip_delay_ms: 0,
        }
    }
    #[test]
    fn rejects_missing_duplicate_extra_and_wrong_widget_recipes() {
        let input = node(ENTRY, 2);
        assert!(registry(vec![], &[input]).is_err());
        assert!(
            registry(
                vec![(1, Recipe::Text(2, None)), (1, Recipe::Text(2, None))],
                &[input]
            )
            .is_err()
        );
        assert!(
            registry(
                vec![
                    (1, Recipe::Text(2, None)),
                    (9, Recipe::Static("extra".into()))
                ],
                &[input]
            )
            .is_err()
        );
        assert!(registry(vec![(1, Recipe::Text(3, None))], &[input]).is_err());
        assert!(registry(vec![(1, Recipe::Static("wrong".into()))], &[input]).is_err());
        assert!(registry(vec![(1, Recipe::Reactive(1))], &[node(CHECKBOX, 2)]).is_err());
        for kind in [ENTRY, TEXTAREA, CHECKBOX, SELECT, SLIDER, PROGRESS, CANVAS] {
            assert!(
                registry(
                    vec![(
                        1,
                        Recipe::RecordScalar(42, "schema\n0".into(), RECORD_DISPLAY_I64)
                    )],
                    &[node(kind, 0)]
                )
                .is_err()
            );
        }
        assert!(
            registry(
                vec![(
                    1,
                    Recipe::RecordScalar(42, "schema\n0".into(), RECORD_DISPLAY_I64)
                )],
                &[node(LABEL, 0)]
            )
            .is_ok()
        );
        assert!(
            registry(
                vec![(1, Recipe::Checkbox(2, "label".into()))],
                &[node(CHECKBOX, 2)]
            )
            .is_ok()
        );
    }
    #[test]
    fn record_select_wire_validates_options_and_controlled_target() {
        let wire = blockui_abi::select::encode_control(
            "Theme\n選択",
            &[blockui_abi::select::OptionItem {
                value: 0,
                label: "Automatic".into(),
                enabled: true,
            }],
        )
        .unwrap();
        let payload = format!("schema\n0\n{wire}");
        let spec = BindingSpec {
            key: 1,
            kind: BIND_RECORD_SELECT,
            token: 0,
            slot: 42,
            text: Bytes::new(&payload),
        };
        let mut collection = Collection::default();
        assert_eq!(
            unsafe { emit_recipe(std::ptr::from_mut(&mut collection).cast(), spec) },
            OK
        );
        let mut select = node(SELECT, 0);
        select.action = 7;
        assert!(registry(collection.recipes.clone(), &[select]).is_ok());
        select.action = 0;
        assert!(registry(collection.recipes.clone(), &[select]).is_err());
        select.action = 7;
        select.slot = 42;
        assert!(registry(collection.recipes.clone(), &[select]).is_err());
        assert!(registry(collection.recipes, &[node(LABEL, 0)]).is_err());
        for bad in [
            BindingSpec {
                token: usize::MAX,
                ..spec
            },
            BindingSpec { slot: 0, ..spec },
            BindingSpec {
                text: Bytes::new("schema\n0\ninvalid"),
                ..spec
            },
        ] {
            let mut collection = Collection::default();
            assert_eq!(
                unsafe { emit_recipe(std::ptr::from_mut(&mut collection).cast(), bad) },
                ERROR
            );
        }
    }
    #[test]
    fn record_select_rejects_oversize_payload_before_copying() {
        let mut collection = Collection::default();
        let spec = BindingSpec {
            key: 1,
            kind: BIND_RECORD_SELECT,
            token: 0,
            slot: 42,
            text: Bytes {
                ptr: std::ptr::null(),
                len: blockui_abi::select::MAX_WIRE_BYTES
                    + blockui_abi::select::MAX_LABEL_BYTES
                    + 257,
            },
        };
        assert_eq!(
            unsafe { emit_recipe(std::ptr::from_mut(&mut collection).cast(), spec) },
            ERROR
        );
        assert_eq!(
            collection.error.as_deref(),
            Some("record select control exceeds limit")
        );
    }
    #[test]
    fn record_progress_protocol_requires_readonly_fraction_target() {
        let spec = BindingSpec {
            key: 1,
            kind: BIND_RECORD_PROGRESS,
            token: 0,
            slot: 42,
            text: Bytes::new("schema\n0\nDownload"),
        };
        let mut collection = Collection::default();
        assert_eq!(
            unsafe { emit_recipe(std::ptr::from_mut(&mut collection).cast(), spec) },
            OK
        );
        let mut progress = node(PROGRESS, 0);
        assert!(registry(collection.recipes.clone(), &[progress]).is_ok());
        progress.action = 7;
        assert!(registry(collection.recipes.clone(), &[progress]).is_err());
        progress.action = 0;
        progress.slot = 42;
        assert!(registry(collection.recipes.clone(), &[progress]).is_err());
        for kind in [LABEL, ENTRY, CHECKBOX, SELECT, SLIDER] {
            assert!(registry(collection.recipes.clone(), &[node(kind, 0)]).is_err());
        }
        for (slot, token) in [(0, 0), (42, 1), (42, usize::MAX)] {
            let mut collection = Collection::default();
            assert_eq!(
                unsafe {
                    emit_recipe(
                        std::ptr::from_mut(&mut collection).cast(),
                        BindingSpec {
                            slot,
                            token,
                            ..spec
                        },
                    )
                },
                ERROR
            );
        }
    }
    #[test]
    fn record_slider_protocol_uses_only_slider_range_and_copied_selector() {
        let spec = BindingSpec {
            key: 1,
            kind: BIND_RECORD_SLIDER,
            token: 0,
            slot: 42,
            text: Bytes::new("schema\n0\nVolume"),
        };
        let mut collection = Collection::default();
        assert_eq!(
            unsafe { emit_recipe(std::ptr::from_mut(&mut collection).cast(), spec) },
            OK
        );
        let mut slider = node(SLIDER, 0);
        slider.action = 7;
        slider.range = IntegerRange {
            min: 0,
            max: 100,
            step: 10,
        };
        let valid = registry(collection.recipes.clone(), &[slider]).unwrap();
        assert!(matches!(valid[&1], Recipe::RecordSlider(_, _, _, range) if range == slider.range));
        slider.range.step = 0;
        assert!(registry(collection.recipes.clone(), &[slider]).is_err());
        for kind in [LABEL, ENTRY, CHECKBOX, SELECT, PROGRESS] {
            assert!(registry(collection.recipes.clone(), &[node(kind, 0)]).is_err());
        }
        for (slot, token) in [(0, 0), (42, 1), (42, usize::MAX)] {
            let mut collection = Collection::default();
            assert_eq!(
                unsafe {
                    emit_recipe(
                        std::ptr::from_mut(&mut collection).cast(),
                        BindingSpec {
                            slot,
                            token,
                            ..spec
                        },
                    )
                },
                ERROR
            );
        }
    }
    #[test]
    fn record_checkbox_protocol_accepts_only_controlled_checkbox_targets() {
        let spec = BindingSpec {
            key: 1,
            kind: BIND_RECORD_CHECKBOX,
            token: 0,
            slot: 42,
            text: Bytes::new("schema\n0\nEnabled"),
        };
        let mut collection = Collection::default();
        assert_eq!(
            unsafe { emit_recipe(std::ptr::from_mut(&mut collection).cast(), spec) },
            OK
        );
        let mut checkbox = node(CHECKBOX, 0);
        checkbox.action = 7;
        assert!(registry(collection.recipes.clone(), &[checkbox]).is_ok());
        checkbox.slot = 42;
        assert!(registry(collection.recipes.clone(), &[checkbox]).is_err());
        checkbox.slot = 0;
        checkbox.action = 0;
        assert!(registry(collection.recipes.clone(), &[checkbox]).is_err());
        for kind in [LABEL, ENTRY, TEXTAREA, SELECT, SLIDER] {
            assert!(registry(collection.recipes.clone(), &[node(kind, 0)]).is_err());
        }
        for (slot, token) in [(0, 0), (42, 1), (42, usize::MAX)] {
            let mut collection = Collection::default();
            assert_eq!(
                unsafe {
                    emit_recipe(
                        std::ptr::from_mut(&mut collection).cast(),
                        BindingSpec {
                            slot,
                            token,
                            ..spec
                        },
                    )
                },
                ERROR
            );
        }
    }
    #[test]
    fn keyed_collection_text_wire_requires_editors_and_remains_non_executable() {
        let spec = BindingSpec {
            key: 1,
            kind: BIND_RECORD_MAP_KEYED_TEXT,
            token: 0,
            slot: 42,
            text: Bytes::new("schema\n0\n1\naName"),
        };
        let mut collection = Collection::default();
        assert_eq!(
            unsafe { emit_recipe(std::ptr::from_mut(&mut collection).cast(), spec) },
            OK
        );
        assert!(matches!(
            collection.recipes[0].1,
            Recipe::RecordMapText { keyed: true, .. }
        ));
        for kind in [ENTRY, TEXTAREA] {
            let mut editor = node(kind, 0);
            editor.action = 7;
            assert!(registry(collection.recipes.clone(), &[editor]).is_ok());
            editor.action = 0;
            assert!(registry(collection.recipes.clone(), &[editor]).is_err());
        }
        for kind in [
            LABEL, COLUMN, ROW, CHECKBOX, SELECT, SLIDER, PROGRESS, CANVAS,
        ] {
            assert!(registry(collection.recipes.clone(), &[node(kind, 0)]).is_err());
        }
        for (slot, token) in [(0, 0), (42, 1)] {
            let mut rejected = Collection::default();
            assert_eq!(
                unsafe {
                    emit_recipe(
                        std::ptr::from_mut(&mut rejected).cast(),
                        BindingSpec {
                            slot,
                            token,
                            ..spec
                        },
                    )
                },
                ERROR
            );
            assert!(rejected.recipes.is_empty());
        }
    }
    #[test]
    fn record_map_text_wire_checks_framed_keys_and_controlled_targets() {
        let spec = BindingSpec {
            key: 1,
            kind: BIND_RECORD_MAP_TEXT,
            token: 0,
            slot: 42,
            text: Bytes::new("schema\n0\n6\na\n🦀Notes\nlabel"),
        };
        let mut collection = Collection::default();
        assert_eq!(
            unsafe { emit_recipe(std::ptr::from_mut(&mut collection).cast(), spec) },
            OK
        );
        assert!(
            matches!(&collection.recipes[0].1, Recipe::RecordMapText { key, label, .. } if key.as_ref() == "a\n🦀" && label.as_deref() == Some("Notes\nlabel"))
        );
        assert!(registry(collection.recipes.clone(), &[node(LABEL, 0)]).is_ok());
        let mut input = node(ENTRY, 0);
        assert!(registry(collection.recipes.clone(), &[input]).is_err());
        input.action = 7;
        for kind in [ENTRY, TEXTAREA] {
            input.kind = kind;
            assert!(registry(collection.recipes.clone(), &[input]).is_ok());
        }
        input.slot = 42;
        assert!(registry(collection.recipes.clone(), &[input]).is_err());
        for kind in [CHECKBOX, SLIDER, SELECT, PROGRESS, CANVAS] {
            assert!(registry(collection.recipes.clone(), &[node(kind, 0)]).is_err());
        }
        for wire in [
            "",
            "schema\n0\n0\n",
            "schema\n00\n1\na",
            "schema\n0\n01\na",
            "schema\n0\n3\n🦀",
            "schema\n0\n9\na",
        ] {
            let mut rejected = Collection::default();
            assert_eq!(
                unsafe {
                    emit_recipe(
                        std::ptr::from_mut(&mut rejected).cast(),
                        BindingSpec {
                            text: Bytes::new(wire),
                            ..spec
                        },
                    )
                },
                ERROR,
                "{wire}"
            );
            assert!(rejected.recipes.is_empty());
        }
        for (slot, token) in [(0, 0), (42, 1), (42, usize::MAX)] {
            let mut rejected = Collection::default();
            assert_eq!(
                unsafe {
                    emit_recipe(
                        std::ptr::from_mut(&mut rejected).cast(),
                        BindingSpec {
                            slot,
                            token,
                            ..spec
                        },
                    )
                },
                ERROR
            );
        }
    }
    #[test]
    fn record_wire_is_data_only_canonical_and_requires_controlled_editors() {
        let spec = BindingSpec {
            key: 1,
            kind: BIND_RECORD_TEXT,
            token: 0,
            slot: 42,
            text: Bytes::new("schema\n0\nNotes\nlabel"),
        };
        let mut collection = Collection::default();
        assert_eq!(
            unsafe { emit_recipe(std::ptr::from_mut(&mut collection).cast(), spec) },
            OK
        );
        assert!(registry(collection.recipes.clone(), &[node(LABEL, 0)]).is_ok());
        assert!(registry(collection.recipes.clone(), &[node(ENTRY, 0)]).is_err());
        let mut input = node(ENTRY, 0);
        input.action = 7;
        assert!(registry(collection.recipes.clone(), &[input]).is_ok());
        input.slot = 42;
        assert!(registry(collection.recipes, &[input]).is_err());
        for text in [
            "\n0\n",
            "schema\n00\n",
            "schema\n+0\n",
            "schema\n184467440737095516160\n",
            "schema\n0",
        ] {
            let mut rejected = Collection::default();
            assert_eq!(
                unsafe {
                    emit_recipe(
                        std::ptr::from_mut(&mut rejected).cast(),
                        BindingSpec {
                            text: Bytes::new(text),
                            ..spec
                        },
                    )
                },
                ERROR
            );
            assert!(rejected.recipes.is_empty());
        }
        for invalid in [
            BindingSpec { token: 1, ..spec },
            BindingSpec { slot: 0, ..spec },
        ] {
            let mut rejected = Collection::default();
            assert_eq!(
                unsafe { emit_recipe(std::ptr::from_mut(&mut rejected).cast(), invalid) },
                ERROR
            );
        }
    }
    #[test]
    fn computed_text_recipe_is_data_only_and_rejects_editor_targets() {
        let spec = BindingSpec {
            key: 1,
            kind: blockui_abi::BIND_COMPUTED_TEXT,
            token: 0,
            slot: 42,
            text: Bytes::new(""),
        };
        let mut collection = Collection::default();
        assert_eq!(
            unsafe { emit_recipe(std::ptr::from_mut(&mut collection).cast(), spec) },
            OK
        );
        assert!(registry(collection.recipes.clone(), &[node(LABEL, 0)]).is_ok());
        assert!(registry(collection.recipes, &[node(ENTRY, 42)]).is_err());
        for invalid in [
            BindingSpec { token: 1, ..spec },
            BindingSpec {
                text: Bytes::new("unexpected"),
                ..spec
            },
        ] {
            let mut rejected = Collection::default();
            assert_eq!(
                unsafe { emit_recipe(std::ptr::from_mut(&mut rejected).cast(), invalid) },
                ERROR
            );
            assert!(rejected.recipes.is_empty());
        }
    }

    #[test]
    fn malformed_wire_recipes_and_registration_limit_fail_without_calling_tokens() {
        let mut collection = Collection::default();
        let base = BindingSpec {
            key: 1,
            kind: BIND_REACTIVE,
            token: 0,
            slot: 0,
            text: Bytes::new(""),
        };
        assert_eq!(
            unsafe { emit_recipe((&mut collection as *mut Collection).cast(), base) },
            ERROR
        );
        assert_eq!(collection.error.as_deref(), Some("invalid binding recipe"));
        collection.error = None;
        let text = BindingSpec {
            kind: BIND_STATIC,
            text: Bytes {
                ptr: std::ptr::null(),
                len: 1,
            },
            ..base
        };
        assert_eq!(
            unsafe { emit_recipe((&mut collection as *mut Collection).cast(), text) },
            ERROR
        );
        assert_eq!(collection.error.as_deref(), Some("invalid byte buffer"));
        collection.error = None;
        let valid = BindingSpec {
            kind: BIND_STATIC,
            ..base
        };
        for _ in 0..blockui_abi::MAX_VIEW_NODES {
            assert_eq!(
                unsafe { emit_recipe((&mut collection as *mut Collection).cast(), valid) },
                OK
            );
        }
        assert_eq!(
            unsafe { emit_recipe((&mut collection as *mut Collection).cast(), valid) },
            ERROR
        );
        assert_eq!(collection.recipes.len(), blockui_abi::MAX_VIEW_NODES);
    }
}

fn display_record_scalar(field: &str, kind: usize) -> Result<String, ()> {
    match kind {
        RECORD_DISPLAY_I64 => {
            let value: i64 = field.parse().map_err(|_| ())?;
            let text = value.to_string();
            if text != field {
                return Err(());
            }
            Ok(text)
        }
        RECORD_DISPLAY_U64 => {
            let value: u64 = field.parse().map_err(|_| ())?;
            let text = value.to_string();
            if text != field {
                return Err(());
            }
            Ok(text)
        }
        RECORD_DISPLAY_BOOL => match field {
            "0" => Ok("false".into()),
            "1" => Ok("true".into()),
            _ => Err(()),
        },
        RECORD_DISPLAY_F64 => Ok(crate::record_fields::decode_float(field)?.to_string()),
        _ => Err(()),
    }
}

#[cfg(test)]
mod scalar_tests {
    use super::*;
    #[test]
    fn rejects_noncanonical_scalars_and_unknown_format_kinds() {
        for (kind, inputs) in [
            (
                RECORD_DISPLAY_I64,
                vec!["+1", "01", "-0", "9223372036854775808"],
            ),
            (RECORD_DISPLAY_U64, vec!["-1", "01", "18446744073709551616"]),
            (RECORD_DISPLAY_BOOL, vec!["true", "2", "01"]),
            (
                RECORD_DISPLAY_F64,
                vec!["0", "3FF0000000000000", "3ff00000000000000"],
            ),
        ] {
            for input in inputs {
                assert!(
                    display_record_scalar(input, kind).is_err(),
                    "{kind}: {input}"
                );
            }
        }
        assert_eq!(
            display_record_scalar("3ff4000000000000", RECORD_DISPLAY_F64),
            Ok("1.25".into())
        );
        assert!(display_record_scalar("1", usize::MAX).is_err());
    }
    #[test]
    fn list_text_wire_bounds_prefix_and_utf8_are_checked() {
        for (kind, wire, expected) in [
            (
                BIND_RECORD_TEXT,
                "\n@record-list-binding1\n15\n\n@record-list1\nLabel",
                OK,
            ),
            (
                BIND_RECORD_TEXT,
                "\n@record-list-binding1\n015\n\n@record-list1\n",
                ERROR,
            ),
            (BIND_RECORD_TEXT, "\n@record-list-binding1\n1025\nx", ERROR),
            (BIND_RECORD_TEXT, "\n@record-list-binding1\n1\nx", ERROR),
            (
                BIND_RECORD_TEXT,
                "\n@record-list-binding1\n16\n\n@record-list1\n🦀",
                ERROR,
            ),
            (
                BIND_RECORD_CHECKBOX,
                "\n@record-list-binding1\n15\n\n@record-list1\n",
                OK,
            ),
            (
                BIND_RECORD_SLIDER,
                "\n@record-list-binding1\n15\n\n@record-list1\n",
                ERROR,
            ),
        ] {
            let mut collection = Collection::default();
            let spec = BindingSpec {
                key: 1,
                kind,
                token: 0,
                slot: 42,
                text: Bytes::new(wire),
            };
            assert_eq!(
                unsafe { emit_recipe(std::ptr::from_mut(&mut collection).cast(), spec) },
                expected
            );
        }
    }
    #[test]
    fn wire_rejects_unknown_tokens_and_editor_labels() {
        for (token, text, expected) in [
            (RECORD_DISPLAY_I64, "schema\n0\n", OK),
            (usize::MAX, "schema\n0\n", ERROR),
            (0, "schema\n0\n", ERROR),
            (RECORD_DISPLAY_BOOL, "schema\n0\nLabel", ERROR),
            (RECORD_DISPLAY_F64, "schema\n00\n", ERROR),
        ] {
            let mut collection = Collection::default();
            let spec = BindingSpec {
                key: 1,
                kind: BIND_RECORD_SCALAR,
                token,
                slot: 42,
                text: Bytes::new(text),
            };
            assert_eq!(
                unsafe { emit_recipe(std::ptr::from_mut(&mut collection).cast(), spec) },
                expected
            );
        }
    }
}
