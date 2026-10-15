# Launcher UI brief

User + job: A keyboard-first desktop user wants to find and open an installed application without leaving the current task.
Primary action: Type a query, confirm the highlighted result with Enter.
Information order: Query, ranked applications, selected application metadata, keyboard guidance.
Density + minimum viewport: Compact fixed utility window at 720×500 logical pixels.
Visual thesis: A quiet graphite command surface with one luminous search row, restrained blue selection, and no dashboard chrome.
Signature interaction or detail: The first result is always ready to launch; pointer and arrow-key selection share one highlight.
Reference delta: Borrow Spotlight's centered transient window and immediate keyboard flow, while exposing application source/category instead of imitating macOS branding or materials.
Shared contracts to reuse or extend: `Input`, `CommandList`, `ScrollArea`, `Icon`, semantic theme tokens, and native window lifecycle.
States and pressure cases: Initial loading, empty system index, no matches, launch failure, long localized names, keyboard-only use, pointer selection.
Proof: Component tests for ranking and keyboard semantics; native layout fixture at 720×500; host-backed capture for transparency and paint; manual platform check for the global shortcut.
