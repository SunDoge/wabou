# Wabou Vault (read-only demo)

This application demonstrates that a Wabou desktop UI can use Bitwarden's Rust
SDK directly. It is a prototype, not a replacement for an official Bitwarden
client.

## Scope

- Email and master-password login for US, EU, and self-hosted servers.
- Read-only vault sync, search, item details, and username/password copy.
- Explicit lock, five-minute native idle lock, and conditional clipboard clearing
  after 30 seconds.
- No writes, tray, autofill, attachments, organizations UI, biometric unlock, or
  two-step login yet.

The SDK dependencies are pinned to commit
`fbd21679c1c3690cba424ad3dbb0ba662eb9863e`; an upstream `main` update cannot
silently change this build.

## Security boundary

The master password is owned by the native `secure-input` widget in zeroizing
Rust memory. It never becomes a DOM value, a QuickJS string, a command argument,
or an environment variable. Authentication, synchronization, decryption, and
clipboard access all happen in Rust. JavaScript receives only the decrypted
metadata needed to render the current read-only view. Passwords are copied by
item ID through the native bridge and are never returned to JavaScript.

DevTools are disabled because UI snapshots could otherwise contain decrypted
item names and usernames. Clipboard clearing is conditional: after 30 seconds
the app clears only the value it wrote, so it does not erase a newer clipboard
entry.

This still is not production-hardened. In particular, decrypted titles and
usernames live in the QuickJS heap while unlocked, memory locking is not
implemented, two-step login is absent, and the upstream Password Manager Rust
SDK describes itself as internal and unstable.

## License

The open-source Bitwarden SDK crates used here are GPL-3.0-only. Consequently
this demo package declares `GPL-3.0-only`; distribution must comply with the SDK
and all other dependency licenses. A commercial Bitwarden SDK agreement is a
separate option.

## Run and verify

Use a disposable test vault without two-step login:

```sh
mise exec -- bun install
mise exec -- bun run --cwd apps/bitwarden build
cargo test -p bitwarden-demo
wabou run --app-dir apps/bitwarden
```

Do not use a real vault to evaluate this prototype.

### Test the unlocked UI without credentials

`apps/bitwarden-preview` renders the same `VaultScreen` with deterministic fake
items. It is a separate build entry and cannot bypass login in the real app.

```sh
.agents/skills/wabou-debug/scripts/capture-png.sh \
  bitwarden-preview /tmp/bitwarden-vault.png 1100 720

# Select the first item and capture its detail view.
.agents/skills/wabou-debug/scripts/capture-png.sh \
  bitwarden-preview /tmp/bitwarden-details.png 1100 720 250 215
```
