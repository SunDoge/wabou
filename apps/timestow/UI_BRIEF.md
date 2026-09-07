# Timestow product brief

- User and job: a desktop user who does not want to learn the rustic CLI needs to create named backups, choose their folders and storage locations, find a snapshot, and inspect its files.
- Primary action: before setup, create a backup profile; inside a profile, back up its selected folders now.
- Information order: backup profiles, selected profile status, snapshot timeline, selected snapshot contents or changes. Source folders are configuration disclosed on demand, not a permanent navigation block.
- Density and viewport: balanced file-workspace density; 1240×780 normal viewport and 900×620 minimum viewport.
- Visual thesis: a calm native backup workbench with a pale shell, one continuous working surface, compact rows, and blue reserved for selection, progress, and the primary backup action. Avoid dashboard card grids and cards nested inside navigation rails.
- Shell hierarchy: a persistent 224px application sidebar owns the named backup profiles and the New backup action. Route content owns its page header and is the only top-level scrolling region.
- Narrow behavior: keep the application sidebar fixed at the minimum viewport; truncate profile names instead of compressing their status indicators. The snapshot/file split remains a secondary workbench inside an unlocked profile.
- Signature interaction: selecting a point in the snapshot timeline immediately opens its breadcrumb-driven, lazy file browser beside the rail; Changes compares it with its recorded parent without leaving the workspace.
- Reference delta: use Wabou's retained native controls, PageViewport, DirectoryPicker, Table, and explicit scroll boundaries; do not inherit browser DOM behavior.
- Shared contracts: ComponentsProvider, ColorThemeProvider, PageHeader, Button, DirectoryPicker, Input, Table, ContentState, ScrollArea, and ProjectionBoundary.
- Required states: no profiles, locked profile, opening, one snapshot with no comparison target, unchanged snapshots, manual or scheduled backup running, schedule success/failure, long paths, narrow width, and large directories or diffs.
- Proof: Rust service tests cover create → backup → list snapshots → list files. Component tests cover setup, source editing, snapshot selection, and empty/error states. Native directory picking remains a focused behavior scenario.

The first vertical slice stores profile metadata and source/repository relationships in Wabou's SQLite KV. Repository credentials deliberately remain process-local, so persisted profiles must be unlocked after restart. S3/OpenDAL and a native secret bridge are follow-up work.

Automatic schedules are profile-scoped and durable. They run while Timestow is open and the corresponding repository is unlocked; manual and automatic runs share one single-flight coordinator. OS-level background scheduling intentionally waits for a native secret-store bridge, because a closed process cannot safely recover the repository password from the credential-free profile database.
