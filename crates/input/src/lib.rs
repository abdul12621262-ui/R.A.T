//! Platform input injection: SendInput (Windows), CGEvent (macOS), enigo (Linux).

use anyhow::Result;
use rat_protocol::{ControlAction, MouseButton};

pub trait InputInjector: Send {
    fn apply(&mut self, action: &ControlAction) -> Result<()>;
}

pub fn create_injector() -> Result<Box<dyn InputInjector>> {
    #[cfg(windows)]
    return Ok(Box::new(WindowsInjector::new()?));
    #[cfg(target_os = "macos")]
    return Ok(Box::new(MacInjector::new()?));
    #[cfg(target_os = "linux")]
    return Ok(Box::new(LinuxInjector::new()?));
}

#[cfg(windows)]
pub struct WindowsInjector {
    enigo: enigo::Enigo,
}

#[cfg(windows)]
impl WindowsInjector {
    pub fn new() -> Result<Self> {
        use enigo::{Enigo, Settings};
        Ok(Self {
            enigo: Enigo::new(&Settings::default())?,
        })
    }
}

#[cfg(windows)]
impl InputInjector for WindowsInjector {
    fn apply(&mut self, action: &ControlAction) -> Result<()> {
        use enigo::{Button, Coordinate, Direction, Key, Keyboard, Mouse};
        match action {
            ControlAction::MouseMove { x, y } => {
                self.enigo.move_mouse(*x, *y, Coordinate::Abs)?;
            }
            ControlAction::MouseClick {
                button,
                double_click,
            } => {
                let btn = match button {
                    MouseButton::Left => Button::Left,
                    MouseButton::Right => Button::Right,
                    MouseButton::Middle => Button::Middle,
                };
                self.enigo.button(btn, Direction::Click)?;
                if *double_click {
                    self.enigo.button(btn, Direction::Click)?;
                }
            }
            ControlAction::TypeText { text } => {
                self.enigo.text(text)?;
            }
            ControlAction::PressKey { key } => {
                let k = match key.to_lowercase().as_str() {
                    "enter" => Key::Return,
                    "backspace" => Key::Backspace,
                    "escape" => Key::Escape,
                    "tab" => Key::Tab,
                    "space" => Key::Space,
                    "up" => Key::UpArrow,
                    "down" => Key::DownArrow,
                    "left" => Key::LeftArrow,
                    "right" => Key::RightArrow,
                    "delete" => Key::Delete,
                    other if other.len() == 1 => {
                        let ch = other.chars().next().unwrap();
                        self.enigo.text(&ch.to_string())?;
                        return Ok(());
                    }
                    _ => return Ok(()),
                };
                self.enigo.key(k, Direction::Click)?;
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(target_os = "macos")]
pub struct MacInjector;

#[cfg(target_os = "macos")]
impl MacInjector {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

#[cfg(target_os = "macos")]
impl InputInjector for MacInjector {
    fn apply(&mut self, action: &ControlAction) -> Result<()> {
        use core_graphics::event::{
            CGEvent, CGEventTapLocation, CGEventType, CGKeyCode, CGMouseButton,
        };
        match action {
            ControlAction::MouseMove { x, y } => {
                let evt = CGEvent::new_mouse_event(
                    CGEventType::MouseMoved,
                    (*x as f64, *y as f64),
                    CGMouseButton::Left,
                )?;
                evt.post(CGEventTapLocation::HID);
            }
            ControlAction::MouseClick {
                button,
                double_click,
            } => {
                let btn = match button {
                    MouseButton::Left => CGMouseButton::Left,
                    MouseButton::Right => CGMouseButton::Right,
                    MouseButton::Middle => CGMouseButton::Center,
                };
                let down =
                    CGEvent::new_mouse_event(CGEventType::LeftMouseDown, (0.0, 0.0), btn)?;
                down.post(CGEventTapLocation::HID);
                let up = CGEvent::new_mouse_event(CGEventType::LeftMouseUp, (0.0, 0.0), btn)?;
                up.post(CGEventTapLocation::HID);
                if *double_click {
                    down.post(CGEventTapLocation::HID);
                    up.post(CGEventTapLocation::HID);
                }
            }
            ControlAction::TypeText { text } => {
                for ch in text.chars() {
                    let evt = CGEvent::new_keyboard_event(CGEventType::KeyDown, 0, true)?;
                    evt.set_string(&ch.to_string());
                    evt.post(CGEventTapLocation::HID);
                }
            }
            ControlAction::PressKey { key } => {
                let code: CGKeyCode = match key.to_lowercase().as_str() {
                    "enter" => 36,
                    "escape" => 53,
                    "backspace" => 51,
                    _ => 0,
                };
                let evt = CGEvent::new_keyboard_event(CGEventType::KeyDown, code, true)?;
                evt.post(CGEventTapLocation::HID);
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
pub struct LinuxInjector {
    enigo: enigo::Enigo,
}

#[cfg(target_os = "linux")]
impl LinuxInjector {
    pub fn new() -> Result<Self> {
        use enigo::{Enigo, Settings};
        Ok(Self {
            enigo: Enigo::new(&Settings::default())?,
        })
    }
}

#[cfg(target_os = "linux")]
impl InputInjector for LinuxInjector {
    fn apply(&mut self, action: &ControlAction) -> Result<()> {
        use enigo::{Button, Coordinate, Direction, Key, Keyboard, Mouse};
        match action {
            ControlAction::MouseMove { x, y } => {
                self.enigo.move_mouse(*x, *y, Coordinate::Abs)?;
            }
            ControlAction::MouseClick {
                button,
                double_click,
            } => {
                let btn = match button {
                    MouseButton::Left => Button::Left,
                    MouseButton::Right => Button::Right,
                    MouseButton::Middle => Button::Middle,
                };
                self.enigo.button(btn, Direction::Click)?;
                if *double_click {
                    self.enigo.button(btn, Direction::Click)?;
                }
            }
            ControlAction::TypeText { text } => {
                self.enigo.text(text)?;
            }
            ControlAction::PressKey { key } => {
                let k = match key.to_lowercase().as_str() {
                    "enter" => Key::Return,
                    "backspace" => Key::Backspace,
                    "escape" => Key::Escape,
                    _ => return Ok(()),
                };
                self.enigo.key(k, Direction::Click)?;
            }
            _ => {}
        }
        Ok(())
    }
}
