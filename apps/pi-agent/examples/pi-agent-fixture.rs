use std::fs::{OpenOptions, read_to_string, write};
use std::io::{self, BufRead as _, Write as _};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use serde_json::{Value, json};

struct FixtureState {
    model_provider: String,
    model_id: String,
    model_name: String,
    thinking: String,
    session_id: String,
    session_file: PathBuf,
    session_name: String,
    last_prompt: Option<String>,
    session_serial: u32,
    pending_response: bool,
    pending_extension_choice: bool,
    subagents_enabled: bool,
}

impl FixtureState {
    fn from_args() -> io::Result<Self> {
        let args = std::env::args_os().collect::<Vec<_>>();
        let argument = |name: &str| {
            args.windows(2)
                .find(|pair| pair[0] == name)
                .and_then(|pair| pair[1].to_str())
                .map(str::to_owned)
        };
        let restored_session = args
            .windows(2)
            .find(|pair| pair[0] == "--session")
            .map(|pair| PathBuf::from(&pair[1]));
        let session_file = restored_session
            .clone()
            .unwrap_or(std::env::current_dir()?.join("wabou-fake-session.jsonl"));
        let session_id = session_file
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("wabou-fake-session")
            .to_owned();
        let current_session_serial = session_id
            .strip_prefix("wabou-fake-session-")
            .and_then(|value| value.parse().ok())
            .unwrap_or(1);
        let session_serial = current_session_serial.max(highest_session_serial(
            session_file
                .parent()
                .unwrap_or_else(|| std::path::Path::new(".")),
        ));
        let last_prompt = if restored_session.is_some() {
            read_to_string(&session_file)
                .ok()
                .filter(|value| !value.is_empty())
        } else {
            write(&session_file, [])?;
            None
        };
        let model_provider = argument("--provider").unwrap_or_else(|| "wabou".to_owned());
        let model_id = argument("--model").unwrap_or_else(|| "fake-model".to_owned());
        let model_name = if model_id == "fake-model" {
            "Fake model".to_owned()
        } else {
            model_id.clone()
        };
        Ok(Self {
            model_provider,
            model_id,
            model_name,
            thinking: "medium".to_owned(),
            session_id,
            session_file,
            session_name: format!("Deterministic test {current_session_serial}"),
            last_prompt,
            session_serial,
            pending_response: false,
            pending_extension_choice: false,
            subagents_enabled: argument("--extension")
                .is_some_and(|value| value == "npm:pi-subagents@0.58.0"),
        })
    }

    fn persist_prompt(&mut self, message: &str) -> io::Result<()> {
        write(&self.session_file, message)?;
        self.last_prompt = Some(message.to_owned());
        Ok(())
    }

    fn create_session(&mut self) -> io::Result<()> {
        self.session_serial += 1;
        self.session_id = format!("wabou-fake-session-{}", self.session_serial);
        self.session_file = std::env::current_dir()?.join(format!("{}.jsonl", self.session_id));
        self.session_name = format!("Deterministic test {}", self.session_serial);
        write(&self.session_file, [])?;
        self.last_prompt = None;
        self.pending_response = false;
        self.pending_extension_choice = false;
        Ok(())
    }

    fn clone_session(&mut self) -> io::Result<()> {
        let last_prompt = self.last_prompt.clone();
        self.create_session()?;
        if let Some(prompt) = last_prompt {
            self.persist_prompt(&prompt)?;
        }
        Ok(())
    }
}

fn highest_session_serial(directory: &std::path::Path) -> u32 {
    std::fs::read_dir(directory)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            entry
                .path()
                .file_stem()
                .and_then(|stem| stem.to_str())
                .and_then(|stem| stem.strip_prefix("wabou-fake-session-"))
                .and_then(|serial| serial.parse::<u32>().ok())
        })
        .max()
        .unwrap_or(1)
}

fn trace(request: &Value) -> io::Result<()> {
    let Some(path) = std::env::var_os("WABOU_FAKE_PI_TRACE") else {
        return Ok(());
    };
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut file, request)?;
    file.write_all(b"\n")
}

fn emit(event: &Value) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, event)?;
    stdout.write_all(b"\n")?;
    stdout.flush()
}

fn response(request: &Value, data: Value) -> io::Result<()> {
    emit(&json!({
        "id": request.get("id"),
        "type": "response",
        "command": request.get("type").and_then(Value::as_str),
        "success": true,
        "data": data,
    }))
}

fn answer_prompt(state: &mut FixtureState, message: &str) -> io::Result<()> {
    emit(&json!({"type":"agent_start"}))?;
    emit(&json!({
        "type":"message_start",
        "message":{"role":"assistant","content":[]}
    }))?;
    if message == "Exit fixture" {
        return Err(io::Error::other("deterministic fixture process exit"));
    }
    if message == "Wait for abort" {
        state.pending_response = true;
        return emit(&json!({
            "type":"message_update",
            "assistantMessageEvent":{
                "type":"thinking_delta",
                "delta":"Waiting for the deterministic abort request."
            }
        }));
    }
    if message == "Exercise extension UI" {
        state.pending_extension_choice = true;
        return emit(&json!({
            "type":"extension_ui_request",
            "id":"fixture-choice",
            "method":"select",
            "title":"Choose fixture mode",
            "message":"The deterministic extension is waiting for a native UI response.",
            "options":["Careful","Fast"]
        }));
    }
    state.persist_prompt(message)?;
    emit(&json!({
        "type":"message_update",
        "assistantMessageEvent":{
            "type":"thinking_delta",
            "delta":"Inspecting the deterministic test workspace. "
        }
    }))?;
    emit(&json!({
        "type":"tool_execution_start",
        "toolCallId":"fake-read",
        "toolName":"read",
        "args":{"path":"README.md"}
    }))?;
    thread::sleep(Duration::from_millis(15));
    emit(&json!({
        "type":"tool_execution_end",
        "toolCallId":"fake-read",
        "result":{"content":[{"type":"text","text":"Fixture file inspected."}]}
    }))?;
    emit(&json!({
        "type":"message_update",
        "assistantMessageEvent":{"type":"text_delta","delta":"Fake Pi completed: "}
    }))?;
    thread::sleep(Duration::from_millis(15));
    emit(&json!({
        "type":"message_update",
        "assistantMessageEvent":{"type":"text_delta","delta":message}
    }))?;
    emit(&json!({"type":"message_end"}))?;
    emit(&json!({"type":"agent_end"}))?;
    emit(&json!({"type":"agent_settled"}))
}

fn answer_image_prompt(image_count: usize) -> io::Result<()> {
    emit(&json!({"type":"agent_start"}))?;
    emit(&json!({
        "type":"message_start",
        "message":{"role":"assistant","content":[]}
    }))?;
    emit(&json!({
        "type":"message_update",
        "assistantMessageEvent":{
            "type":"text_delta",
            "delta":format!("Fixture received {image_count} image attachment")
        }
    }))?;
    emit(&json!({"type":"message_end"}))?;
    emit(&json!({"type":"agent_end"}))?;
    emit(&json!({"type":"agent_settled"}))
}

fn handle(request: &Value, state: &mut FixtureState) -> io::Result<()> {
    match request
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default()
    {
        "get_state" => response(
            request,
            json!({
                "model":{
                    "provider":state.model_provider,
                    "id":state.model_id,
                    "name":state.model_name
                },
                "thinkingLevel":state.thinking,
                "sessionId":state.session_id,
                "sessionFile":state.session_file,
                "sessionName":state.session_name,
                "autoCompactionEnabled":true,
                "steeringMode":"one-at-a-time",
                "followUpMode":"one-at-a-time"
            }),
        ),
        "get_available_models" => response(
            request,
            json!({"models":[{
                "provider":state.model_provider,
                "id":state.model_id,
                "name":state.model_name
            },{
                "provider":"wabou","id":"alternative-model","name":"Alternative model"
            }]}),
        ),
        "get_available_thinking_levels" => {
            response(request, json!({"levels":["off","medium","high"]}))
        }
        "get_commands" => {
            let mut commands = vec![json!({
                "name":"fixture",
                "source":"test",
                "description":"Run a deterministic fixture command"
            })];
            if state.subagents_enabled {
                commands.push(json!({
                    "name":"subagents",
                    "source":"extension",
                    "description":"Administer isolated Pi subagents"
                }));
            }
            response(request, json!({"commands":commands}))
        }
        "get_messages" => response(
            request,
            json!({"messages":state.last_prompt.as_ref().map_or_else(Vec::new, |message| vec![
                json!({"role":"user","content":[{"type":"text","text":message}]}),
                json!({"role":"assistant","content":[{
                    "type":"text",
                    "text":format!("Fake Pi completed: {message}")
                }]})
            ])}),
        ),
        "get_fork_messages" => response(
            request,
            json!({
                "messages":state.last_prompt.as_ref().map_or_else(Vec::new, |message| vec![
                    json!({"entryId":"fixture-user-entry","text":message})
                ])
            }),
        ),
        "get_session_stats" => response(
            request,
            json!({"inputTokens":12,"outputTokens":8,"totalTokens":20}),
        ),
        "set_model" => {
            let id = request
                .get("modelId")
                .and_then(Value::as_str)
                .unwrap_or("fake-model");
            state.model_provider = request
                .get("provider")
                .and_then(Value::as_str)
                .unwrap_or("wabou")
                .to_owned();
            state.model_id = id.to_owned();
            state.model_name = if id == "alternative-model" {
                "Alternative model".to_owned()
            } else {
                "Fake model".to_owned()
            };
            response(
                request,
                json!({
                    "provider":state.model_provider,
                    "id":state.model_id,
                    "name":state.model_name
                }),
            )
        }
        "set_thinking_level" => {
            state.thinking = request
                .get("level")
                .and_then(Value::as_str)
                .unwrap_or("medium")
                .to_owned();
            response(request, json!({"level":state.thinking}))
        }
        "prompt" | "steer" | "follow_up" => {
            let message = request
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            let image_count = request
                .get("images")
                .and_then(Value::as_array)
                .map_or(0, Vec::len);
            if image_count == 0 {
                answer_prompt(state, &message)
            } else {
                answer_image_prompt(image_count)
            }
        }
        "new_session" => {
            state.create_session()?;
            emit(&json!({
                "type":"new_session",
                "sessionId":state.session_id
            }))
        }
        "set_session_name" => {
            state.session_name = request
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            response(request, json!({"name":state.session_name}))
        }
        "clone" => {
            state.clone_session()?;
            response(request, json!({"cancelled":false}))
        }
        "fork" => {
            let entry_id = request
                .get("entryId")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if entry_id != "fixture-user-entry" {
                return response(request, json!({"cancelled":true}));
            }
            response(request, json!({"cancelled":false,"text":state.last_prompt}))
        }
        "compact" => response(request, json!({"compacted":true})),
        "export_html" => {
            let path = request
                .get("outputPath")
                .and_then(Value::as_str)
                .ok_or_else(|| io::Error::other("export path missing"))?;
            write(
                path,
                format!(
                    "<!doctype html><title>{}</title><p>{}</p>",
                    state.session_name,
                    state.last_prompt.as_deref().unwrap_or_default()
                ),
            )?;
            response(request, json!({"path":path}))
        }
        "abort" => {
            if state.pending_response {
                state.pending_response = false;
                emit(&json!({"type":"message_end"}))?;
                emit(&json!({"type":"agent_end"}))?;
            }
            emit(&json!({"type":"agent_settled"}))
        }
        "extension_ui_response" => {
            if !state.pending_extension_choice
                || request.get("id").and_then(Value::as_str) != Some("fixture-choice")
            {
                return Ok(());
            }
            state.pending_extension_choice = false;
            let choice = request
                .get("value")
                .and_then(Value::as_str)
                .unwrap_or("cancelled");
            emit(&json!({
                "type":"message_update",
                "assistantMessageEvent":{
                    "type":"text_delta",
                    "delta":format!("Extension UI selected: {choice}")
                }
            }))?;
            emit(&json!({"type":"message_end"}))?;
            emit(&json!({"type":"agent_end"}))?;
            emit(&json!({"type":"agent_settled"}))
        }
        _ => response(request, json!({})),
    }
}

fn main() -> io::Result<()> {
    let mut state = FixtureState::from_args()?;
    for line in io::stdin().lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<Value>(&line) {
            Ok(request) => {
                trace(&request)?;
                handle(&request, &mut state)?;
            }
            Err(error) => emit(&json!({
                "type":"bridge_error",
                "message":format!("invalid fixture request: {error}")
            }))?,
        }
    }
    Ok(())
}
