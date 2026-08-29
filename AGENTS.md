# Wabou agent guidance

For rendering, layout, clipping, custom-widget, resize/maximize, HMR, DevTools, hit-testing, platform-difference, or performance debugging, read and follow `.agents/skills/wabou-debug/SKILL.md` before changing code.

For new UI, visual redesigns, component-library work, or UI quality reviews, also read and follow `.agents/skills/frontend-design-review/SKILL.md`. Apply its Wabou-specific review profile instead of assuming a browser, Figma, or Storybook workflow.

Require evidence from the failing layer. A successful build is not proof of a visual fix, and a Linux 1× PNG is not proof of macOS HiDPI behavior. Restart the native process after Rust changes; Vite HMR cannot reload Rust code.

Do not push commits or tags. When a push is required, stop and give the user the exact command to run.
