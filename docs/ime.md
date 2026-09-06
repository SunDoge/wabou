# IME architecture

Wabou treats IME support as a text-client contract, not as a renderer feature.
QuickJS and Solid own application state; a focused native editor owns transient
composition state; a platform adapter connects that editor to the operating
system. Vello, GPUI, or another renderer only paints the resulting geometry.

The legacy Winit/Vello backend is the first portability experiment. Its focused
widget publishes one immutable IME snapshot containing:

- committed surrounding text and UTF-8 cursor/anchor offsets for platform IMEs;
- JavaScript-compatible UTF-16 selection and marked ranges;
- candidate-window geometry in widget-local coordinates.

The same client contract supports GPUI-style read queries for text in a UTF-16
range, visual bounds for an arbitrary range, and point-to-character hit
testing. Purpose and completion/spellcheck/multiline hints are mapped to
Winit without exposing Winit types to widgets.

Parley's editor computes the candidate exclusion area from the active preedit
run or focused-line selection, including nearby visual context to avoid popup
jitter. The runtime transforms that geometry into window coordinates and Winit forwards
the cursor area and surrounding text to its macOS, Windows, Wayland, and X11
adapters. Preedit text is deliberately excluded from surrounding text. Long
documents are reduced to a UTF-8-safe excerpt below Winit's 4,000-byte limit.

This proves the shared contract and composition data path, but it is not yet
full GPUI parity. Arbitrary platform-driven replacement, rich-text attributes,
and platform-specific recovery behavior remain follow-up work.
If direct platform integration becomes necessary, it should live in separate
platform crates behind this same contract rather than leaking AppKit, TSF/IMM,
Wayland text-input, or XIM details into widgets or renderers.
