#![cfg(windows)]

#[allow(dead_code)]
mod common;

use std::any::Any;

#[test]
fn panic_payload_message_preserves_string_payload() {
    let payload: Box<dyn Any + Send> = Box::new(String::from("string payload"));
    assert_eq!(
        common::panic_payload_message(payload.as_ref()),
        "string payload"
    );
}

#[test]
fn panic_payload_message_preserves_static_str_payload() {
    let payload: Box<dyn Any + Send> = Box::new("static payload");
    assert_eq!(
        common::panic_payload_message(payload.as_ref()),
        "static payload"
    );
}

#[test]
fn panic_payload_message_labels_opaque_payload() {
    let payload: Box<dyn Any + Send> = Box::new(42_u32);
    assert_eq!(
        common::panic_payload_message(payload.as_ref()),
        "non-string panic payload from runtime-owning thread"
    );
}
