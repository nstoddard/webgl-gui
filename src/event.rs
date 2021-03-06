use cgmath::*;
use web_sys::{window, KeyboardEvent, MouseEvent};

// TODO: can Clone be removed for these types?
/// An event.
#[derive(Clone, Debug)]
pub enum Event {
    KeyDown(Key),
    KeyUp(Key),
    MouseDown(MouseButton, Point2<i32>),
    MouseUp(MouseButton, Point2<i32>),
    MouseMove { pos: Point2<i32>, movement: Vector2<i32> },
    MouseEnter,
    MouseLeave,
    FocusGained,
    FocusLost,
    WindowResized(Vector2<u32>),
    PointerLocked,
    PointerUnlocked,
    Scroll(f64),
}

pub type Keycode = String;

/// A key.
#[derive(Clone, Debug)]
pub struct Key {
    /// These correspond to `event.key` values. In most cases these are the same as the ASCII
    /// character the key represents. In other cases, see
    /// [this page](https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/key/Key_Values).
    pub key: String,
    /// These correspond to `event.code` values.
    pub code: Keycode,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

// TODO: add more
const MODIFIERS: &[&str] = &["Alt", "Control", "Shift"];

impl Key {
    pub(crate) fn from_js(js_key: &KeyboardEvent) -> Self {
        Self {
            key: js_key.key(),
            code: js_key.code(),
            shift: js_key.shift_key(),
            ctrl: js_key.ctrl_key(),
            alt: js_key.alt_key(),
        }
    }

    pub fn is_modifier(&self) -> bool {
        MODIFIERS.contains(&self.key.as_ref())
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
}

impl MouseButton {
    pub(crate) fn from_js(js_button: i16) -> Option<Self> {
        match js_button {
            0 => Some(MouseButton::Left),
            1 => Some(MouseButton::Middle),
            2 => Some(MouseButton::Right),
            3 => Some(MouseButton::Back),
            4 => Some(MouseButton::Forward),
            _ => None,
        }
    }
}

fn mouse_pos_from_js(event: MouseEvent) -> Point2<i32> {
    point2(event.offset_x(), event.offset_y())
}

pub(crate) fn mouse_down_event_from_js(event: MouseEvent) -> Option<Event> {
    let button = MouseButton::from_js(event.button())?;
    Some(Event::MouseDown(button, mouse_pos_from_js(event)))
}

pub(crate) fn mouse_up_event_from_js(event: MouseEvent) -> Option<Event> {
    let button = MouseButton::from_js(event.button())?;
    Some(Event::MouseUp(button, mouse_pos_from_js(event)))
}

pub(crate) fn mouse_move_event_from_js(event: MouseEvent) -> Option<Event> {
    Some(Event::MouseMove {
        movement: vec2(event.movement_x(), event.movement_y()),
        pos: mouse_pos_from_js(event),
    })
}

pub fn get_window_size() -> Vector2<u32> {
    let window = window().unwrap();
    vec2(
        window.inner_width().unwrap().as_f64().unwrap() as u32,
        window.inner_height().unwrap().as_f64().unwrap() as u32,
    )
}
