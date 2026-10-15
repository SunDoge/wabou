---
"@wabou/core": minor
---

Make native window creation asynchronous and expose window identities as typed
two-u32 generational keys so closed handles cannot target reused window slots.
