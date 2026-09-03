# Wabou product UI brief

Use this brief before creating a screen or substantially changing its hierarchy,
density, navigation, or visual language. It exists to prevent a series of locally
reasonable component edits from producing an incoherent application.

Keep the brief short enough to remain visible while implementing:

```text
User + job:
Primary action:
Information order:
Density + minimum viewport:
Visual thesis:
Signature interaction or detail:
Reference delta:
Shared contracts to reuse or extend:
States and pressure cases:
Proof:
```

## How to fill it

- **User + job** names the person and the concrete outcome, not the technology.
- **Primary action** is the one action that should win the first visual scan.
- **Information order** lists at most four layers from immediate to disclosed.
- **Density + minimum viewport** states whether the surface is compact, balanced,
  or spacious and names the smallest supported window.
- **Visual thesis** is one sentence describing hierarchy, material, and tone. It
  must be specific enough to reject a plausible but wrong design.
- **Signature interaction or detail** names one memorable, product-specific move.
  Do not distribute novelty across every component.
- **Reference delta** records what to borrow from the supplied reference and what
  must differ because Wabou is a native retained UI rather than a browser. For the
  Pi Agent app, use Waku and Picot as interaction and density calibration, not as
  branding to clone.
- **Shared contracts to reuse or extend** identifies the existing Wabou components,
  tokens, and layout regions that own the result. A page-local class recipe is not
  a shared contract.
- **States and pressure cases** names the non-ideal states that could disprove the
  design: empty, loading, streaming, failure, long translated text, narrow resize,
  many sessions, or active native widgets.
- **Proof** selects the cheapest authoritative component, layout, behavior, and
  paint evidence. A screenshot alone cannot prove semantics or resize behavior.

## Product UI defaults

- Prefer calm, information-dense workspaces over card-per-section dashboards.
- Keep persistent navigation and primary controls visually quieter than the work.
- Reserve the accent for selection, progress, focus, and the primary action.
- Preserve control dimensions across labels, loading, counts, and translated copy.
- Use elevation only to explain overlap or a raised interaction surface.
- Treat typography, spacing rhythm, icon alignment, borders, and shadows as shared
  contracts; do not compensate for drift with repeated local offsets.
- Motion communicates continuity. It must never delay opening a menu or disguise a
  remount, layout jump, or stale state.

## Exit gate

Before calling the screen complete, compare the rendered result with its visual
thesis and reference delta. If the hierarchy can only be explained by reading the
JSX, the design has not made the information order visible. If a shared component
needed screen-specific correction more than once, move the correction into its
contract and add component plus layout evidence.

This brief adapts the product-UI direction and stable-control guidance from
[mblode/agent-skills](https://github.com/mblode/agent-skills/tree/main/skills/ui-design)
to Wabou's native rendering and evidence model.
