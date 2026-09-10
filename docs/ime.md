# IME architecture

Wabou treats IME support as a text-client contract, not as a renderer feature.
QuickJS and Solid own application state. `TextInput` and `TextArea` keep their
small native documents, while the general `Editor` owns its document and
transactions in DOM-free CodeMirror and mirrors composition into a native
viewport. A platform adapter connects either client to the operating system;
Vello Hybrid only paints the resulting geometry.

The focused widget publishes one immutable IME snapshot containing:

- committed surrounding text and UTF-8 cursor/anchor offsets for platform IMEs;
- JavaScript-compatible UTF-16 selection and marked ranges;
- candidate-window geometry in widget-local coordinates.

The same client contract supports platform read queries for text in a UTF-16
range, visual bounds for an arbitrary range, and point-to-character hit
testing. Purpose and completion/spellcheck/multiline hints are mapped to
Winit without exposing Winit types to widgets.

Each focused client also publishes a stable host-node identity. Moving focus
directly between two editors cancels the outgoing widget's uncommitted preedit
and restarts Winit's native IME session, matching Xilem/Masonry's protection
against composition text travelling between controls.

Parley's editor computes the candidate exclusion area from the active preedit
run or focused-line selection, including nearby visual context to avoid popup
jitter. The runtime transforms that geometry into window coordinates and Winit forwards
the cursor area and surrounding text to its macOS, Windows, Wayland, and X11
adapters. Preedit text is deliberately excluded from surrounding text. Long
documents are reduced to a UTF-8-safe excerpt below Winit's 4,000-byte limit.

This proves the shared contract and composition data path. Arbitrary
platform-driven replacement, rich-text attributes,
and platform-specific recovery behavior remain follow-up work.
If direct platform integration becomes necessary, it should live in separate
platform crates behind this same contract rather than leaking AppKit, TSF/IMM,
Wayland text-input, or XIM details into widgets or renderers.

## Complex-script regression contract

IME correctness and glyph shaping are related but tested separately. The
renderer fixture `component/ComplexShaping` covers a Latin/Tibetan line and a
Latin/Arabic editable line. Its backend-labelled pixel capture checks font
fallback, combining glyph placement, bidirectional shaping, clipping, and
baseline stability. The matching native behavior scenario selects and replaces
the complete Arabic mixed-direction value, checking the editor's UTF-16/UTF-8
selection bridge. Unit tests continue to cover preedit, commit, surrounding
text, range geometry, and candidate-window placement.

Passing only one layer is insufficient: preserved string contents do not prove
correct shaping, and a visually plausible capture does not prove that an IME
can edit or select the text.
