use std::rc::Rc;

pub type BinaryReply = Option<Box<dyn FnOnce(Option<&[u8]>)>>;

pub type BinaryMessageHandler = Option<Rc<dyn FnMut(&[u8], BinaryReply)>>;

pub trait BinaryMessenger {
    fn send(&mut self, channel: &str, message: &[u8], reply: BinaryReply);
    fn set_message_handler(&mut self, channel: &str, handler: BinaryMessageHandler);
}
