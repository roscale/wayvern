use crate::flutter_engine_rs::platform_channels::{
    encodable_value::EncodableValue, event_sink::EventSink,
};

pub struct StreamHandlerError<T = EncodableValue> {
    pub code: String,
    pub message: String,
    pub details: Option<T>,
}

impl<T> StreamHandlerError<T> {
    pub fn new(code: String, message: String, details: Option<T>) -> Self {
        Self {
            code,
            message,
            details,
        }
    }
}

pub trait StreamHandler<T = EncodableValue> {
    fn on_listen(
        &mut self,
        arguments: Option<&T>,
        events: Box<dyn EventSink<T>>,
    ) -> Option<StreamHandlerError<T>> {
        self.on_listen_internal(arguments, events)
    }

    fn on_cancel(&mut self, arguments: Option<&T>) -> Option<StreamHandlerError<T>> {
        self.on_cancel_internal(arguments)
    }

    fn on_listen_internal(
        &mut self,
        arguments: Option<&T>,
        events: Box<dyn EventSink<T>>,
    ) -> Option<StreamHandlerError<T>>;

    fn on_cancel_internal(&mut self, arguments: Option<&T>) -> Option<StreamHandlerError<T>>;
}
