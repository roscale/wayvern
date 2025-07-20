use std::collections::HashMap;
use std::ffi::{CString, c_void};
use std::mem::size_of;
use std::ptr::{null, null_mut};
use std::sync::{Mutex, mpsc};

use crate::flutter_engine_rs::flutter_engine_sys::{
    FlutterDataCallback, FlutterEngine, FlutterEngineResult_kSuccess,
    FlutterEngineSendPlatformMessage, FlutterEngineSendPlatformMessageResponse,
    FlutterPlatformMessage, FlutterPlatformMessageCreateResponseHandle,
    FlutterPlatformMessageReleaseResponseHandle,
};
use crate::flutter_engine_rs::platform_channels::binary_messenger::{
    BinaryMessageHandler, BinaryMessenger, BinaryReply,
};
use crate::flutter_engine_rs::task_runner::{PostCallback, ScheduledTask};

pub struct BinaryMessengerImpl {
    flutter_engine: FlutterEngine,
    handlers: Mutex<HashMap<String, BinaryMessageHandler>>,
    tx_tasks: mpsc::Sender<ScheduledTask>,
}

impl BinaryMessengerImpl {
    pub fn new(tx_tasks: mpsc::Sender<ScheduledTask>) -> Self {
        Self {
            flutter_engine: null_mut(),
            handlers: Mutex::new(HashMap::new()),
            tx_tasks,
        }
    }

    pub fn set_flutter_engine(&mut self, flutter_engine: FlutterEngine) {
        self.flutter_engine = flutter_engine;
    }
}

struct Captures {
    reply: BinaryReply,
}

extern "C" fn message_reply(data: *const u8, data_size: usize, user_data: *mut c_void) {
    let captures = unsafe { Box::from_raw(user_data as *mut Captures) };

    let data = if !data.is_null() && data_size != 0 {
        Some(unsafe { std::slice::from_raw_parts(data, data_size).to_vec() })
    } else {
        None
    };

    captures.reply.unwrap()(data);
}

impl BinaryMessenger for BinaryMessengerImpl {
    fn handle_message(&mut self, message: &FlutterPlatformMessage) {
        let channel = unsafe { std::ffi::CStr::from_ptr(message.channel) };
        let channel = channel.to_str().unwrap();
        if let Some(handler) = self.handlers.lock().unwrap().get_mut(channel) {
            let message_bytes =
                unsafe { std::slice::from_raw_parts(message.message, message.message_size) };

            let tx_tasks = self.tx_tasks.clone();
            let engine_handle = self.flutter_engine;
            let response_handle = message.response_handle;

            let reply_handler: BinaryReply = Some(Box::new(move |reply: Option<Vec<u8>>| {
                tx_tasks.post_callback(move || {
                    let (data, data_length) = reply
                        .as_ref()
                        .map(|v| (v.as_ptr(), v.len()))
                        .unwrap_or((null(), 0));

                    unsafe {
                        FlutterEngineSendPlatformMessageResponse(
                            engine_handle,
                            response_handle,
                            data,
                            data_length,
                        )
                    };
                });
            }));

            handler.as_mut().unwrap()(message_bytes, reply_handler);
        } else {
            // This one already runs on the platform thread, no need to wrap the call.
            unsafe {
                FlutterEngineSendPlatformMessageResponse(
                    self.flutter_engine,
                    message.response_handle,
                    null(),
                    0,
                )
            };
        }
    }

    fn send(&mut self, channel: &str, message: &[u8], reply: BinaryReply) {
        if reply.is_some() {
            let captures = Box::into_raw(Box::new(Captures { reply }));
            let result = self.send_message_with_reply(
                channel,
                message,
                Some(message_reply),
                captures as *mut c_void,
            );
            if !result {
                let _ = unsafe { Box::from_raw(captures) };
            }
        } else {
            self.send_message_with_reply(channel, message, None, null_mut());
        };
    }

    fn set_message_handler(&mut self, channel: &str, handler: BinaryMessageHandler) {
        if handler.is_some() {
            self.handlers
                .lock()
                .unwrap()
                .insert(channel.to_string(), handler);
        } else {
            self.handlers.lock().unwrap().remove(channel);
        };
    }
}

impl BinaryMessengerImpl {
    fn send_message_with_reply(
        &mut self,
        channel: &str,
        message: &[u8],
        reply: FlutterDataCallback,
        user_data: *mut c_void,
    ) -> bool {
        let flutter_engine = self.flutter_engine;

        let mut response_handle = null_mut();
        if reply.is_some() && !user_data.is_null() {
            let result = self.tx_tasks.post_callback(move || unsafe {
                FlutterPlatformMessageCreateResponseHandle(
                    flutter_engine,
                    reply,
                    user_data,
                    &mut response_handle,
                )
            });

            if result != FlutterEngineResult_kSuccess {
                eprintln!("Failed to create response handle");
                return false;
            }
        }

        let channel_cstring = CString::new(channel).unwrap();
        let platform_message = FlutterPlatformMessage {
            struct_size: size_of::<FlutterPlatformMessage>(),
            channel: channel_cstring.as_ptr(),
            message: message.as_ptr(),
            message_size: message.len(),
            response_handle,
        };

        let message_result = self.tx_tasks.post_callback(move || unsafe {
            FlutterEngineSendPlatformMessage(flutter_engine, &platform_message)
        });

        if !response_handle.is_null() {
            self.tx_tasks.post_callback(move || {
                unsafe {
                    FlutterPlatformMessageReleaseResponseHandle(flutter_engine, response_handle)
                };
            });
        }
        message_result == FlutterEngineResult_kSuccess
    }
}
