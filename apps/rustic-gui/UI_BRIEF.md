# Rustic GUI product brief

- User and job: a desktop user who does not want to learn the rustic CLI needs to create named backups, choose their folders and storage locations, find a snapshot, and inspect its files.
- Primary action: before setup, create a backup profile; inside a profile, back up its selected folders now.
- Information order: backup profiles, selected profile status and sources, snapshots, selected snapshot contents.
- Density and viewport: balanced file-workspace density; 1240×780 normal viewport and 900×620 minimum viewport.
- Visual thesis: a calm native file workspace with a pale canvas, crisp white working surfaces, compact rows, and blue reserved for selection and progress. Avoid dashboard card grids.
- Shell hierarchy: a persistent 224px application sidebar owns the named backup profiles and the New backup action. Route content owns its page header and is the only top-level scrolling region.
- Narrow behavior: keep the application sidebar fixed at the minimum viewport; truncate profile names instead of compressing their status indicators. The snapshot/file split remains a secondary workbench inside an unlocked profile.
- Signature interaction: selecting a snapshot immediately opens a breadcrumb-driven, lazy file browser beside the timeline.
- Reference delta: use Wabou's retained native controls, PageViewport, DirectoryPicker, Table, and explicit scroll boundaries; do not inherit browser DOM behavior.
- Shared contracts: ComponentsProvider, ColorThemeProvider, PageHeader, Button, DirectoryPicker, Input, Table, ContentState, ScrollArea, and ProjectionBoundary.
- Required states: no profiles, locked profile, opening, empty snapshots, backup running, failure with retry, long paths, narrow width, and large directories.
- Proof: Rust service tests cover create → backup → list snapshots → list files. Component tests cover setup, source editing, snapshot selection, and empty/error states. Native directory picking remains a focused behavior scenario.

The first vertical slice stores profile metadata and source/repository relationships in Wabou's SQLite KV. Repository credentials deliberately remain process-local, so persisted profiles must be unlocked after restart. S3/OpenDAL and a native secret bridge are follow-up work.
