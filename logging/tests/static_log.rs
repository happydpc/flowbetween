extern crate flo_logging;
extern crate desync;

use flo_logging::*;
use desync::*;

use std::sync::*;

#[test]
fn publish_log_messages_to_static_log() {
    let tgt         = LogPublisher::new("Test");
    let messages    = Arc::new(Desync::new(vec![]));

    pipe_in(Arc::clone(&messages), subscribe_to_logs(), |messages, new_message| messages.push(new_message.unwrap()));

    tgt.log("Hello, world");
    tgt.log("... goodbye, world :-(");

    let messages    = messages.sync(|messages| messages.clone());

    assert!(messages.len() != 0);
    assert!(&messages[0].message() == "Hello, world");
    assert!(messages.len() != 1);
    assert!(&messages[1].message() == "... goodbye, world :-(");
    assert!(messages.len() == 2);
}
