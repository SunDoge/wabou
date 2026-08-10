# Wabou interactions roadmap

Wabou uses WAI-ARIA APG as the semantic baseline, Zag as the primary state-machine
reference, React Stately for collection and selection behavior, and Kobalte/Corvu
for Solid composition patterns. Browser DOM effects are replaced with explicit
Wabou host capabilities.

## P0 — shared foundations

- [x] Elm-style `update(state, event) -> state + commands`
- [x] Solid machine adapter
- [x] controlled and uncontrolled state
- [x] ordered collection with disabled-item navigation
- [x] locale-aware typeahead
- [x] single and multiple selection
- [x] roving focus with orientation and looping
- [x] disclosure behavior
- [ ] stable element IDs and semantic relationships
- [ ] dismissable layer backed by Wabou overlay hit testing
- [ ] focus scope and focus restoration

## P1 — component behaviors

- [x] Select and Listbox (native long-list scrolling; virtualization remains optional)
- [ ] Menu and Context Menu
- [ ] Combobox
- [ ] Tooltip
- [ ] Dialog/AlertDialog behavior adapter
- [ ] ToggleGroup

## P2 — advanced collections

- [ ] Tree and TreeView
- [ ] Grid and DataGrid navigation
- [ ] Date and range selection
- [ ] Drag-and-drop collection operations
- [ ] Virtualized collection hooks

Every behavior should have pure transition tests, Solid adapter tests, host
integration tests, and an interaction matrix recording APG, Zag, and deliberate
Wabou-native differences.
