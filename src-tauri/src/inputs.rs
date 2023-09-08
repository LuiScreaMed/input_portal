use std::{collections::HashMap, time::SystemTime};

use rdev::{listen, Button, Event, Key};
use serde::{Deserialize, Serialize};
use serde_with::{formats::Flexible, serde_as, TimestampMilliSeconds};
use tokio::sync::broadcast::Sender;

use crate::{
    keys,
    message::{Message, MessageData, MessageType},
    CONFIG,
};

/// 输入开源
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputSource {
    Keyboard,
    MouseMove,
    MouseButton,
    MouseWheel,
}

/// 输入的信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum InputInfo {
    /// 按键名，是否正在按住
    #[serde(rename_all = "snake_case")]
    Pressing { name: String, pressing: bool },
    /// 鼠标坐标和屏幕大小
    Coord {
        x: f64,
        y: f64,
        /// 备用
        screen_size: (u64, u64),
    },
    /// 滚轮方向
    Roll { delta_x: i64, delta_y: i64 },
}

/// 输入消息，用于打包通过sse发送到client
#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputMessage {
    /// 输入来源
    pub source: InputSource,
    /// 输入信息
    pub info: InputInfo,
    /// 输入时间
    #[serde_as(as = "TimestampMilliSeconds<String, Flexible>")]
    pub time: SystemTime,
}

#[derive(Debug, Clone)]
pub enum KeyButton {
    Key(Key),
    Button(Button),
}

pub struct Handler {
    /// 当前按下的键盘按键
    pub pressing_keys: HashMap<String, bool>,
    /// 当前按下的鼠标按键
    pub pressing_mouse_buttons: HashMap<String, bool>,
    /// 输入消息发送器，发送到服务端
    pub sender: Sender<Message>,
    /// 屏幕尺寸，备用
    pub screen_size: (u64, u64),
    // pub message_sender: Sender<InputMessage>,
    // 按键回调接收器
    // pub receiver: UnboundedReceiver<Event>,
    // 是否发送输入消息
    // pub enable: bool,
    // 是否发送鼠标移动消息
    // pub mouse_move_enable: bool,
}

impl Handler {
    /// 实例化
    pub fn new(
        // enable: &'static bool,
        // mouse_move_enable: &'static bool,
        // message_sender: Sender<InputMessage>,
        sender: Sender<Message>,
    ) -> Self {
        Self {
            pressing_keys: HashMap::new(),
            pressing_mouse_buttons: HashMap::new(),
            sender,
            screen_size: rdev::display_size().unwrap_or((1920, 1080)),
            // enable: enable.clone(),
            // mouse_move_enable: mouse_move_enable.clone(),
            // message_sender,
        }
    }

    fn send(&self, data: InputMessage) {
        if let Err(_) = self.sender.send(Message {
            r#type: MessageType::Input,
            data: MessageData::InputMessage(data),
        }) {
            // eprintln!("Error sending input message: {:?}", error);
        };
    }

    // 按下或按住按键时
    pub fn on_press(&mut self, event: Event, source: InputSource, key_button: KeyButton) {
        let name_res = match key_button {
            KeyButton::Key(key) => get_key_name(key),
            KeyButton::Button(button) => get_mouse_button_name(button),
        };
        if let Err(keycode) = name_res {
            eprintln!("name error: {}", keycode);
            return;
        }
        let name = name_res.unwrap();
        // 判断是键盘按键还是鼠标按钮，并获取相应hashmap的引用
        let keymap = if let InputSource::Keyboard = source {
            &mut self.pressing_keys
        } else {
            &mut self.pressing_mouse_buttons
        };
        // 按键之前不是按住状态，加入按住状态并发送按下消息
        if let None = keymap.get(name) {
            keymap.insert(name.to_string(), true);
            self.send(InputMessage {
                source,
                info: InputInfo::Pressing {
                    name: name.to_string(),
                    pressing: true,
                },
                time: event.time,
            });
        }
    }

    // 放开按键时
    pub fn on_release(&mut self, event: Event, source: InputSource, key_button: KeyButton) {
        let name_res = match key_button {
            KeyButton::Key(key) => get_key_name(key),
            KeyButton::Button(button) => get_mouse_button_name(button),
        };
        if let Err(keycode) = name_res {
            eprintln!("name error: {}", keycode);
            return;
        }
        let name = name_res.unwrap();
        // 判断是键盘按键还是鼠标按钮，并获取相应hashmap的引用
        let keymap = if let InputSource::Keyboard = source {
            &mut self.pressing_keys
        } else {
            &mut self.pressing_mouse_buttons
        };
        // 按键之前是按住状态，去除按住状态并发送抬起消息
        if let Some(_) = keymap.get(name) {
            keymap.remove(&name.to_string());
            self.send(InputMessage {
                source,
                info: InputInfo::Pressing {
                    name: name.to_string(),
                    pressing: false,
                },
                time: event.time,
            });
        }
    }

    // 移动鼠标时
    pub fn on_mouse_move(&mut self, x: f64, y: f64, time: SystemTime) {
        self.send(InputMessage {
            source: InputSource::MouseMove,
            info: InputInfo::Coord {
                x,
                y,
                screen_size: self.screen_size,
            },
            time,
        });
    }

    // 滚动滚轮时
    pub fn on_mouse_scroll(&mut self, delta_x: i64, delta_y: i64, time: SystemTime) {
        self.send(InputMessage {
            source: InputSource::MouseWheel,
            info: InputInfo::Roll { delta_x, delta_y },
            time,
        });
    }
}

/// 处理输入
pub fn start(sender: Sender<Message>) {
    let mut handler = Handler::new(sender);

    if let Err(error) = listen(move |event| {
        if !unsafe { CONFIG.lock().unwrap().enable } {
            return ();
        }
        match event.event_type {
            rdev::EventType::KeyPress(key) => {
                handler.on_press(event, InputSource::Keyboard, KeyButton::Key(key))
            }
            rdev::EventType::KeyRelease(key) => {
                handler.on_release(event, InputSource::Keyboard, KeyButton::Key(key))
            }
            rdev::EventType::ButtonPress(button) => {
                handler.on_press(event, InputSource::MouseButton, KeyButton::Button(button))
            }
            rdev::EventType::ButtonRelease(button) => {
                handler.on_release(event, InputSource::MouseButton, KeyButton::Button(button))
            }
            rdev::EventType::MouseMove { x, y } => {
                // 如果不处理鼠标移动事件就返回
                if !unsafe { CONFIG.lock().unwrap().mouse_move_enable } {
                    return ();
                }
                handler.on_mouse_move(x, y, event.time);
            }
            rdev::EventType::Wheel { delta_x, delta_y } => {
                handler.on_mouse_scroll(delta_x, delta_y, event.time)
            }
        }
    }) {
        eprintln!("Error: {:?}", error)
    }
}

/// 通过Button获取鼠标按键名
fn get_mouse_button_name(button: Button) -> Result<&'static str, u32> {
    match button {
        Button::Left => Ok(keys::MOUSE_LEFT),
        Button::Right => Ok(keys::MOUSE_RIGHT),
        Button::Middle => Ok(keys::MOUSE_MIDDLE),
        Button::Unknown(keycode) => match keycode {
            1 => Ok(keys::MOUSE_4),
            2 => Ok(keys::MOUSE_5),
            3 => Ok(keys::MOUSE_6),
            code => Err(code.into()),
        },
    }
}

/// 通过Key获取按键名
fn get_key_name(key: Key) -> Result<&'static str, u32> {
    match key {
        Key::Alt => Ok(keys::ALT_LEFT),
        Key::AltGr => Ok(keys::ALT_RIGHT),
        Key::Backspace => Ok(keys::BACKSPACE),
        Key::CapsLock => Ok(keys::CAPSLOCK),
        Key::ControlLeft => Ok(keys::CONTROL_LEFT),
        Key::ControlRight => Ok(keys::CONTROL_RIGHT),
        Key::Delete => Ok(keys::DELETE),
        Key::DownArrow => Ok(keys::DOWNARROW),
        Key::End => Ok(keys::END),
        Key::Escape => Ok(keys::ESCAPE),
        Key::F1 => Ok(keys::F1),
        Key::F10 => Ok(keys::F10),
        Key::F11 => Ok(keys::F11),
        Key::F12 => Ok(keys::F12),
        Key::F2 => Ok(keys::F2),
        Key::F3 => Ok(keys::F3),
        Key::F4 => Ok(keys::F4),
        Key::F5 => Ok(keys::F5),
        Key::F6 => Ok(keys::F6),
        Key::F7 => Ok(keys::F7),
        Key::F8 => Ok(keys::F8),
        Key::F9 => Ok(keys::F9),
        Key::Home => Ok(keys::HOME),
        Key::LeftArrow => Ok(keys::LEFT_ARROW),
        // also known as "windows", "super", and "command"
        Key::MetaLeft => Ok(keys::META_LEFT),
        // also known as "windows", "super", and "command"
        Key::MetaRight => Ok(keys::META_RIGHT),
        Key::PageDown => Ok(keys::PAGEDOWN),
        Key::PageUp => Ok(keys::PAGEUP),
        Key::Return => Ok(keys::RETURN),
        Key::RightArrow => Ok(keys::RIGHT_ARROW),
        Key::ShiftLeft => Ok(keys::SHIFT_LEFT),
        Key::ShiftRight => Ok(keys::SHIFT_RIGHT),
        Key::Space => Ok(keys::SPACE),
        Key::Tab => Ok(keys::TAB),
        Key::UpArrow => Ok(keys::UP_ARROW),
        Key::PrintScreen => Ok(keys::PRINT_SCREEN),
        Key::ScrollLock => Ok(keys::SCROLL_LOCK),
        Key::Pause => Ok(keys::PAUSE),
        Key::NumLock => Ok(keys::NUM_LOCK),
        Key::BackQuote => Ok(keys::BACK_QUOTE),
        Key::Num1 => Ok(keys::NUM_1),
        Key::Num2 => Ok(keys::NUM_2),
        Key::Num3 => Ok(keys::NUM_3),
        Key::Num4 => Ok(keys::NUM_4),
        Key::Num5 => Ok(keys::NUM_5),
        Key::Num6 => Ok(keys::NUM_6),
        Key::Num7 => Ok(keys::NUM_7),
        Key::Num8 => Ok(keys::NUM_8),
        Key::Num9 => Ok(keys::NUM_9),
        Key::Num0 => Ok(keys::NUM_0),
        Key::Minus => Ok(keys::MINUS),
        Key::Equal => Ok(keys::EQUAL),
        Key::KeyQ => Ok(keys::KEY_Q),
        Key::KeyW => Ok(keys::KEY_W),
        Key::KeyE => Ok(keys::KEY_E),
        Key::KeyR => Ok(keys::KEY_R),
        Key::KeyT => Ok(keys::KEY_T),
        Key::KeyY => Ok(keys::KEY_Y),
        Key::KeyU => Ok(keys::KEY_U),
        Key::KeyI => Ok(keys::KEY_I),
        Key::KeyO => Ok(keys::KEY_O),
        Key::KeyP => Ok(keys::KP_0),
        Key::LeftBracket => Ok(keys::LEFT_BRACKET),
        Key::RightBracket => Ok(keys::RIGHT_BRACKET),
        Key::KeyA => Ok(keys::KEY_A),
        Key::KeyS => Ok(keys::KEY_S),
        Key::KeyD => Ok(keys::KEY_D),
        Key::KeyF => Ok(keys::KEY_F),
        Key::KeyG => Ok(keys::KEY_G),
        Key::KeyH => Ok(keys::KEY_H),
        Key::KeyJ => Ok(keys::KEY_J),
        Key::KeyK => Ok(keys::KEY_K),
        Key::KeyL => Ok(keys::KEY_L),
        Key::SemiColon => Ok(keys::SEMICOLON),
        Key::Quote => Ok(keys::QUOTE),
        Key::BackSlash => Ok(keys::BACKSLASH),
        Key::IntlBackslash => Err(226),
        Key::KeyZ => Ok(keys::KEY_Z),
        Key::KeyX => Ok(keys::KEY_X),
        Key::KeyC => Ok(keys::KEY_C),
        Key::KeyV => Ok(keys::KEY_V),
        Key::KeyB => Ok(keys::KEY_B),
        Key::KeyN => Ok(keys::KEY_N),
        Key::KeyM => Ok(keys::KEY_M),
        Key::Comma => Ok(keys::COMMA),
        Key::Dot => Ok(keys::DOT),
        Key::Slash => Ok(keys::SLASH),
        Key::Insert => Ok(keys::INSERT),
        Key::KpReturn => Ok(keys::KP_RETURN),
        Key::KpMinus => Ok(keys::KP_MINUS),
        Key::KpPlus => Ok(keys::KP_PLUS),
        Key::KpMultiply => Ok(keys::KP_MULTIPLY),
        Key::KpDivide => Ok(keys::KP_DIVIDE),
        Key::Kp0 => Ok(keys::KP_0),
        Key::Kp1 => Ok(keys::KP_1),
        Key::Kp2 => Ok(keys::KP_2),
        Key::Kp3 => Ok(keys::KP_3),
        Key::Kp4 => Ok(keys::KP_4),
        Key::Kp5 => Ok(keys::KP_5),
        Key::Kp6 => Ok(keys::KP_6),
        Key::Kp7 => Ok(keys::KP_7),
        Key::Kp8 => Ok(keys::KP_8),
        Key::Kp9 => Ok(keys::KP_9),
        Key::KpDelete => Ok(keys::KP_DELETE),
        Key::Function => Ok(keys::FUNCTION),
        Key::Unknown(keycode) => Err(keycode),
    }
}
