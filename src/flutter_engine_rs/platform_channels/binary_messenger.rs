use crate::flutter_engine_rs::flutter_engine_sys::FlutterPlatformMessage;

pub type BinaryReply = Option<Box<dyn FnMut(Option<Vec<u8>>)>>;

pub type BinaryMessageHandler = Option<Box<dyn FnMut(&[u8], BinaryReply)>>;

pub trait BinaryMessenger {
    fn handle_message(&mut self, message: &FlutterPlatformMessage);
    fn send(&mut self, channel: &str, message: &[u8], reply: BinaryReply);
    fn set_message_handler(&mut self, channel: &str, handler: BinaryMessageHandler);
}
