use wabou_runtime::{
    HostBuilder, HostMessage, HostMessageContext, HostMessageError, HostMessagePayload,
    HostMessageRouter,
};

#[test]
fn application_can_register_a_typed_host_message_producer() {
    let _builder = HostBuilder::new().host_message_producer(|context: HostMessageContext| {
        let _window_key = context.window_key();
        let _messages = context.messages().clone();
        let task_context = context.clone();
        let _task = context.spawn(async move {
            task_context.cancelled().await;
        });
    });

    let message = HostMessage::str("example:status", "ready");
    assert_eq!(message.payload, HostMessagePayload::Str("ready".into()));
    let _error = HostMessageError::Full;

    let router = HostMessageRouter::new();
    let _builder = HostBuilder::new().host_message_router(router.clone());
    let unavailable = router.send_to(
        wabou_runtime::initial_window_resource_key(0),
        HostMessage::null("example:not-ready"),
    );
    assert_eq!(unavailable, Err(HostMessageError::WindowUnavailable));
}
