# @wabou/animation

Motion's value animation engine adapted to Wabou's QuickJS runtime. It supports
numbers, colors, springs and playback controls without pulling in React or DOM
animation code.

```tsx
import { animate, animateKeyframes } from "@wabou/animation";

const controls = animate(0, 100, {
  duration: 0.3,
  onUpdate: setProgress,
});

const pulse = animateKeyframes([0.5, 1, 0.5], {
  duration: 0.8,
  times: [0, 0.5, 1],
  onUpdate: setOpacity,
});
```

As with Motion's public API, `duration`, `delay`, and `repeatDelay` are measured
in seconds. The wrapper converts them to the millisecond units used internally
by `motion-dom`.

The public options and playback controls are owned by Wabou; Motion types are
not exposed. This keeps the backend replaceable. DOM targets, selectors,
object timelines, scroll observers and gesture helpers are intentionally out
of scope. Animate values into Solid signals, then bind those values to Wabou
classes or inline styles.

For component-owned repeating motion, use the Solid-aware controllers. They
create the signal, expose playback controls, and stop automatically when the
current Solid owner is disposed:

```tsx
const rotation = createRotation({ duration: 0.9 });
<View transform={rotation.transform()} />;

const pulse = createPulse({ from: 0.45, to: 0.85, duration: 1.8 });
<View style={{ opacity: pulse.value() }} />;
```

`createLoop` is the lower-level repeating scalar controller. Rotation matrices
contain no manual pivot translation: Wabou's native scene builder applies
runtime transforms around the node's border-box center.
