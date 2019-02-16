use cgmath::*;
use collect_mac::*;
use fnv::*;
use std::mem;
use uid::*;
use webgl_wrapper::*;

use crate::color::*;
use crate::draw_2d::*;
use crate::event::*;
use crate::text::*;

#[doc(hidden)]
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct WidgetId_(());

pub type WidgetId = Id<WidgetId_>;

// TODO: components should be passed a keyboard event, and *then* decide whether to handle it. If not, it's passed to the next component. If no component handles an event, it's sent to the rest of the program.

/// Controls the appearance of the GUI.
pub struct Theme {
    pub font: Font,
    pub label_color: Color4,
    pub button_text_color: Color4,
    pub button_fill_color: Color4,
    pub button_border_color: Color4,
    pub button_selected_fill_color: Color4,
    pub button_active_fill_color: Color4,
    pub padding: i32,
}

/// Components store persistent data about a widget or group of widgets. They
/// are typically used for widgets that provide user input.
pub trait Component: Widget {
    /// The result of updating the `Component`. Typically contains output events; for instance,
    /// a button component's `Res` type might describe whether the button was pressed.
    type Res;

    /// Updates the component's internal state and returns a result. This shouldn't be called from
    /// outside of `webgl-gui`.
    fn update(&mut self, events: Vec<Event>) -> Self::Res;
}

/// Something that can be drawn as part of the GUI.
pub trait Widget {
    /// Each widget must have a unique ID.
    fn id(&self) -> WidgetId;

    /// This must return true iff the widget is the root widget of a component.
    ///
    /// It is undefined behavior if there's a component within another
    /// component. In the current implementation, the outer component will
    /// receive the event, but it's not guaranteed.
    fn is_component(&self) -> bool {
        false
    }

    /// Does *not* need to draw its children. Its children will be automatically drawn after
    /// this widget.
    fn draw(
        &self,
        context: &GlContext,
        rect: Rect<i32>,
        theme: &Theme,
        draw_2d: &mut Draw2d,
        cursor_pos: Option<Point2<f64>>,
        is_active: bool,
    );

    /// By the time this is called, min_sizes will contain the min size of each
    /// child.
    fn min_size(
        &self,
        context: &GlContext,
        theme: &Theme,
        min_sizes: &FnvHashMap<WidgetId, Vector2<i32>>,
        window_size: Vector2<i32>,
    ) -> Vector2<i32>;

    /// Returns a reference to each child.
    fn children(&self) -> Vec<&dyn Widget> {
        vec![]
    }

    /// This must add the widget's Rect and call itself recursively for each
    /// child. It must be overridden if the widget has any children.
    fn compute_rects(
        &self,
        rect: Rect<i32>,
        _theme: &Theme,
        _min_sizes: &FnvHashMap<WidgetId, Vector2<i32>>,
        widget_rects: &mut FnvHashMap<WidgetId, Rect<i32>>,
    ) {
        widget_rects.insert(self.id(), rect);
    }
}

fn compute_widget_min_size(
    widget: &dyn Widget,
    context: &GlContext,
    theme: &Theme,
    min_sizes: &mut FnvHashMap<WidgetId, Vector2<i32>>,
    window_size: Vector2<i32>,
) {
    for child in widget.children() {
        compute_widget_min_size(child, context, theme, min_sizes, window_size);
    }
    let min_size = widget.min_size(context, theme, min_sizes, window_size);
    min_sizes.insert(widget.id(), min_size); //Vec2(min_size.x.min(window_size.x), min_size.y.min(window_size.y)));
}

fn widget_handle_event(
    widget: &dyn Widget,
    event: &Event,
    widget_rects: &FnvHashMap<WidgetId, Rect<i32>>,
    events_out: &mut FnvHashMap<WidgetId, Vec<Event>>,
    active_component_id: Option<WidgetId>,
) -> bool {
    if widget.is_component() {
        let rect = widget_rects[&widget.id()];
        let is_active = active_component_id == Some(widget.id());

        let event2 = match event {
            Event::KeyDown(_) => {
                if is_active {
                    Some(event)
                } else {
                    None
                }
            }
            Event::KeyUp(_) => {
                if is_active {
                    Some(event)
                } else {
                    None
                }
            }
            Event::MouseDown(_, pos) => {
                if rect.contains_point(*pos) {
                    Some(event)
                } else {
                    None
                }
            }
            Event::MouseUp(_, pos) => {
                if rect.contains_point(*pos) {
                    Some(event)
                } else {
                    None
                }
            }
            Event::MouseMove(pos) => {
                if rect.contains_point(*pos) {
                    Some(event)
                } else {
                    None
                }
            }
            Event::MouseEnter => None,
            Event::MouseLeave => None,
            Event::FocusGained => Some(event),
            Event::FocusLost => Some(event),
        };
        if let Some(event2) = event2 {
            let events = events_out.entry(widget.id()).or_insert(vec![]);
            events.push(event2.clone());
            return true;
        }
    }
    for child in widget.children() {
        if widget_handle_event(child, event, widget_rects, events_out, active_component_id) {
            return true;
        }
    }
    false
}

fn draw_widget(
    widget: &dyn Widget,
    context: &GlContext,
    theme: &Theme,
    draw_2d: &mut Draw2d,
    widget_rects: &FnvHashMap<WidgetId, Rect<i32>>,
    cursor_pos: Option<Point2<f64>>,
    active_widget_id: Option<WidgetId>,
) {
    let rect = widget_rects[&widget.id()];
    let is_active = active_widget_id == Some(widget.id());
    widget.draw(context, rect, theme, draw_2d, cursor_pos, is_active);
    for child in widget.children() {
        draw_widget(child, context, theme, draw_2d, widget_rects, cursor_pos, active_widget_id);
    }
}

pub struct GuiResult {
    rendered_size: Vector2<i32>,
}

pub struct GuiEventResult {
    /// Events to be handled by each component
    component_events: FnvHashMap<WidgetId, Vec<Event>>,
    /// Events not handled by any component
    unhandled_events: Vec<Event>,
}

impl GuiResult {
    /// The actual rendered size of the GUI.
    pub fn rendered_size(&self) -> Vector2<i32> {
        self.rendered_size
    }
}

impl GuiEventResult {
    /// Updates the given `Component` with any events that apply to it.
    pub fn update_component<C: Component>(&mut self, component: &mut Box<C>) -> C::Res {
        let events = self.component_events.remove(&component.id()).unwrap_or_else(|| vec![]);
        component.update(events)
    }

    /// Returns all events that weren't handled by any `Component`.
    pub fn unhandled_events(&mut self) -> Vec<Event> {
        mem::replace(&mut self.unhandled_events, vec![])
    }
}

pub struct Gui {
    // None if there are no components
    // The Id is that of the component
    active_component: Option<(i32, WidgetId)>,
    last_render: Option<RenderedGui>,
}

struct RenderedGui {
    widget: Box<dyn Widget>,
    widget_rects: FnvHashMap<WidgetId, Rect<i32>>,
}

impl Gui {
    pub fn new() -> Self {
        Self { active_component: None, last_render: None }
    }

    /// Draws the GUI.
    pub fn draw(
        &mut self,
        context: &GlContext,
        surface: &impl Surface,
        theme: &Theme,
        draw_2d: &mut Draw2d,
        cursor_pos: Option<Point2<f64>>,
        widget: Box<dyn Widget>,
    ) -> GuiResult {
        println!("Computing widget rects");
        let mut min_sizes = collect![];
        let mut widget_rects = collect![];
        compute_widget_min_size(
            &*widget,
            context,
            theme,
            &mut min_sizes,
            surface.size().cast().unwrap(),
        );
        let rect = Rect::new(Point2::origin(), Point2::from_vec(surface.size().cast().unwrap()));
        widget.compute_rects(rect, theme, &min_sizes, &mut widget_rects);

        let active_component_id = self.active_component.map(|(_a, b)| b);
        println!("Drawing main widget");
        draw_widget(
            &*widget,
            context,
            theme,
            draw_2d,
            &widget_rects,
            cursor_pos,
            active_component_id,
        );

        let res = GuiResult { rendered_size: widget_rects[&widget.id()].size() };

        println!("Setting last_render");
        self.last_render = Some(RenderedGui { widget, widget_rects });

        println!("Done drawing GUI");

        res
    }

    /// Handles events by applying them to the most recently rendered output.
    /// The ordered_components must use the same IDs as the ones passed into
    /// the last call to GUI.draw().
    // TODO: is keyboard_navigates redundant with active_component_id being Some?
    // TODO: events: Vec<Event>
    pub fn handle_events(
        &mut self,
        events: &[Event],
        // keyboard_navigates: bool,
        ordered_components: &[WidgetId],
    ) -> GuiEventResult {
        if let Some(RenderedGui { widget, widget_rects }) = &self.last_render {
            let mut events_out = collect![];
            let mut unhandled_events = vec![];
            let active_component_id = self.active_component.map(|(_a, b)| b);

            if true//keyboard_navigates
                && !ordered_components.is_empty()
                && self.active_component.is_none()
            {
                self.active_component = Some((0, ordered_components[0]));
            }

            for event in events {
                if widget_handle_event(
                    &**widget,
                    &event,
                    &widget_rects,
                    &mut events_out,
                    active_component_id,
                ) {
                    continue;
                }
                if true {
                    //keyboard_navigates {
                    if let Some((ref mut active_component_index, ref mut active_component_id)) =
                        &mut self.active_component
                    {
                        match event {
                            Event::KeyDown(key) => {
                                if key.key == "ArrowDown"
                                    || key.key == "ArrowRight"
                                    || (key.key == "Tab" && !key.shift)
                                {
                                    *active_component_index = (*active_component_index + 1)
                                        % (ordered_components.len() as i32);
                                    *active_component_id =
                                        ordered_components[*active_component_index as usize];
                                    continue;
                                } else if key.key == "ArrowUp"
                                    || key.key == "ArrowLeft"
                                    || (key.key == "Tab" && key.shift)
                                {
                                    // Workaround for mod_euc not yet being stable
                                    *active_component_index = (*active_component_index - 1
                                        + ordered_components.len() as i32)
                                        % (ordered_components.len() as i32);
                                    *active_component_id =
                                        ordered_components[*active_component_index as usize];
                                    continue;
                                }
                            }
                            _ => (),
                        }
                    } else {
                        panic!();
                    }
                }
                unhandled_events.push(event.clone());
            }

            GuiEventResult { component_events: events_out, unhandled_events }
        } else {
            GuiEventResult { component_events: collect![], unhandled_events: events.to_vec() }
        }
    }
}
