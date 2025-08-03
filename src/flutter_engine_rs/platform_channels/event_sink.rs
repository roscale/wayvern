use crate::flutter_engine_rs::platform_channels::encodable_value::EncodableValue;

pub trait EventSink<T = EncodableValue> {
    fn success(&mut self, result: Option<T>) {
        self.success_internal(result);
    }

    fn error(&mut self, code: String, message: String, details: Option<T>) {
        self.error_internal(code, message, details);
    }

    fn end_of_stream(&mut self) {
        self.end_of_stream_internal();
    }

    fn success_internal(&mut self, result: Option<T>);

    fn error_internal(&mut self, code: String, message: String, details: Option<T>);

    fn end_of_stream_internal(&mut self);
}

