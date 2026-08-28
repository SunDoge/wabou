use std::io::{self, BufRead as _, Write as _};
use std::thread;
use std::time::Duration;

use serde_json::{Value, json};

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

fn handle(request: &Value) -> io::Result<()> {
    match request
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default()
    {
        "get_state" => response(
            request,
            json!({
                "model":{"provider":"wabou","id":"fake-model","name":"Fake model"},
                "thinkingLevel":"medium",
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
    for line in io::stdin().lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<Value>(&line) {
            Ok(request) => handle(&request)?,
            Err(error) => emit(&json!({
                "type":"bridge_error",
                "message":format!("invalid fixture request: {error}")
            }))?,
        }
    }
    Ok(())
}
