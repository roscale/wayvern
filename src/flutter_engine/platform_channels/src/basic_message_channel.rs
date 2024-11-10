use std::rc::Rc;

use crate::binary_messenger::{
    BinaryMessageHandler, BinaryMessenger, BinaryReply,
};
use crate::encodable_value::EncodableValue;
use crate::message_codec::MessageCodec;

pub type MessageReply<T> = Option<Box<dyn FnMut(Option<T>)>>;

type MessageHandler<T> = Option<Box<dyn FnMut(Option<T>, MessageReply<T>)>>;

pub struct BasicMessageChannel<'m, T = EncodableValue> where T: 'static {
    messenger: &'m mut dyn BinaryMessenger,
    name: String,
    codec: Rc<dyn MessageCodec<T>>,
}

impl<'m, T> BasicMessageChannel<'m, T> {
    pub fn new(
        messenger: &'m mut dyn BinaryMessenger,
        name: String,
        codec: Rc<dyn MessageCodec<T>>,
    ) -> Self {
        Self {
            messenger,
            name,
            codec,
        }
    }

    pub fn send(&mut self, message: &T, reply: BinaryReply) {
        let message = self.codec.encode_message(message);
        self.messenger.send(&self.name, &message, reply);
    }

    pub fn set_message_handler(&mut self, handler: MessageHandler<T>) {
        let mut handler = if let Some(handler) = handler {
            handler
        } else {
            self.messenger.set_message_handler(&self.name, None);
            return;
        };

        let codec = self.codec.clone();
        let channel_name = self.name.clone();

        let binary_handler: BinaryMessageHandler =
            Some(Box::new(move |message: &[u8], mut reply: BinaryReply| {
                let message = codec.decode_message(message);
                let message = if let Some(message) = message {
                    message
                } else {
                    eprintln!("Unable to decode message on channel {}", channel_name);
                    reply.unwrap()(None);
                    return;
                };

                let codec = codec.clone();

                let unencoded_reply: MessageReply<T> =
                    Some(Box::new(move |response: Option<T>| {
                        let response = codec.encode_message(&response.unwrap());
                        reply.as_mut().unwrap()(Some(&response));
                    }));

                handler(Some(message), unencoded_reply);
            }));

        self.messenger.set_message_handler(&self.name, binary_handler);
    }
}