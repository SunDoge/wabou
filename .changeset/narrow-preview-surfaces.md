---
"@wabou/core": minor
"@wabou/ui": minor
---

Remove the deprecated `useFps` and `AnimationPlaybackControls` aliases before
the developer preview, keeping effect-owning primitives and animation handles
under their single canonical names. Keep the numeric native-effect dispatcher
inside the framework; application integrations use generated JSON capabilities
and host messages instead of depending on Wabou's private effect ABI.
