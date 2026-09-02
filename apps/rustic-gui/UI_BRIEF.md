# Rustic GUI product brief

- User and job: a desktop user who does not want to learn the rustic CLI needs to choose local folders, create encrypted backups, find a snapshot, and inspect its files.
- Primary action: before setup, create or open a repository; after setup, back up the selected folders now.
- Information order: repository status, backup sources, snapshots, selected snapshot contents.
- Density and viewport: balanced file-workspace density; 1240×780 normal viewport and 900×620 minimum viewport.
- Visual thesis: a calm native file workspace with a pale canvas, crisp white working surfaces, compact rows, and blue reserved for selection and progress. Avoid dashboard card grids.
- Signature interaction: selecting a snapshot immediately opens a breadcrumb-driven, lazy file browser beside the timeline.
- Reference delta: use Wabou's retained native controls, PageViewport, DirectoryPicker, Table, and explicit scroll boundaries; do not inherit browser DOM behavior.
- Shared contracts: ComponentsProvider, ColorThemeProvider, PageHeader, Button, DirectoryPicker, Input, Table, ContentState, ScrollArea, and ProjectionBoundary.
- Required states: no repository, opening, empty snapshots, backup running, failure with retry, long paths, narrow width, and large directories.
- Proof: Rust service tests cover create → backup → list snapshots → list files. Component tests cover setup, source editing, snapshot selection, and empty/error states. Native directory picking remains a focused behavior scenario.

The first vertical slice supports local repositories and deliberately keeps repository credentials process-local. S3/OpenDAL and a native secret bridge are follow-up work.
