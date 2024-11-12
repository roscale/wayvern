//! https://api.flutter.dev/flutter/services/SystemChannels/textInput-constant.html

use crate::server_state::Common;
use input_linux::sys::KEY_ENTER;
use platform_channels::method_call::MethodCall;
use platform_channels::method_channel::MethodChannel;
use platform_channels::method_result::MethodResult;
use platform_channels::text_imput_model::TextInputModel;
use platform_channels::text_range::TextRange;
use serde_json::{json, Value};
use smithay::reexports::calloop::channel::Event;

#[derive(Default)]
pub struct TextInput {
    active_model: Option<TextInputModel>,
    client_id: u64,
    input_type: String,
    input_action: String,
}

impl TextInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_active(&self) -> bool {
        self.active_model.is_some()
    }

    pub fn press_key(
        &mut self,
        keycode: u32,
        code_point: Option<char>,
        channel: &mut MethodChannel<Value>,
    ) -> bool {
        let model = match self.active_model.as_mut() {
            None => return false,
            Some(model) => model,
        };

        let changed = match keycode as i32 {
            // Navigation keys like arrow keys are handled by Flutter, but the following keys are not.
            KEY_ENTER => {
                self.press_enter(channel);
                false
            }
            _ => {
                if let Some(code_point) = code_point {
                    // A regular character.
                    model.add_char_point(code_point);
                    true
                } else {
                    false
                }
            }
        };
        if changed {
            self.send_state_update(channel);
        }
        changed
    }

    fn set_client(&mut self, id: u64, input_type: &str, input_action: &str) {
        self.active_model = Some(TextInputModel::default());
        self.client_id = id;
        self.input_type = input_type.to_string();
        self.input_action = input_action.to_string();
    }

    fn clear_client(&mut self) {
        self.active_model = None;
        self.client_id = 0;
        self.input_type = "".to_string();
        self.input_action = "".to_string();
    }

    fn send_state_update(&mut self, channel: &mut MethodChannel<Value>) {
        let model = self.active_model.as_ref().unwrap();

        let state = json!({
            "text": model.get_text(),
            "selectionBase": model.selection().base(),
            "selectionExtent": model.selection().extent(),
            "composingBase": model.composing_range().base(),
            "composingExtent": model.composing_range().extent(),
        });

        channel.invoke_method(
            "TextInputClient.updateEditingState",
            Some(Box::new(json!([self.client_id, state,]))),
            None,
        );
    }

    fn press_enter(&mut self, channel: &mut MethodChannel<Value>) {
        if self.input_type == "TextInputType.multiline" {
            self.active_model.as_mut().unwrap().add_char_point('\n');
            self.send_state_update(channel);
        }
        channel.invoke_method(
            "TextInputClient.performAction",
            Some(Box::new(json!([self.client_id, self.input_action,]))),
            None,
        );
    }
}

pub fn text_input_channel_method_call_handler(
    event: Event<(
        MethodCall<Value>,
        Box<dyn MethodResult<Value>>,
    )>,
    _: &mut (),
    data: &mut Common,
) {
    let text_input = &mut data.flutter_engine.text_input;

    if let Event::Msg((method_call, mut result)) = event {
        let arguments = method_call.arguments();
        match method_call.method() {
            "TextInput.setClient" => {
                let client_id = arguments.unwrap().get(0).unwrap().as_u64().unwrap();
                let config = arguments.unwrap().get(1).unwrap().as_object().unwrap();

                let input_type = config
                    .get("inputType")
                    .unwrap()
                    .get("name")
                    .unwrap()
                    .as_str()
                    .unwrap();
                let input_action = config.get("inputAction").unwrap().as_str().unwrap();

                text_input.set_client(client_id, input_type, input_action);
            }
            "TextInput.clearClient" => text_input.clear_client(),
            "TextInput.setEditingState" => {
                let object = arguments.unwrap().as_object().unwrap();

                let text = object.get("text").unwrap().as_str().unwrap();
                let mut selection_base = object.get("selectionBase").unwrap().as_i64().unwrap();
                let mut selection_extent = object.get("selectionExtent").unwrap().as_i64().unwrap();
                let mut composing_base = object.get("composingBase").unwrap().as_i64().unwrap();
                let mut composing_extent = object.get("composingExtent").unwrap().as_i64().unwrap();

                // Flutter uses -1/-1 for invalid; translate that to 0/0 for the model.
                if selection_base == -1 && selection_extent == -1 {
                    selection_base = 0;
                    selection_extent = 0;
                }
                if composing_base == -1 && composing_extent == -1 {
                    composing_base = 0;
                    composing_extent = 0;
                }

                text_input.active_model.as_mut().unwrap().set_text(
                    text,
                    TextRange::new(selection_base as usize, selection_extent as usize),
                    TextRange::new(composing_base as usize, composing_extent as usize),
                );
            }
            // There are other methods that we don't implement.
            _ => {}
        }
        result.success(None);
    }
}