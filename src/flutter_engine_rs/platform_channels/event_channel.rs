use std::fmt::Debug;
use std::rc::Rc;

use crate::flutter_engine_rs::platform_channels::{
    binary_messenger::{BinaryMessageHandler, BinaryMessenger},
    encodable_value::EncodableValue,
    event_sink::EventSink,
    event_stream_handler::StreamHandler,
    method_codec::MethodCodec,
};

pub struct EventChannel<'m, T = EncodableValue> {
    messenger: &'m mut dyn BinaryMessenger,
    name: String,
    codec: Rc<dyn MethodCodec<T>>,
}

impl<'m, T> EventChannel<'m, T>
where
    T: Debug,
{
    pub fn new(
        messenger: &'m mut dyn BinaryMessenger,
        name: String,
        codec: Rc<dyn MethodCodec<T>>,
    ) -> Self {
        Self {
            messenger,
            name,
            codec,
        }
    }

    fn set_stream_handler(&mut self, handler: Option<Box<dyn StreamHandler<T>>>) {
        let Some(mut handler) = handler else {
            self.messenger.set_message_handler(&self.name, None);
            return;
        };

        let name = self.name.clone();
        let codec = self.codec.clone();

        let binary_handler: BinaryMessageHandler = Some(Box::new(move |message, mut reply| {
            let mut is_listening = false;

            let method_call = codec.decode_method_call(message);

            let Some(method_call) = method_call else {
                eprintln!("Unable to construct method call from message on channel: {name}");
                reply.as_mut().unwrap()(None);
                return;
            };

            if method_call.method() == "listen" {
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
                    // TODO(roscale): Make messenger Rc.
                    self.messenger,
                    name.clone(),
                    codec.clone(),
                ));

                let arguments = method_call.arguments();

                let error = handler.on_listen(arguments, sink);

                // TODO(roscale): continue: https://github.com/flutter/flutter/blob/09f4c9c71d3725c66bc223931bbb1fade33b194d/engine/src/flutter/shell/platform/common/client_wrapper/include/flutter/event_channel.h#L106
            }
        }));

        self.messenger
            .set_message_handler(&self.name, binary_handler);
    }
}

struct EventSinkImplementation<'m, T = EncodableValue> {
    messenger: &'m mut dyn BinaryMessenger,
    name: String,
    codec: Rc<dyn MethodCodec<T>>,
}

impl<'m, T: 'static> EventSinkImplementation<'m, T> {
    pub fn new(
        messenger: &'m mut dyn BinaryMessenger,
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

impl<'m, T> EventSink<T> for EventSinkImplementation<'m, T> {
    fn success_internal(&mut self, result: Option<T>) {
        let result = self.codec.encode_success_envelope(result.as_ref());
        self.messenger.send(&self.name, &result, None);
    }

    fn error_internal(&mut self, code: String, message: String, details: Option<T>) {
        let result = self
            .codec
            .encode_error_envelope(&code, &message, details.as_ref());
        self.messenger.send(&self.name, &result, None);
    }

    fn end_of_stream_internal(&mut self) {
        self.messenger.send(&self.name, &[], None);
    }
}
