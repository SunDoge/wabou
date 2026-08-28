use std::fs::OpenOptions;
use std::io::{self, BufRead as _, Write as _};
use std::thread;
use std::time::Duration;

use serde_json::{Value, json};

struct FixtureState {
    model_provider: String,
    model_id: String,
    model_name: String,
    thinking: String,
}

impl Default for FixtureState {
    fn default() -> Self {
        Self {
            model_provider: "wabou".to_owned(),
            model_id: "fake-model".to_owned(),
            model_name: "Fake model".to_owned(),
            thinking: "medium".to_owned(),
        }
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

fn answer_prompt(message: &str) -> io::Result<()> {
    emit(&json!({"type":"agent_start"}))?;
    emit(&json!({
        "type":"message_start",
        "message":{"role":"assistant","content":[]}
    }))?;
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
                "sessionId":"wabou-fake-session",
                "sessionFile":std::env::current_dir()?.join(".wabou-fake-session.jsonl"),
                "sessionName":"Deterministic test",
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
        "get_messages" => response(request, json!({"messages":[]})),
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
        "prompt" | "steer" | "follow_up" => answer_prompt(
            request
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or_default(),
        ),
        "new_session" => emit(&json!({
            "type":"new_session",
            "sessionId":"wabou-fake-session"
        })),
        "abort" => emit(&json!({"type":"agent_settled"})),
        _ => response(request, json!({})),
    }
}

fn main() -> io::Result<()> {
    let mut state = FixtureState::default();
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
