use std::any::Any;
use std::rc::Rc;

use crate::flutter_engine::embedder::FlutterPlatformMessageResponseHandle;

pub type FlutterDesktopMessengerRef = Rc<dyn FlutterDesktopMessenger>;

type FlutterDesktopMessageResponseHandle = Option<FlutterPlatformMessageResponseHandle>;

type FlutterDesktopBinaryReply = Box<dyn FnOnce(Option<&[u8]>, Option<Box<dyn Any>>)>;

pub struct FlutterDesktopMessage<'m> {
    struct_size: usize,
    channel: String,
    pub(crate) message: &'m [u8],
    pub(crate) response_handle: FlutterDesktopMessageResponseHandle,
}

pub type FlutterDesktopMessageCallback = Option<Box<dyn FnMut(Rc<dyn FlutterDesktopMessenger>, &FlutterDesktopMessage, Option<Box<dyn Any>>)>>;

pub trait FlutterDesktopMessenger {
    fn flutter_desktop_messenger_send(&mut self, channel: &str, message: &[u8]);
    fn flutter_desktop_messenger_send_with_reply(&mut self, channel: &str, message: &[u8], reply: FlutterDesktopBinaryReply, user_data: Option<Box<dyn Any>>);
    fn flutter_desktop_messenger_send_response(&mut self, handle: FlutterDesktopMessageResponseHandle, data: &[u8]);
    fn flutter_desktop_messenger_set_callback(&mut self, channel: &str, callback: FlutterDesktopMessageCallback, user_data: Option<Box<dyn Any>>);
    fn flutter_desktop_messenger_add_ref(&mut self);
    fn flutter_desktop_messenger_release(&mut self);
    fn flutter_desktop_messenger_is_available(&mut self) -> bool;
    fn flutter_desktop_messenger_lock(&mut self) -> FlutterDesktopMessengerRef;
    fn flutter_desktop_messenger_unlock(&mut self);
}
