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
    last_prompt: Option<String>,
    session_serial: u32,
    pending_response: bool,
}

impl FixtureState {
    fn from_args() -> io::Result<Self> {
        let args = std::env::args_os().collect::<Vec<_>>();
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
        let session_serial = session_id
            .strip_prefix("wabou-fake-session-")
            .and_then(|value| value.parse().ok())
            .unwrap_or(1);
        let last_prompt = if restored_session.is_some() {
            read_to_string(&session_file)
                .ok()
                .filter(|value| !value.is_empty())
        } else {
            write(&session_file, [])?;
            None
        };
        Ok(Self {
            model_provider: "wabou".to_owned(),
            model_id: "fake-model".to_owned(),
            model_name: "Fake model".to_owned(),
            thinking: "medium".to_owned(),
            session_id,
            session_file,
            last_prompt,
            session_serial,
            pending_response: false,
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
        write(&self.session_file, [])?;
        self.last_prompt = None;
        self.pending_response = false;
        Ok(())
    }
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
                "sessionName":format!("Deterministic test {}", state.session_serial),
                "autoCompactionEnabled":true,
                "steeringMode":"one-at-a-time",
                "followUpMode":"one-at-a-time"
            }),
        ),
        "get_available_models" => response(
            request,
            json!({"models":[{
                "provider":"wabou","id":"fake-model","name":"Fake model"
            },{
                "provider":"wabou","id":"alternative-model","name":"Alternative model"
            }]}),
        ),
        "get_available_thinking_levels" => {
            response(request, json!({"levels":["off","medium","high"]}))
        }
        "get_commands" => response(
            request,
            json!({"commands":[{
                "name":"fixture",
                "source":"test",
                "description":"Run a deterministic fixture command"
            }]}),
        ),
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
            answer_prompt(state, &message)
        }
        "new_session" => {
            state.create_session()?;
            emit(&json!({
                "type":"new_session",
                "sessionId":state.session_id
            }))
        }
        "abort" => {
            if state.pending_response {
                state.pending_response = false;
                emit(&json!({"type":"message_end"}))?;
                emit(&json!({"type":"agent_end"}))?;
            }
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
