# Pi Agent · Wabou

A native desktop client for the Pi coding agent. Wabou's QuickJS runtime owns the
Solid UI; each configured agent owns an independent Bun-based Pi process, session,
workspace, provider and proxy environment. They communicate through Pi's
LF-delimited JSONL RPC protocol.

## Run from the workspace

Install Bun, then run:

```bash
bun install
bun run wabou dev apps/pi-agent
```

The app uses `bun x` to resolve a pinned Pi release. Set `WABOU_PI_BIN` to use an
existing Pi executable instead.

Global defaults live on the Settings route and can be overridden per agent before it
starts. Provider and model can be supplied at process startup or changed through
Pi's `set_model` RPC command.

Proxy settings are available before Bun or Pi starts. An explicit proxy is exported as
`HTTP_PROXY`, `HTTPS_PROXY`, and `ALL_PROXY` (plus lowercase aliases) to both Bun's
`bun x` npm package resolution and Pi's provider requests. Leaving it empty inherits
the desktop process environment; `NO_PROXY` is separately configurable.

The first preview expects Bun to be installed. Windows distribution can use the same
process boundary with a bundled Bun/Pi sidecar, so the UI and RPC protocol do
not need to change. A future experiment may move compatible agent code into Wabou's
QuickJS runtime; that is deliberately not required by this application today.

The interface defaults to English for public demos and uses compiled Paraglide
messages. Chinese and future locales can be added incrementally without shipping a
browser-oriented i18n runtime.

## Deterministic GUI test

The native behavior suite uses a small Rust Pi JSONL fixture, so it exercises the
real Solid → capability → Rust service → child process → batched host-message path
without network access, provider credentials, or model output variance:

```bash
cargo build -p pi-agent-wabou --example pi-agent-fixture
cargo run -p wabou-cli -- test apps/pi-agent \
  --env "WABOU_PI_BIN=$PWD/target/debug/examples/pi-agent-fixture"
```

`bun run verify:behavior` builds and injects this fixture automatically in CI.
