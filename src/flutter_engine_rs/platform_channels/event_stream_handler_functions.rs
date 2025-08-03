use crate::flutter_engine_rs::platform_channels::{
    encodable_value::EncodableValue,
    event_sink::EventSink,
    event_stream_handler::{StreamHandler, StreamHandlerError},
};

type StreamHandlerListen<T> =
    Option<Box<dyn FnMut(Option<&T>, Box<dyn EventSink<T>>) -> Option<StreamHandlerError<T>>>>;

type StreamHandlerCancel<T> = Option<Box<dyn FnMut(Option<&T>) -> Option<StreamHandlerError<T>>>>;

pub struct StreamHandlerFunctions<T = EncodableValue> {
    on_listen: StreamHandlerListen<T>,
    on_cancel: StreamHandlerCancel<T>,
}

impl<T> StreamHandlerFunctions<T> {
    pub fn new(on_listen: StreamHandlerListen<T>, on_cancel: StreamHandlerCancel<T>) -> Self {
        Self {
            on_listen,
            on_cancel,
        }
    }
}

impl StreamHandler for StreamHandlerFunctions {
    fn on_listen_internal(
        &mut self,
        arguments: Option<&EncodableValue>,
        events: Box<dyn EventSink<EncodableValue>>,
    ) -> Option<StreamHandlerError<EncodableValue>> {
        match &mut self.on_listen {
            Some(on_listen) => on_listen(arguments, events),
            None => Some(StreamHandlerError::new(
                "error".to_string(),
                "No OnListen handler set".to_string(),
                None,
            )),
        }
    }

    fn on_cancel_internal(
        &mut self,
        arguments: Option<&EncodableValue>,
    ) -> Option<StreamHandlerError<EncodableValue>> {
        match &mut self.on_cancel {
            Some(on_cancel) => on_cancel(arguments),
            None => Some(StreamHandlerError::new(
                "error".to_string(),
                "No OnCancel handler set".to_string(),
                None,
            )),
        }
    }
}
