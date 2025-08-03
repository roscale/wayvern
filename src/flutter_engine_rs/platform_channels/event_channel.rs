use std::fmt::Debug;
use std::rc::Rc;

use crate::flutter_engine_rs::platform_channels::{
    binary_messenger::{BinaryMessageHandler, BinaryMessenger},
    encodable_value::EncodableValue,
    event_sink::EventSink,
    event_stream_handler::StreamHandler,
    method_codec::MethodCodec,
};

pub struct EventChannel<T = EncodableValue> {
    // SAFETY: It's a pointer because the BinaryMessageHandler captures the messenger and stores the
    // closure into the messenger itself, so it's self referential. I assume Flutter's original code is safe,
    // so I don't want to deal the lifetimes.
    messenger: *mut dyn BinaryMessenger,
    name: String,
    codec: Rc<dyn MethodCodec<T>>,
}

impl<T> EventChannel<T>
where
    T: Debug + 'static,
{
    pub fn new(
        messenger: *mut dyn BinaryMessenger,
        name: String,
        codec: Rc<dyn MethodCodec<T>>,
    ) -> Self {
        Self {
            messenger,
            name,
            codec,
        }
    }

    pub fn set_stream_handler(&mut self, handler: Option<Box<dyn StreamHandler<T>>>) {
        let Some(mut handler) = handler else {
            unsafe { (*self.messenger).set_message_handler(&self.name, None) };
            return;
        };

        let messenger = self.messenger;
        let name = self.name.clone();
        let codec = self.codec.clone();

        let mut is_listening = false;

        let binary_handler: BinaryMessageHandler = Some(Box::new(move |message, mut reply| {
            let method_call = codec.decode_method_call(message);
            let reply = reply.as_mut().unwrap();

            let Some(method_call) = method_call else {
                eprintln!("Unable to construct method call from message on channel: {name}");
                reply(None);
                return;
            };

            match method_call.method() {
                "listen" => {
                    if is_listening {
                        let error = handler.on_cancel(None);

                        if let Some(error) = error {
                            eprintln!(
                                "Failed to cancel existing stream: {}, {}, {:?}",
                                error.code, error.message, error.details,
                            )
                        }
                    }

                    is_listening = true;

                    let sink: Box<dyn EventSink<T>> = Box::new(EventSinkImplementation::new(
                        // SAFETY: This is where the messenger is captured.
                        messenger,
                        name.clone(),
                        codec.clone(),
                    ));

                    let error = handler.on_listen(method_call.arguments(), sink);

                    let result = match error {
                        Some(error) => codec.encode_error_envelope(
                            &error.code,
                            &error.message,
                            error.details.as_ref(),
                        ),
                        None => codec.encode_success_envelope(None),
                    };

                    reply(Some(result));
                }

                "cancel" => {
                    let result = if is_listening {
                        let error = handler.on_cancel(method_call.arguments());

                        let result = match error {
                            Some(error) => codec.encode_error_envelope(
                                &error.code,
                                &error.message,
                                error.details.as_ref(),
                            ),
                            None => codec.encode_success_envelope(None),
                        };

                        is_listening = false;

                        result
                    } else {
                        codec.encode_error_envelope("error", "No active stream to cancel", None)
                    };

                    reply(Some(result));
                }

                _ => {
                    reply(None);
                }
            }
        }));

        // SAFETY: This is where the messenger inside the binary_handler is stored in the messenger itself.
        unsafe { (*self.messenger).set_message_handler(&self.name, binary_handler) };
    }
}

struct EventSinkImplementation<T = EncodableValue> {
    messenger: *mut dyn BinaryMessenger,
    name: String,
    codec: Rc<dyn MethodCodec<T>>,
}

impl<T: 'static> EventSinkImplementation<T> {
    pub fn new(
        messenger: *mut dyn BinaryMessenger,
        name: String,
        codec: Rc<dyn MethodCodec<T>>,
    ) -> Self {
        Self {
            messenger,
            name,
            codec,
        }
    }
}

impl<T> EventSink<T> for EventSinkImplementation<T> {
    fn success_internal(&mut self, result: Option<T>) {
        let result = self.codec.encode_success_envelope(result.as_ref());
        unsafe { (*self.messenger).send(&self.name, &result, None) };
    }

    fn error_internal(&mut self, code: String, message: String, details: Option<T>) {
        let result = self
            .codec
            .encode_error_envelope(&code, &message, details.as_ref());
        unsafe { (*self.messenger).send(&self.name, &result, None) };
    }

    fn end_of_stream_internal(&mut self) {
        unsafe { (*self.messenger).send(&self.name, &[], None) };
    }
}
