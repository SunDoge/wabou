# Desktop UI reference applications

Wabou uses open-source Electron applications as reproducible product-design
references. The goal is not source compatibility or pixel-for-pixel copying.
Each reference isolates an information architecture that Wabou applications
should be able to reproduce with native rendering, shared components, and
layout contracts.

## Reference set

| Application | Why it is useful | Wabou target |
| --- | --- | --- |
| [GitHub Desktop](https://github.com/desktop/desktop) | Dense repository lists, split content, selection, menus, dialogs, and high-frequency state changes. | A future source-control workbench fixture using the shared sidebar, table, tree, diff, and inspector components. |
| [Fluent Reader](https://github.com/yang991178/fluent-reader) | Sidebar/feed/reader hierarchy, long virtualized lists, article typography, and narrow-window behavior. | A future reading-workspace fixture using virtual lists, rich text, search, and adaptive split panes. |

These projects are references, not dependencies. Do not copy their trademarks
or bundled assets into Wabou fixtures.

## Benchmark contract

Every reference reproduction must:

1. render a named fixture at a fixed desktop viewport;
2. use production components rather than a screenshot-only mock;
3. assert overflow, text collision, and visual-quality diagnostics;
4. cover the application's minimum supported viewport separately;
5. record deliberate differences from the reference;
6. capture a native PNG only after the component and layout checks pass.

Full-image pixel equality is not a useful gate across different text engines
and icon sets. Prefer structural assertions for regions and alignment, theme
token assertions for color and contrast, and focused pixel comparisons for
shadows, clipping, compositing, and other renderer behavior.
