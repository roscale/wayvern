use std::rc::Rc;

use crate::flutter_engine_rs::flutter_engine_sys::FlutterPlatformMessage;

use crate::flutter_engine_rs::platform_channels::{
    basic_message_channel::BasicMessageChannel, message_codec::MessageCodec,
    method_channel::MethodChannel, method_codec::MethodCodec, method_result::MethodResult,
};

pub type BinaryReply = Option<Box<dyn FnMut(Option<&[u8]>)>>;

pub type BinaryMessageHandler = Option<Box<dyn FnMut(&[u8], BinaryReply)>>;

pub trait BinaryMessenger {
    fn handle_message(&mut self, message: &FlutterPlatformMessage);
    fn send(&mut self, channel: &str, message: &[u8], reply: BinaryReply);
    fn set_message_handler(&mut self, channel: &str, handler: BinaryMessageHandler);

    fn invoke_method<T: 'static, C: MethodCodec<T> + 'static>(
        &mut self,
        codec: C,
        channel_name: &str,
        method: &str,
        arguments: Option<Box<T>>,
        result: Option<Box<dyn MethodResult<T>>>,
    ) where
        Self: Sized,
    {
        let codec = Rc::new(codec);

        let mut method_channel = MethodChannel::new(self, channel_name.to_string(), codec);

        method_channel.invoke_method(method, arguments, result);
    }

    fn send_message<T: 'static, C: MessageCodec<T> + 'static>(
        &mut self,
        codec: C,
        channel_name: &str,
        message: &T,
        reply: BinaryReply,
    ) where
        Self: Sized,
    {
        let codec = Rc::new(codec);

        let mut basic_message_channel =
            BasicMessageChannel::new(self, channel_name.to_string(), codec);

        basic_message_channel.send(message, reply);
    }
}
