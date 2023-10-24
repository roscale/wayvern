use crate::flutter_engine::platform_channels::json_message_codec::JsonMessageCodec;
use crate::flutter_engine::platform_channels::message_codec::MessageCodec;
use crate::flutter_engine::platform_channels::method_call::MethodCall;
use crate::flutter_engine::platform_channels::method_codec::MethodCodec;
use crate::flutter_engine::platform_channels::method_result::MethodResult;

const K_MESSAGE_METHOD_KEY: &str = "method";
const K_MESSAGE_ARGUMENTS_KEY: &str = "args";

struct JsonMethodCodec {}

impl MethodCodec<serde_json::Value> for JsonMethodCodec {
    fn decode_method_call_internal(&self, message: &[u8]) -> Option<MethodCall<serde_json::Value>> {
        let mut json_message = JsonMessageCodec {}.decode_message(message)?;
        let arguments = json_message.get_mut(K_MESSAGE_ARGUMENTS_KEY).map(|v| Box::new(v.take()));
        let method_name = json_message.get(K_MESSAGE_METHOD_KEY)?.as_str()?;
        Some(MethodCall::new(method_name.to_string(), arguments))
    }

    fn encode_method_call_internal(&self, method_call: &MethodCall<serde_json::Value>) -> Vec<u8> {
        let message = serde_json::json!({
            K_MESSAGE_METHOD_KEY: method_call.method(),
            K_MESSAGE_ARGUMENTS_KEY: method_call.arguments(),
        });
        JsonMessageCodec {}.encode_message(&message)
    }

    fn encode_success_envelope_internal(&self, result: Option<&serde_json::Value>) -> Vec<u8> {
        let envelope = serde_json::json!([result]);
        JsonMessageCodec {}.encode_message(&envelope)
    }

    fn encode_error_envelope_internal(&self, code: &str, message: &str, details: Option<&serde_json::Value>) -> Vec<u8> {
        let envelope = serde_json::json!([code, message, details]);
        JsonMessageCodec {}.encode_message(&envelope)
    }

    fn decode_and_process_response_envelope_internal(&self, response: &[u8], result: &mut dyn MethodResult<serde_json::Value>) -> bool {
        (|| {
            let mut json_response = JsonMessageCodec {}.decode_message(response)?;
            let json_response = json_response.as_array_mut()?;
            match json_response.len() {
                1 => {
                    result.success(if json_response[0].is_null() {
                        None
                    } else {
                        Some(&json_response[0])
                    });
                    Some(())
                }
                3 => {
                    let code = json_response[0].as_str()?;
                    let message = json_response[1].as_str()?;
                    let details = if json_response[2].is_null() {
                        None
                    } else {
                        Some(&json_response[2])
                    };
                    result.error(code, message, details);
                    Some(())
                }
                _ => None,
            }
        })().is_some()
    }
}
