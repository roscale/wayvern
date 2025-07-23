use smithay::reexports::calloop::channel::{self};
use tracing::error;

use crate::flutter_engine_rs::platform_channels::{
    method_call::MethodCall,
    method_channel::{MethodCallHandler, MethodCallHandlerParameters, MethodChannel},
    method_result::MethodResult,
};

pub trait ChannelMethodCallHandler<T> {
    fn channel_call_handler(&mut self) -> channel::Channel<MethodCallHandlerParameters<T>>;
}

impl<T: 'static> ChannelMethodCallHandler<T> for MethodChannel<'_, T> {
    fn channel_call_handler(&mut self) -> channel::Channel<MethodCallHandlerParameters<T>> {
        let (tx, rx) = channel::channel();

        self.set_method_call_handler(Some(Box::new(
            move |method_call: MethodCall<T>, result: Box<dyn MethodResult<T>>| {
                if let Err(e) = tx.send((method_call, result)) {
                    error!("Failed to send method call: {e}");
                }
            },
        ) as MethodCallHandler<T>));

        rx
    }
}
