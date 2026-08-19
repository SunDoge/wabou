# Host messages

Wabou applications use host messages for long-running Rust → JavaScript event
streams such as WebSockets, file watchers, database subscriptions, and device
events. Request/response operations should continue to use typed capabilities.

Register a producer on `HostBuilder`:

```rust
HostBuilder::new()
    .host_message_producer(|context| {
        let task_context = context.clone();
        context.spawn(async move {
            loop {
                tokio::select! {
                    _ = task_context.cancelled() => break,
                    value = next_value() => {
                        let Ok(value) = value else { break };
                        if task_context.messages().emit_str("demo:value", value).is_err() {
                            break;
                        }
                    }
                }
            }
        });
    })
    .run()?;
```

Subscribe in JavaScript:

```ts
import { hostMessages } from "@wabou/ui";

const unsubscribe = hostMessages.subscribe("demo:value", (value) => {
  console.log(value);
});
```

Each native window receives a distinct `HostMessageContext`. The context owns
the window ID, a cloneable `HostMessageHandle`, access to the window host's
Tokio runtime, and a cancellation signal triggered when the window is dropped.

The queue is bounded and producers never block by default. `Full` means the UI
cannot currently keep up. High-frequency telemetry should coalesce or discard
intermediate samples instead of retrying in a tight loop. `Disconnected` means
the window has gone away. String and binary payloads also have explicit size
limits; large resources belong in the resource/handle APIs rather than this
event channel.
