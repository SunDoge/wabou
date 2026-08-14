use wabou_quick::{
    HostBuilder, HostMessage, HostMessageContext, HostMessageError, HostMessagePayload,
};

#[test]
fn application_can_register_a_typed_host_message_producer() {
    let _builder = HostBuilder::new().host_message_producer(|context: HostMessageContext| {
        let _window_id = context.window_id();
        let _messages = context.messages().clone();
        let task_context = context.clone();
        let _task = context.spawn(async move {
            task_context.cancelled().await;
        });
    });

    let message = HostMessage::str("example:status", "ready");
    assert_eq!(message.payload, HostMessagePayload::Str("ready".into()));
    let _error = HostMessageError::Full;
}
