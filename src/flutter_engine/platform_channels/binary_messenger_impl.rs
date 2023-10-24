use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;
use crate::flutter_engine::platform_channels::binary_messenger::{BinaryMessageHandler, BinaryMessenger, BinaryReply};
use crate::flutter_engine::platform_channels::flutter_messenger::{FlutterDesktopMessage, FlutterDesktopMessageCallback, FlutterDesktopMessenger, FlutterDesktopMessengerRef};

struct BinaryMessengerImpl {
    messenger: Rc<dyn FlutterDesktopMessenger>,
    handlers: HashMap<String, BinaryMessageHandler>,
}

impl BinaryMessengerImpl {
    pub fn new(messenger: Rc<dyn FlutterDesktopMessenger>) -> Self {
        Self {
            messenger,
            handlers: HashMap::new(),
        }
    }
}

impl BinaryMessenger for BinaryMessengerImpl {
    fn send(&mut self, channel: &str, message: &[u8], reply: BinaryReply) {
        let channel = channel.to_string();
        let reply: BinaryReply = if let Some(reply) = reply {
            Some(reply)
        } else {
            Rc::get_mut(&mut self.messenger).unwrap().flutter_desktop_messenger_send(channel.as_str(), message);
            return;
        };

        let message_reply = move |data: Option<&[u8]>, user_data: Option<Box<dyn Any>>| {
            let reply = user_data.unwrap().downcast::<BinaryReply>().unwrap().unwrap();
            reply(data);
        };
        Rc::get_mut(&mut self.messenger).unwrap().flutter_desktop_messenger_send_with_reply(channel.as_str(), message, Box::new(message_reply), Some(Box::new(reply)));
    }

    fn set_message_handler(&mut self, channel: &str, handler: BinaryMessageHandler) {
        let handler: BinaryMessageHandler = if let Some(handler) = handler {
            Some(handler)
        } else {
            self.handlers.remove(channel);
            Rc::get_mut(&mut self.messenger).unwrap().flutter_desktop_messenger_set_callback(channel, None, None);
            return;
        };

        self.handlers.insert(channel.to_string(), handler);
        let message_handler: BinaryMessageHandler = self.handlers[channel].as_ref().cloned();

        let a = move |mut messenger: FlutterDesktopMessengerRef, message: &FlutterDesktopMessage, user_data: Option<Box<dyn Any>>| {
            let response_handle = message.response_handle;
            let reply_handler: BinaryReply = Some(Box::new(move |data: Option<&[u8]>| {
                if response_handle.is_some() {
                    Rc::get_mut(&mut messenger).unwrap().flutter_desktop_messenger_send_response(response_handle, data.unwrap());
                } else {
                    eprintln!("Error: Response can be set only once. Ignoring duplicate response.");
                }
            }));
            let message_handler = user_data.unwrap().downcast::<BinaryMessageHandler>().unwrap();
            Rc::get_mut(&mut message_handler.unwrap()).unwrap()(message.message, reply_handler);
        };

        let callback: FlutterDesktopMessageCallback = Some(Box::new(a));

        Rc::get_mut(&mut self.messenger).unwrap().flutter_desktop_messenger_set_callback(channel, callback, Some(Box::new(message_handler)));
    }
}

// fn forward_to_handler(mut messenger: FlutterDesktopMessengerRef, mut message: &FlutterDesktopMessage, user_data: Option<Box<dyn Any>>) {
//     let response_handle = message.response_handle;
//     let reply_handler: BinaryReply = Some(Box::new(move |data: Option<&[u8]>| {
//         if response_handle.is_some() {
//             Rc::get_mut(&mut messenger).unwrap().flutter_desktop_messenger_send_response(response_handle, data.unwrap());
//         } else {
//             eprintln!("Error: Response can be set only once. Ignoring duplicate response.");
//         }
//     }));
//     let message_handler = user_data.unwrap().downcast::<BinaryMessageHandler>().unwrap();
//     Rc::get_mut(&mut message_handler.unwrap()).unwrap()(message.message, reply_handler);
// }
