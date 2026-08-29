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

## Automated application verification

Run the safe, fully automated Pi Agent gate with:

```bash
bun run verify:agent
```

It runs the complete deterministic behavior suite, then uses the real application
host with the fixture to write a conversation screenshot and final DevTools tree
under `target/wabou-test/pi-agent/`. More focused entry points are available:

```bash
bun run test:app:pi-agent
bun run test:app:pi-agent:capture
bun run test:app:pi-agent:native
```

The native smoke test requires a usable desktop session. Real provider testing is
deliberately excluded from CI and the default gate because it uses credentials,
network access, and provider credits. Run it only with explicit consent:

```bash
WABOU_RUN_REAL_PI=1 bun run test:app:pi-agent:real
```

Proxy variables are inherited by Bun, Pi, and provider requests. Use a dedicated,
low-limit test credential; behavior traces can contain authored prompt text.
