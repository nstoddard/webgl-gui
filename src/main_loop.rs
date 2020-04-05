use cgmath::*;
use collect_mac::*;
use fnv::*;
use log::*;
use std::cell::RefCell;
use std::ops::*;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_stopwatch::*;
use web_sys::{window, KeyboardEvent, MouseEvent, WheelEvent};

use crate::event::*;

pub struct EventState {
    /// Contains all keys that are currently pressed.
    /// Note that this contains keycodes (`event.code`), not `event.key` values.
    pub pressed_keys: FnvHashSet<Keycode>,
    /// All mouse buttons that are currently pressed.
    pub pressed_mouse_buttons: FnvHashSet<MouseButton>,
    /// The current position of the cursor, if it's within the canvas.
    pub cursor_pos: Option<Point2<i32>>,
    /// The position of the cursor before the last mouse movement event.
    pub prev_cursor_pos: Option<Point2<i32>>,
    /// True if a pointer lock is active (through the pointer lock API).
    pub pointer_locked: bool,
}

/// The callback will be called every time an event occurs. This function is called by
/// `start_main_loop` so if that function is called, this function shouldn't be called.
///
/// This should typically be used by applications for which the `App` trait isn't suitable, such
/// as applications for which `request_animation_frame` isn't the best way to schedule rendering.
///
/// Returns a reference to the `EventState`, though this should never be modified, only read from.
pub fn setup_event_callbacks(
    canvas_id: &str,
    callback: Box<dyn Fn(Event, &EventState)>,
) -> Rc<RefCell<EventState>> {
    let event_state = Rc::new(RefCell::new(EventState {
        pressed_keys: collect![],
        pressed_mouse_buttons: collect![],
        cursor_pos: None,
        prev_cursor_pos: None,
        pointer_locked: false,
    }));
    let event_state2 = event_state.clone();
    let event_state3 = event_state.clone();
    let event_state4 = event_state.clone();

    let callback = Rc::new(RefCell::new(move |event: Event| {
        let mut event_state = event_state.borrow_mut();
        match event {
            Event::KeyDown(ref key) => {
                event_state.pressed_keys.insert(key.code.clone());
            }
            Event::KeyUp(ref key) => {
                event_state.pressed_keys.remove(&key.code);
            }
            Event::FocusLost => {
                event_state.pressed_keys.clear();
                event_state.pressed_mouse_buttons.clear();
            }
            Event::MouseDown(button, _) => {
                event_state.pressed_mouse_buttons.insert(button);
            }
            Event::MouseUp(button, _) => {
                event_state.pressed_mouse_buttons.remove(&button);
            }
            Event::MouseLeave => {
                event_state.pressed_mouse_buttons.clear();
            }
            Event::PointerLocked => {
                event_state.pointer_locked = true;
            }
            Event::PointerUnlocked => {
                event_state.pointer_locked = false;
            }
            _ => (),
        }
        callback(event, &event_state);
    }));
    // A clone of this is needed for each event handler.
    let callback2 = callback.clone();
    let callback3 = callback.clone();
    let callback4 = callback.clone();
    let callback5 = callback.clone();
    let callback6 = callback.clone();
    let callback7 = callback.clone();
    let callback8 = callback.clone();
    let callback9 = callback.clone();
    let callback10 = callback.clone();
    let callback11 = callback.clone();
    let callback12 = callback.clone();

    let window = window().unwrap();
    let document = window.document().unwrap();
    let document2 = document.clone();
    let canvas = document.get_element_by_id(canvas_id).unwrap();

    let keydown_handler = Closure::wrap(Box::new(move |e: KeyboardEvent| {
        let key = Key::from_js(&e);
        callback.borrow_mut().deref_mut()(Event::KeyDown(key))
    }) as Box<dyn FnMut(KeyboardEvent)>);
    document
        .add_event_listener_with_callback("keydown", keydown_handler.as_ref().unchecked_ref())
        .unwrap();
    keydown_handler.forget();

    let keyup_handler = Closure::wrap(Box::new(move |e: KeyboardEvent| {
        callback2.borrow_mut().deref_mut()(Event::KeyUp(Key::from_js(&e)))
    }) as Box<dyn FnMut(KeyboardEvent)>);
    document
        .add_event_listener_with_callback("keyup", keyup_handler.as_ref().unchecked_ref())
        .unwrap();
    keyup_handler.forget();

    let focus_handler =
        Closure::wrap(Box::new(move || callback3.borrow_mut().deref_mut()(Event::FocusGained))
            as Box<dyn FnMut()>);
    document
        .add_event_listener_with_callback("focus", focus_handler.as_ref().unchecked_ref())
        .unwrap();
    focus_handler.forget();

    let blur_handler =
        Closure::wrap(Box::new(move || callback4.borrow_mut().deref_mut()(Event::FocusLost))
            as Box<dyn FnMut()>);
    document
        .add_event_listener_with_callback("blur", blur_handler.as_ref().unchecked_ref())
        .unwrap();
    blur_handler.forget();

    let mousedown_handler = Closure::wrap(Box::new(move |e: MouseEvent| {
        if let Some(event) = mouse_down_event_from_js(e) {
            callback5.borrow_mut().deref_mut()(event);
        } else {
            warn!("Invalid mouse event");
        }
    }) as Box<dyn FnMut(MouseEvent)>);
    canvas
        .add_event_listener_with_callback("mousedown", mousedown_handler.as_ref().unchecked_ref())
        .unwrap();
    mousedown_handler.forget();

    let mouseup_handler = Closure::wrap(Box::new(move |e: MouseEvent| {
        if let Some(event) = mouse_up_event_from_js(e) {
            callback6.borrow_mut().deref_mut()(event);
        } else {
            warn!("Invalid mouse event");
        }
    }) as Box<dyn FnMut(MouseEvent)>);
    canvas
        .add_event_listener_with_callback("mouseup", mouseup_handler.as_ref().unchecked_ref())
        .unwrap();
    mouseup_handler.forget();

    let mousemove_handler = Closure::wrap(Box::new(move |e: MouseEvent| {
        if let Some(event) = mouse_move_event_from_js(e) {
            if let Event::MouseMove { pos, .. } = &event {
                let mut event_state = event_state2.borrow_mut();
                event_state.prev_cursor_pos = event_state.cursor_pos;
                event_state.cursor_pos = Some(*pos);
            } else {
                panic!();
            }
            callback7.borrow_mut().deref_mut()(event);
        } else {
            warn!("Invalid mouse event");
        }
    }) as Box<dyn FnMut(MouseEvent)>);
    canvas
        .add_event_listener_with_callback("mousemove", mousemove_handler.as_ref().unchecked_ref())
        .unwrap();
    mousemove_handler.forget();

    let mouseenter_handler = Closure::wrap(Box::new(move |_e: MouseEvent| {
        callback8.borrow_mut().deref_mut()(Event::MouseEnter);
    }) as Box<dyn FnMut(MouseEvent)>);
    canvas
        .add_event_listener_with_callback("mouseenter", mouseenter_handler.as_ref().unchecked_ref())
        .unwrap();
    mouseenter_handler.forget();

    let mouseleave_handler = Closure::wrap(Box::new(move |_e: MouseEvent| {
        event_state3.borrow_mut().cursor_pos = None;
        (&mut callback9.borrow_mut())(Event::MouseLeave);
    }) as Box<dyn FnMut(MouseEvent)>);
    canvas
        .add_event_listener_with_callback("mouseleave", mouseleave_handler.as_ref().unchecked_ref())
        .unwrap();
    mouseleave_handler.forget();

    let resize_handler = Closure::wrap(Box::new(move || {
        (&mut callback10.borrow_mut())(Event::WindowResized(get_window_size()));
    }) as Box<dyn FnMut()>);
    window
        .add_event_listener_with_callback("resize", resize_handler.as_ref().unchecked_ref())
        .unwrap();
    resize_handler.forget();

    let pointer_lock_change_handler = Closure::wrap(Box::new(move || {
        (&mut callback11.borrow_mut())(if document2.pointer_lock_element().is_some() {
            Event::PointerLocked
        } else {
            Event::PointerUnlocked
        });
    }) as Box<dyn FnMut()>);
    document
        .add_event_listener_with_callback(
            "pointerlockchange",
            pointer_lock_change_handler.as_ref().unchecked_ref(),
        )
        .unwrap();
    pointer_lock_change_handler.forget();

    let wheel_handler = Closure::wrap(Box::new(move |e: WheelEvent| {
        // Different browsers have different behavior for the "wheel" event, so restrict it to either -1 or 1.
        // TODO: is there a better solution?
        callback12.borrow_mut().deref_mut()(Event::Scroll(e.delta_y().signum()));
    }) as Box<dyn FnMut(WheelEvent)>);
    canvas
        .add_event_listener_with_callback("wheel", wheel_handler.as_ref().unchecked_ref())
        .unwrap();
    wheel_handler.forget();

    event_state4
}

/// An app that renders to a WebGL canvas.
pub trait App {
    /// Called every time an event occurs. Apps may handle events here, or in `render_frame`.
    fn handle_event(&mut self, _event: Event) {}

    /// Called every time a frame should be rendered; uses `requestAnimationFrame`.
    ///
    /// `events` contains all events that have occurred since the last call to this function.
    fn render_frame(
        &mut self,
        events: Vec<Event>,
        event_state: &EventState,
        // How much time has passed since the last call to render_frame, in seconds.
        dt: f64,
    );

    /// Called when the web page is being closed.
    fn on_close(&mut self) {}
}

/// Starts a main loop for a WebGL app. `request_animation_frame` is used to schedule rendering.
///
/// `canvas_id` should be the ID of the canvas the app is rendering to. All mouse event positions
/// are relative to the top-left corner of this canvas.
///
/// `app` will never be dropped. The `on_close` method can be used as an alternative.
pub fn start_main_loop(canvas_id: &str, app: Box<dyn App>) {
    let queued_events = Rc::new(RefCell::new(vec![]));
    let queued_events2 = queued_events.clone();

    let app = Rc::new(RefCell::new(app));
    let app2 = app.clone();
    let app3 = app.clone();

    let mut stopwatch = Stopwatch::new();

    let callback = move |event: Event, _: &EventState| {
        app.borrow_mut().handle_event(event.clone());
        queued_events.borrow_mut().push(event);
    };
    let event_state = setup_event_callbacks(canvas_id, Box::new(callback));

    let window = window().unwrap();

    let close_handler = Closure::wrap(Box::new(move || {
        app2.borrow_mut().on_close();
    }) as Box<dyn FnMut()>);
    window.set_onbeforeunload(Some(close_handler.as_ref().unchecked_ref()));
    close_handler.forget();

    let closure: Rc<RefCell<Option<Closure<_>>>> = Rc::new(RefCell::new(None));
    let closure2 = closure.clone();
    *closure.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let mut queued_events = queued_events2.borrow_mut();
        let event_state = event_state.borrow_mut();
        let events = std::mem::replace(&mut *queued_events, vec![]);
        let dt = stopwatch.get_time();
        stopwatch.reset();
        app3.borrow_mut().render_frame(events, &event_state, dt);

        web_sys::window()
            .unwrap()
            .request_animation_frame(closure2.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }) as Box<dyn FnMut()>));

    window
        .request_animation_frame(closure.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();
}
