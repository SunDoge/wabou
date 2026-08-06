# Wabou agent guidance

For rendering, layout, clipping, custom-widget, resize/maximize, HMR, DevTools, hit-testing, platform-difference, or performance debugging, read and follow `.agents/skills/wabou-debug/SKILL.md` before changing code.

Require evidence from the failing layer. A successful build is not proof of a visual fix, and a Linux 1× PNG is not proof of macOS HiDPI behavior. Restart the native process after Rust changes; Vite HMR cannot reload Rust code.
