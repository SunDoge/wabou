# Timestow product brief

- User and job: a desktop user who does not want to learn the rustic CLI needs to open a local or S3-backed repository, back up several source folders, find an older file, and restore only the selected file or directory.
- Primary action: before setup, create a local backup or import a rustic configuration; inside a profile, browse snapshots and restore selected content, or back up its selected folders now.
- Information order: backup profiles, selected profile status, snapshot timeline, selected snapshot contents or changes. Source folders are configuration disclosed on demand, not a permanent navigation block.
- Density and viewport: balanced file-workspace density; 1240×780 normal viewport and 900×620 minimum viewport.
- Visual thesis: a calm native backup workbench with a pale shell, one continuous working surface, compact rows, and blue reserved for selection, progress, and the primary backup action. Avoid dashboard card grids and cards nested inside navigation rails.
- Shell hierarchy: a persistent 224px application sidebar owns the named backup profiles and the New backup action. Route content owns its page header and is the only top-level scrolling region.
- Narrow behavior: keep the application sidebar fixed at the minimum viewport; truncate profile names instead of compressing their status indicators. The snapshot/file split remains a secondary workbench inside an unlocked profile.
- Signature interaction: selecting a point in the snapshot timeline immediately opens its breadcrumb-driven, lazy file browser beside the rail; Changes compares it with its recorded parent without leaving the workspace.
- Reference delta: use Wabou's retained native controls, PageViewport, DirectoryPicker, Table, and explicit scroll boundaries; do not inherit browser DOM behavior.
- Shared contracts: ComponentsProvider, ColorThemeProvider, PageHeader, Button, DirectoryPicker, Input, Table, ContentState, ScrollArea, and ProjectionBoundary.
- Required states: no profiles, locked profile, opening, imported read-only S3 profile, one snapshot with no comparison target, unchanged snapshots, protected and deletable snapshots, deletion failure, manual or scheduled backup running, restore overwrite review, schedule success/failure, long paths, narrow width, and large directories or diffs.
- Proof: Rust service tests cover create → multi-source backup → list snapshots → browse files → partial restore, including original-location path resolution and overwrite confirmation. An opt-in smoke test opens a real rustic remote configuration. Component tests cover setup, source editing, snapshot selection, destination choice, and empty/error states; layout fixtures cover the remote setup and restore dialog at their minimum sizes.

Profile metadata and source/repository relationships live in Wabou's SQLite KV. Repository credentials never cross into JavaScript: local passwords remain process-local, while imported `rustic.toml` credentials are parsed by Rust. Imported configurations are initially read-only in Timestow, but can browse S3/OpenDAL snapshots and restore selected content.

Automatic schedules are profile-scoped and durable. They run while Timestow is open and the corresponding repository is unlocked; manual and automatic runs share one single-flight coordinator. OS-level background scheduling intentionally waits for a native secret-store bridge, because a closed process cannot safely recover the repository password from the credential-free profile database.
