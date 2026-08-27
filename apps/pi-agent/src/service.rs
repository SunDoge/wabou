use std::{
    collections::HashMap,
    env,
    io::{BufRead as _, BufReader, Write as _},
    path::PathBuf,
    process::{Child, ChildStdin, Command, Stdio},
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use wabou::{
    HostMessage, HostMessageContext, JsonCapability, JsonCapabilityContract, JsonMethod, rquickjs,
};

pub const CAPABILITY: JsonCapabilityContract = JsonCapabilityContract::new("piAgent", 1);
const EVENT_TOPIC: &str = "pi.event";

const GET_STATUS: JsonMethod<AgentRequest, PiStatus> = JsonMethod::new("getStatus");
const START: JsonMethod<StartRequest, PiStatus> = JsonMethod::new("start");
const PROMPT: JsonMethod<PromptRequest, ()> = JsonMethod::new("prompt");
const FOLLOW_UP: JsonMethod<PromptRequest, ()> = JsonMethod::new("followUp");
const ABORT: JsonMethod<AgentRequest, ()> = JsonMethod::new("abort");
const STOP: JsonMethod<AgentRequest, ()> = JsonMethod::new("stop");
const NEW_SESSION: JsonMethod<AgentRequest, ()> = JsonMethod::new("newSession");
const CYCLE_MODEL: JsonMethod<AgentRequest, ()> = JsonMethod::new("cycleModel");
const CYCLE_THINKING: JsonMethod<AgentRequest, ()> = JsonMethod::new("cycleThinking");
const SET_MODEL: JsonMethod<SetModelRequest, ()> = JsonMethod::new("setModel");
const LIST_SESSIONS: JsonMethod<AgentRequest, Vec<PiSession>> = JsonMethod::new("listSessions");
const GET_MESSAGES: JsonMethod<AgentRequest, ()> = JsonMethod::new("getMessages");
const LIST_AGENTS: JsonMethod<(), Vec<AgentProfile>> = JsonMethod::no_request("listAgents");
const SAVE_AGENTS: JsonMethod<Vec<AgentProfile>, ()> = JsonMethod::new("saveAgents");
const DELETE_AGENT: JsonMethod<AgentRequest, ()> = JsonMethod::new("deleteAgent");
const DEFAULT_WORKSPACE: JsonMethod<AgentRequest, String> = JsonMethod::new("defaultWorkspace");

#[derive(Clone)]
pub struct PiService {
    state: Arc<Mutex<HashMap<String, PiProcess>>>,
    events_tx: flume::Sender<Value>,
    events_rx: flume::Receiver<Value>,
    sessions: Arc<Mutex<SessionCatalog>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionCatalog {
    #[serde(default)]
    sessions: Vec<PiSession>,
    #[serde(default)]
    agents: Vec<AgentProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AgentProfile {
    id: String,
    name: String,
    cwd: String,
    proxy: String,
    no_proxy: String,
    provider: String,
    model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PiSession {
    agent_id: String,
    session_id: String,
    session_file: String,
    name: Option<String>,
    cwd: String,
    updated_at: u64,
}

struct PiProcess {
    child: Option<Child>,
    stdin: Option<Arc<Mutex<ChildStdin>>>,
    cwd: Option<PathBuf>,
    last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PiStatus {
    running: bool,
    cwd: Option<String>,
    runtime: &'static str,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StartRequest {
    agent_id: String,
    cwd: Option<String>,
    proxy: Option<String>,
    no_proxy: Option<String>,
    provider: Option<String>,
    model: Option<String>,
    session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PromptRequest {
    #[serde(rename = "agentId")]
    agent_id: String,
    message: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AgentRequest {
    agent_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SetModelRequest {
    agent_id: String,
    provider: String,
    model_id: String,
}

impl PiService {
    pub fn new() -> Self {
        let (events_tx, events_rx) = flume::bounded(1024);
        let sessions = load_session_catalog();
        Self {
            state: Arc::new(Mutex::new(HashMap::new())),
            events_tx,
            events_rx,
            sessions: Arc::new(Mutex::new(sessions)),
        }
    }

    fn status(&self, agent_id: &str) -> Result<PiStatus, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "Pi process lock poisoned".to_owned())?;
        let running = state
            .get_mut(agent_id)
            .and_then(|process| process.child.as_mut())
            .is_some_and(|child| child.try_wait().ok().flatten().is_none());
        if !running {
            state.remove(agent_id);
        }
        let process = state.get(agent_id);
        Ok(PiStatus {
            running,
            cwd: process
                .and_then(|process| process.cwd.as_ref())
                .map(|path| path.display().to_string()),
            runtime: "bun",
            error: process.and_then(|process| process.last_error.clone()),
        })
    }

    fn start(&self, request: StartRequest) -> Result<PiStatus, String> {
        validate_agent_id(&request.agent_id)?;
        self.stop(&request.agent_id)?;
        let cwd = request
            .cwd
            .filter(|value| !value.trim().is_empty())
            .map(PathBuf::from)
            .unwrap_or(env::current_dir().map_err(|error| error.to_string())?);
        if cwd.exists() && !cwd.is_dir() {
            return Err(format!("workspace does not exist: {}", cwd.display()));
        }
        if !cwd.exists() {
            std::fs::create_dir_all(&cwd).map_err(|error| {
                format!("could not create workspace {}: {error}", cwd.display())
            })?;
        }
        let explicit_pi = env::var_os("WABOU_PI_BIN");
        let mut command = match explicit_pi {
            Some(executable) => {
                let mut command = Command::new(executable);
                command.args(["--mode", "rpc"]);
                command
            }
            None => {
                let mut command = Command::new("bun");
                command.args([
                    "x",
                    "--package",
                    "@earendil-works/pi-coding-agent@0.84.3",
                    "pi",
                    "--mode",
                    "rpc",
                ]);
                command
            }
        };
        if let Some(provider) = request.provider.filter(|value| !value.trim().is_empty()) {
            command.args(["--provider", provider.trim()]);
        }
        if let Some(model) = request.model.filter(|value| !value.trim().is_empty()) {
            command.args(["--model", model.trim()]);
        }
        if let Some(session_id) = request.session_id.filter(|value| !value.trim().is_empty()) {
            command.args(["--session", session_id.trim()]);
        } else {
            command.arg("--continue");
        }
        command
            .current_dir(&cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(proxy) = request.proxy.filter(|value| !value.trim().is_empty()) {
            for name in [
                "HTTP_PROXY",
                "HTTPS_PROXY",
                "ALL_PROXY",
                "http_proxy",
                "https_proxy",
                "all_proxy",
            ] {
                command.env(name, &proxy);
            }
        }
        if let Some(no_proxy) = request.no_proxy.filter(|value| !value.trim().is_empty()) {
            command.env("NO_PROXY", &no_proxy).env("no_proxy", no_proxy);
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt as _;
            command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
        }
        let mut child = command.spawn().map_err(|error| {
            format!("could not start Pi through Bun. Install Bun or set WABOU_PI_BIN: {error}")
        })?;
        let stdin = Arc::new(Mutex::new(
            child.stdin.take().ok_or("Pi stdin was not piped")?,
        ));
        let stdout = child.stdout.take().ok_or("Pi stdout was not piped")?;
        let stderr = child.stderr.take().ok_or("Pi stderr was not piped")?;

        let stdout_events = self.events_tx.clone();
        let sessions = self.sessions.clone();
        let session_cwd = cwd.display().to_string();
        let stdout_agent_id = request.agent_id.clone();
        std::thread::Builder::new()
            .name("pi-rpc-stdout".to_owned())
            .spawn(move || {
                for line in BufReader::new(stdout).lines() {
                    match line {
                        Ok(line) if !line.is_empty() => match serde_json::from_str(&line) {
                            Ok(event) => {
                                remember_session(&sessions, &stdout_agent_id, &session_cwd, &event);
                                let _ = stdout_events.send(tag_event(&stdout_agent_id, event));
                            }
                            Err(error) => {
                                let _ = stdout_events.send(tag_event(
                                    &stdout_agent_id,
                                    json!({
                                        "type":"bridge_error",
                                        "message":format!("invalid Pi RPC event: {error}")
                                    }),
                                ));
                            }
                        },
                        Ok(_) => {}
                        Err(error) => {
                            let _ = stdout_events.send(tag_event(
                                &stdout_agent_id,
                                json!({"type":"bridge_error","message":error.to_string()}),
                            ));
                            break;
                        }
                    }
                }
                let _ =
                    stdout_events.send(tag_event(&stdout_agent_id, json!({"type":"process_exit"})));
            })
            .map_err(|error| error.to_string())?;
        let stderr_events = self.events_tx.clone();
        let stderr_agent_id = request.agent_id.clone();
        std::thread::Builder::new()
            .name("pi-rpc-stderr".to_owned())
            .spawn(move || {
                for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                    let _ = stderr_events.send(tag_event(
                        &stderr_agent_id,
                        json!({"type":"process_log","message":line}),
                    ));
                }
            })
            .map_err(|error| error.to_string())?;

        let mut state = self
            .state
            .lock()
            .map_err(|_| "Pi process lock poisoned".to_owned())?;
        state.insert(
            request.agent_id.clone(),
            PiProcess {
                child: Some(child),
                stdin: Some(stdin),
                cwd: Some(cwd),
                last_error: None,
            },
        );
        drop(state);
        self.events_tx
            .send(tag_event(
                &request.agent_id,
                json!({"type":"process_start"}),
            ))
            .map_err(|error| error.to_string())?;
        self.send(
            &request.agent_id,
            json!({"id":"wabou-bootstrap-state","type":"get_state"}),
        )?;
        self.status(&request.agent_id)
    }

    fn send(&self, agent_id: &str, value: Value) -> Result<(), String> {
        let stdin = self
            .state
            .lock()
            .map_err(|_| "Pi process lock poisoned".to_owned())?
            .get(agent_id)
            .and_then(|process| process.stdin.clone())
            .ok_or_else(|| format!("Pi agent `{agent_id}` is not running"))?;
        let mut stdin = stdin.lock().map_err(|_| "Pi stdin lock poisoned")?;
        serde_json::to_writer(&mut *stdin, &value).map_err(|error| error.to_string())?;
        stdin.write_all(b"\n").map_err(|error| error.to_string())?;
        stdin.flush().map_err(|error| error.to_string())
    }

    fn stop(&self, agent_id: &str) -> Result<(), String> {
        let process = self
            .state
            .lock()
            .map_err(|_| "Pi process lock poisoned".to_owned())?
            .remove(agent_id);
        if let Some(mut child) = process.and_then(|mut process| process.child.take()) {
            child.kill().map_err(|error| error.to_string())?;
            let _ = child.wait();
        }
        Ok(())
    }

    fn sessions(&self, agent_id: &str) -> Result<Vec<PiSession>, String> {
        let catalog = self
            .sessions
            .lock()
            .map_err(|_| "Pi session catalog lock poisoned".to_owned())?;
        let mut sessions = catalog
            .sessions
            .iter()
            .filter(|session| session.agent_id == agent_id)
            .cloned()
            .collect::<Vec<_>>();
        sessions.sort_by_key(|session| std::cmp::Reverse(session.updated_at));
        Ok(sessions)
    }

    fn agents(&self) -> Result<Vec<AgentProfile>, String> {
        self.sessions
            .lock()
            .map(|catalog| catalog.agents.clone())
            .map_err(|_| "Pi session catalog lock poisoned".to_owned())
    }

    fn save_agents(&self, agents: Vec<AgentProfile>) -> Result<(), String> {
        validate_agent_profiles(&agents)?;
        let mut catalog = self
            .sessions
            .lock()
            .map_err(|_| "Pi session catalog lock poisoned".to_owned())?;
        catalog.agents = agents;
        persist_catalog(&catalog)
    }

    fn delete_agent(&self, agent_id: &str) -> Result<(), String> {
        validate_agent_id(agent_id)?;
        self.stop(agent_id)?;
        let mut catalog = self
            .sessions
            .lock()
            .map_err(|_| "Pi session catalog lock poisoned".to_owned())?;
        catalog.agents.retain(|agent| agent.id != agent_id);
        catalog
            .sessions
            .retain(|session| session.agent_id != agent_id);
        persist_catalog(&catalog)
    }
}

fn validate_agent_profiles(agents: &[AgentProfile]) -> Result<(), String> {
    if agents.len() > 32 {
        return Err("at most 32 agent workspaces may be saved".to_owned());
    }
    let mut ids = std::collections::HashSet::with_capacity(agents.len());
    for agent in agents {
        validate_agent_id(&agent.id)?;
        if agent.name.trim().is_empty() {
            return Err(format!("agent `{}` must have a name", agent.id));
        }
        if !ids.insert(agent.id.as_str()) {
            return Err(format!("duplicate agent id `{}`", agent.id));
        }
    }
    Ok(())
}

fn session_catalog_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("dev", "Wabou", "Pi Agent")
        .map(|dirs| dirs.data_local_dir().join("sessions.json"))
}

fn load_session_catalog() -> SessionCatalog {
    session_catalog_path()
        .and_then(|path| std::fs::read(path).ok())
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

fn persist_catalog(catalog: &SessionCatalog) -> Result<(), String> {
    let Some(path) = session_catalog_path() else {
        return Err("could not resolve Pi Agent data directory".to_owned());
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(catalog).map_err(|error| error.to_string())?;
    std::fs::write(path, bytes).map_err(|error| error.to_string())
}

fn remember_session(
    sessions: &Arc<Mutex<SessionCatalog>>,
    agent_id: &str,
    cwd: &str,
    event: &Value,
) {
    if event.get("type").and_then(Value::as_str) != Some("response")
        || event.get("command").and_then(Value::as_str) != Some("get_state")
        || event.get("success").and_then(Value::as_bool) != Some(true)
    {
        return;
    }
    let Some(data) = event.get("data") else {
        return;
    };
    let (Some(session_id), Some(session_file)) = (
        data.get("sessionId").and_then(Value::as_str),
        data.get("sessionFile").and_then(Value::as_str),
    ) else {
        return;
    };
    let updated_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    let Ok(mut catalog) = sessions.lock() else {
        return;
    };
    let entry = PiSession {
        agent_id: agent_id.to_owned(),
        session_id: session_id.to_owned(),
        session_file: session_file.to_owned(),
        name: data
            .get("sessionName")
            .and_then(Value::as_str)
            .map(str::to_owned),
        cwd: cwd.to_owned(),
        updated_at,
    };
    if let Some(existing) = catalog
        .sessions
        .iter_mut()
        .find(|existing| existing.session_id == session_id)
    {
        *existing = entry;
    } else {
        catalog.sessions.push(entry);
    }
    let _ = persist_catalog(&catalog);
}

fn validate_agent_id(agent_id: &str) -> Result<(), String> {
    if agent_id.is_empty()
        || !agent_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err("agentId must contain only letters, numbers, '-' or '_'".to_owned());
    }
    Ok(())
}

fn default_workspace(agent_id: &str) -> Result<String, String> {
    validate_agent_id(agent_id)?;
    let user = directories::UserDirs::new()
        .ok_or_else(|| "could not resolve the user home directory".to_owned())?;
    let root = user
        .document_dir()
        .unwrap_or_else(|| user.home_dir())
        .join("Pi Agent");
    Ok(root.join(agent_id).display().to_string())
}

fn tag_event(agent_id: &str, mut event: Value) -> Value {
    if let Some(object) = event.as_object_mut() {
        object.insert("agentId".to_owned(), Value::String(agent_id.to_owned()));
    }
    event
}

impl Drop for PiProcess {
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

pub fn mount(capability: JsonCapability<'_>, service: PiService) -> rquickjs::Result<()> {
    capability.method(DEFAULT_WORKSPACE, |request: AgentRequest| async move {
        default_workspace(&request.agent_id)
    })?;
    let list_agents = service.clone();
    capability.method(LIST_AGENTS, move |(): ()| {
        let service = list_agents.clone();
        async move { service.agents() }
    })?;
    let save_agents = service.clone();
    capability.method(SAVE_AGENTS, move |agents: Vec<AgentProfile>| {
        let service = save_agents.clone();
        async move { service.save_agents(agents) }
    })?;
    let delete_agent = service.clone();
    capability.method(DELETE_AGENT, move |request: AgentRequest| {
        let service = delete_agent.clone();
        async move { service.delete_agent(&request.agent_id) }
    })?;
    let status = service.clone();
    capability.method(GET_STATUS, move |request: AgentRequest| {
        let service = status.clone();
        async move { service.status(&request.agent_id) }
    })?;
    let start = service.clone();
    capability.method(START, move |request: StartRequest| {
        let service = start.clone();
        async move { service.start(request) }
    })?;
    let list_sessions = service.clone();
    capability.method(LIST_SESSIONS, move |request: AgentRequest| {
        let service = list_sessions.clone();
        async move { service.sessions(&request.agent_id) }
    })?;
    let get_messages = service.clone();
    capability.method(GET_MESSAGES, move |request: AgentRequest| {
        let service = get_messages.clone();
        async move { service.send(&request.agent_id, json!({"type":"get_messages"})) }
    })?;
    let prompt = service.clone();
    capability.method(PROMPT, move |request: PromptRequest| {
        let service = prompt.clone();
        async move {
            let message = request.message.trim();
            if message.is_empty() {
                return Err("prompt cannot be empty".to_owned());
            }
            service.send(
                &request.agent_id,
                json!({"type":"prompt","message":message}),
            )
        }
    })?;
    let follow_up = service.clone();
    capability.method(FOLLOW_UP, move |request: PromptRequest| {
        let service = follow_up.clone();
        async move {
            let message = request.message.trim();
            if message.is_empty() {
                return Err("follow-up cannot be empty".to_owned());
            }
            service.send(
                &request.agent_id,
                json!({"type":"follow_up","message":message}),
            )
        }
    })?;
    let abort = service.clone();
    capability.method(ABORT, move |request: AgentRequest| {
        let service = abort.clone();
        async move { service.send(&request.agent_id, json!({"type":"abort"})) }
    })?;
    let stop = service.clone();
    capability.method(STOP, move |request: AgentRequest| {
        let service = stop.clone();
        async move { service.stop(&request.agent_id) }
    })?;
    let new_session = service.clone();
    capability.method(NEW_SESSION, move |request: AgentRequest| {
        let service = new_session.clone();
        async move {
            service.send(&request.agent_id, json!({"type":"new_session"}))?;
            service.send(
                &request.agent_id,
                json!({"id":"wabou-new-session-state","type":"get_state"}),
            )
        }
    })?;
    let cycle_model = service.clone();
    capability.method(CYCLE_MODEL, move |request: AgentRequest| {
        let service = cycle_model.clone();
        async move { service.send(&request.agent_id, json!({"type":"cycle_model"})) }
    })?;
    let cycle_thinking = service.clone();
    capability.method(CYCLE_THINKING, move |request: AgentRequest| {
        let service = cycle_thinking.clone();
        async move { service.send(&request.agent_id, json!({"type":"cycle_thinking_level"})) }
    })?;
    capability.method(SET_MODEL, move |request: SetModelRequest| {
        let service = service.clone();
        async move {
            service.send(
                &request.agent_id,
                json!({"type":"set_model","provider":request.provider,"modelId":request.model_id}),
            )
        }
    })
}

fn drain_event_batch(first: Value, receiver: &flume::Receiver<Value>) -> Vec<Value> {
    let mut events = Vec::with_capacity(32);
    events.push(first);
    while events.len() < 64 {
        match receiver.try_recv() {
            Ok(event) => events.push(event),
            Err(_) => break,
        }
    }
    events
}

pub fn stream_events(context: HostMessageContext, service: PiService) {
    let producer = context.clone();
    context.spawn(async move {
        loop {
            tokio::select! {
                () = producer.cancelled() => break,
                event = service.events_rx.recv_async() => match event {
                    Ok(event) => {
                        // Pi can emit a JSONL record for every streamed token. Give the reader a
                        // short coalescing window, then deliver one JS update for the whole batch.
                        tokio::time::sleep(std::time::Duration::from_millis(8)).await;
                        let events = drain_event_batch(event, &service.events_rx);
                        let payload = match serde_json::to_string(&events) {
                            Ok(payload) => payload,
                            Err(error) => {
                                tracing::warn!(?error, "could not encode Pi event batch");
                                continue;
                            }
                        };
                        if producer.messages().send_async(HostMessage::str(EVENT_TOPIC, payload)).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_is_stopped_before_start() {
        let status = PiService::new().status("default").expect("status");
        assert!(!status.running);
        assert_eq!(status.runtime, "bun");
    }

    #[test]
    fn event_batch_is_bounded_and_leaves_backpressure_in_the_channel() {
        let (sender, receiver) = flume::bounded(128);
        for index in 1..=70 {
            sender.send(json!({"index": index})).expect("queued event");
        }
        let first = receiver.recv().expect("first event");
        let batch = drain_event_batch(first, &receiver);
        assert_eq!(batch.len(), 64);
        assert_eq!(receiver.len(), 6);
    }

    #[test]
    fn tags_events_with_their_agent_identity() {
        assert_eq!(
            tag_event("agent-2", json!({"type":"agent_start"}))["agentId"],
            "agent-2"
        );
    }

    #[test]
    fn older_session_catalogs_default_to_no_saved_agents() {
        let catalog: SessionCatalog =
            serde_json::from_str(r#"{"sessions":[]}"#).expect("legacy catalog");
        assert!(catalog.agents.is_empty());
    }

    #[test]
    fn saved_agent_profiles_require_unique_stable_ids() {
        let profile = AgentProfile {
            id: "agent-1".to_owned(),
            name: "Agent 1".to_owned(),
            cwd: String::new(),
            proxy: String::new(),
            no_proxy: String::new(),
            provider: String::new(),
            model: String::new(),
        };
        assert!(validate_agent_profiles(std::slice::from_ref(&profile)).is_ok());
        assert_eq!(
            validate_agent_profiles(&[profile.clone(), profile]).unwrap_err(),
            "duplicate agent id `agent-1`"
        );
    }

    #[test]
    fn default_workspace_is_scoped_to_a_valid_agent_directory() {
        let workspace = PathBuf::from(default_workspace("agent-2").expect("workspace"));
        assert_eq!(
            workspace.file_name().and_then(|name| name.to_str()),
            Some("agent-2")
        );
        assert_eq!(
            workspace
                .parent()
                .and_then(|path| path.file_name())
                .and_then(|name| name.to_str()),
            Some("Pi Agent")
        );
        assert!(default_workspace("../escape").is_err());
    }
}
