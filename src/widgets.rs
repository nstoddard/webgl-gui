use cgmath::*;
use fnv::*;
use std::rc::Rc;
use webgl_wrapper::*;

use crate::color::*;
use crate::draw_2d::*;
use crate::event::*;
use crate::gui::*;

pub struct Label {
    id: WidgetId,
    text: String,
}

impl Label {
    pub fn new(text: &str) -> Box<Self> {
        Box::new(Label { id: WidgetId::new(), text: text.to_string() })
    }
}

impl Widget for Label {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn draw(
        &self,
        context: &GlContext,
        rect: Rect<i32>,
        theme: &Theme,
        _draw_2d: &mut Draw2d,
        _cursor_pos: Option<Point2<f64>>,
        _is_active: bool,
    ) {
        theme.font.draw_string(context, &self.text, rect.start, theme.label_color);
    }

    fn min_size(
        &self,
        context: &GlContext,
        theme: &Theme,
        _min_sizes: &FnvHashMap<WidgetId, Vector2<i32>>,
        _window_size: Vector2<i32>,
    ) -> Vector2<i32> {
        theme.font.string_size(context, &self.text)
    }
}

pub struct ButtonResult {
    pressed: bool,
}

impl ButtonResult {
    pub fn pressed(&self) -> bool {
        self.pressed
    }
}

#[derive(Clone)]
pub struct Button {
    id: WidgetId,
    text: String,
}

impl Button {
    pub fn new(text: &str) -> Box<Self> {
        let id = WidgetId::new();
        Box::new(Button { id, text: text.to_string() })
    }
}

impl Component for Button {
    type Res = ButtonResult;

    fn update(&mut self, events: Vec<Event>) -> ButtonResult {
        let mut pressed = false;
        for event in events {
            match event {
                Event::MouseDown(MouseButton::Left, _) => {
                    pressed = true;
                    break;
                }
                Event::KeyDown(key) => {
                    if key.key == "Enter" || key.key == " " {
                        pressed = true;
                        break;
                    }
                }
                _ => (),
            }
        }

        ButtonResult { pressed }
    }
}

impl Widget for Button {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn is_component(&self) -> bool {
        true
    }

    fn draw(
        &self,
        context: &GlContext,
        rect: Rect<i32>,
        theme: &Theme,
        draw_2d: &mut Draw2d,
        cursor_pos: Option<Point2<f64>>,
        is_active: bool,
    ) {
        let fill_color =
            if cursor_pos.is_some() && rect.contains_point(cursor_pos.unwrap().cast().unwrap()) {
                theme.button_selected_fill_color
            } else if is_active {
                theme.button_active_fill_color
            } else {
                theme.button_fill_color
            };
        draw_2d.fill_rect(rect, fill_color);
        draw_2d.outline_rect(rect, theme.button_border_color, 1.0);
        theme.font.draw_string(
            context,
            &self.text,
            rect.start + vec2(2, 1),
            theme.button_text_color,
        );
    }

    fn min_size(
        &self,
        context: &GlContext,
        theme: &Theme,
        _min_sizes: &FnvHashMap<WidgetId, Vector2<i32>>,
        _window_size: Vector2<i32>,
    ) -> Vector2<i32> {
        theme.font.string_size(context, &self.text) + vec2(4, 2)
    }
}

/// A widget that makes its child its minimum possible size rather than filling the whole
/// window.
pub struct NoFill {
    id: WidgetId,
    child: Box<dyn Widget>,
}

impl NoFill {
    pub fn new(child: Box<dyn Widget>) -> Box<Self> {
        Box::new(NoFill { id: WidgetId::new(), child })
    }
}

impl Widget for NoFill {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn draw(
        &self,
        _context: &GlContext,
        _rect: Rect<i32>,
        _theme: &Theme,
        _draw_2d: &mut Draw2d,
        _cursor_pos: Option<Point2<f64>>,
        _is_active: bool,
    ) {
    }

    fn min_size(
        &self,
        _context: &GlContext,
        _theme: &Theme,
        min_sizes: &FnvHashMap<WidgetId, Vector2<i32>>,
        _window_size: Vector2<i32>,
    ) -> Vector2<i32> {
        min_sizes[&self.child.id()]
    }

    fn children(&self) -> Vec<&dyn Widget> {
        vec![&*self.child]
    }

    fn compute_rects(
        &self,
        rect: Rect<i32>,
        theme: &Theme,
        min_sizes: &FnvHashMap<WidgetId, Vector2<i32>>,
        widget_rects: &mut FnvHashMap<WidgetId, Rect<i32>>,
    ) {
        let min_size = min_sizes[&self.id()];
        widget_rects.insert(self.id(), Rect::new(rect.start, rect.start + min_size));
        self.child.compute_rects(
            Rect::new(rect.start, rect.start + min_size),
            theme,
            min_sizes,
            widget_rects,
        );
    }
}

pub struct Col {
    id: WidgetId,
    children: Vec<(Box<dyn Widget>, f64)>,
}

impl Col {
    pub fn new() -> Box<Self> {
        Box::new(Col { id: WidgetId::new(), children: vec![] })
    }

    /// Flex controls how to distribute unused space.
    pub fn child(mut self: Box<Self>, flex: f64, child: Box<dyn Widget>) -> Box<Self> {
        self.children.push((child, flex));
        self
    }

    pub fn children(mut self: Box<Self>, children: Vec<(f64, Box<dyn Widget>)>) -> Box<Self> {
        self.children.extend(children.into_iter().map(|(a, b)| (b, a)));
        self
    }
}

impl Widget for Col {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn draw(
        &self,
        _context: &GlContext,
        _rect: Rect<i32>,
        _theme: &Theme,
        _draw_2d: &mut Draw2d,
        _cursor_pos: Option<Point2<f64>>,
        _is_active: bool,
    ) {
    }

    fn min_size(
        &self,
        _context: &GlContext,
        _theme: &Theme,
        min_sizes: &FnvHashMap<WidgetId, Vector2<i32>>,
        _window_size: Vector2<i32>,
    ) -> Vector2<i32> {
        let mut min_size: Vector2<i32> = Vector2::zero();
        for &(ref child, _flex) in &self.children {
            let child_min_size = min_sizes[&child.id()];
            min_size.x = min_size.x.max(child_min_size.x);
            min_size.y += child_min_size.y;
        }
        min_size
    }

    fn children(&self) -> Vec<&dyn Widget> {
        self.children.iter().map(|(child, _)| &**child as &dyn Widget).collect()
    }

    fn compute_rects(
        &self,
        rect: Rect<i32>,
        theme: &Theme,
        min_sizes: &FnvHashMap<WidgetId, Vector2<i32>>,
        widget_rects: &mut FnvHashMap<WidgetId, Rect<i32>>,
    ) {
        let total_flex = self.children.iter().map(|&(ref _child, flex)| flex).sum();
        let min_size = min_sizes[&self.id()];
        let own_rect = if total_flex == 0.0 {
            Rect::new(rect.start, rect.start + vec2(rect.size().x, min_size.y))
        } else {
            Rect::new(rect.start, rect.start + vec2(rect.size().x, rect.size().y))
        };
        widget_rects.insert(self.id(), own_rect);
        let mut next_pos = rect.start;
        let total_flex = if total_flex == 0.0 { 1.0 } else { total_flex };
        let extra_space = rect.size().y - min_size.y;
        for &(ref child, flex) in &self.children {
            let child_min_size = min_sizes[&child.id()];
            let widget_extra_space = (extra_space as f64 * flex / total_flex) as i32;
            let widget_height = child_min_size.y + widget_extra_space;
            let widget_rect = Rect::new(next_pos, next_pos + vec2(rect.size().x, widget_height));
            next_pos.y += widget_height;
            child.compute_rects(widget_rect, theme, min_sizes, widget_rects);
        }
    }
}

pub struct Row {
    id: WidgetId,
    children: Vec<(Box<dyn Widget>, f64)>,
}

impl Row {
    pub fn new() -> Box<Self> {
        Box::new(Row { id: WidgetId::new(), children: vec![] })
    }

    /// Flex controls how to distribute unused space.
    pub fn child(mut self: Box<Self>, flex: f64, child: Box<dyn Widget>) -> Box<Self> {
        self.children.push((child, flex));
        self
    }

    pub fn children(mut self: Box<Self>, children: Vec<(f64, Box<dyn Widget>)>) -> Box<Self> {
        self.children.extend(children.into_iter().map(|(a, b)| (b, a)));
        self
    }
}

impl Widget for Row {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn draw(
        &self,
        _context: &GlContext,
        _rect: Rect<i32>,
        _theme: &Theme,
        _draw_2d: &mut Draw2d,
        _cursor_pos: Option<Point2<f64>>,
        _is_active: bool,
    ) {
    }

    fn min_size(
        &self,
        _context: &GlContext,
        _theme: &Theme,
        min_sizes: &FnvHashMap<WidgetId, Vector2<i32>>,
        _window_size: Vector2<i32>,
    ) -> Vector2<i32> {
        let mut min_size: Vector2<i32> = Vector2::zero();
        for &(ref child, _flex) in &self.children {
            let child_min_size = min_sizes[&child.id()];
            min_size.y = min_size.y.max(child_min_size.y);
            min_size.x += child_min_size.x;
        }
        min_size
    }

    fn children(&self) -> Vec<&dyn Widget> {
        self.children.iter().map(|(child, _)| &**child as &dyn Widget).collect()
    }

    fn compute_rects(
        &self,
        rect: Rect<i32>,
        theme: &Theme,
        min_sizes: &FnvHashMap<WidgetId, Vector2<i32>>,
        widget_rects: &mut FnvHashMap<WidgetId, Rect<i32>>,
    ) {
        let total_flex = self.children.iter().map(|&(ref _child, flex)| flex).sum();
        let min_size = min_sizes[&self.id()];
        let own_rect = if total_flex == 0.0 {
            Rect::new(rect.start, rect.start + vec2(min_size.x, rect.size().y))
        } else {
            Rect::new(rect.start, rect.start + vec2(rect.size().x, rect.size().y))
        };
        widget_rects.insert(self.id(), own_rect);
        let mut next_pos = rect.start;
        let total_flex = if total_flex == 0.0 { 1.0 } else { total_flex };
        let extra_space = rect.size().x - min_size.x;
        for &(ref child, flex) in &self.children {
            let child_min_size = min_sizes[&child.id()];
            let widget_extra_space = (extra_space as f64 * flex / total_flex) as i32;
            let widget_width = child_min_size.x + widget_extra_space;
            let widget_rect = Rect::new(next_pos, next_pos + vec2(widget_width, rect.size().y));
            next_pos.x += widget_width;
            child.compute_rects(widget_rect, theme, min_sizes, widget_rects);
        }
    }
}

#[derive(Clone)]
pub struct TextBox {
    text: String,
    lines: Vec<String>,
    text_color: Color4,
    id: WidgetId,
}

impl TextBox {
    pub fn new(text: &str) -> Box<Self> {
        let mut res = Box::new(TextBox {
            text: text.to_string(),
            lines: vec![],
            text_color: Color4::BLACK,
            id: WidgetId::new(),
        });
        res.update_lines();
        res
    }

    pub fn text_color(mut self: Box<Self>, color: Color4) -> Box<Self> {
        self.text_color = color;
        self
    }

    fn update_lines(&mut self) {
        self.lines = self.text.split('\n').map(|x| x.to_string()).collect();
    }
}

impl Widget for TextBox {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn draw(
        &self,
        context: &GlContext,
        rect: Rect<i32>,
        theme: &Theme,
        _draw_2d: &mut Draw2d,
        _cursor_pos: Option<Point2<f64>>,
        _is_active: bool,
    ) {
        let advance_y = theme.font.advance_y();
        for (i, line) in self.lines.iter().enumerate() {
            theme.font.draw_string(
                context,
                &line,
                rect.start.cast().unwrap() + vec2(0, advance_y * i as i32),
                self.text_color,
            );
        }
    }

    fn min_size(
        &self,
        context: &GlContext,
        theme: &Theme,
        _min_sizes: &FnvHashMap<WidgetId, Vector2<i32>>,
        _window_size: Vector2<i32>,
    ) -> Vector2<i32> {
        let max_width = self.lines.iter().map(|x| theme.font.string_width(context, x) as i32).max();
        if let Some(max_width) = max_width {
            vec2(max_width as i32, theme.font.advance_y() as i32 * self.lines.len() as i32)
        } else {
            vec2(0, 0)
        }
    }
}

// This is intended to be persistent, which is tricky since widgets have to own
// their children, but you can clone it.
#[derive(Clone)]
pub struct MessageBox {
    lines: Vec<(String, Color4)>,
    max_lines: usize,
    id: WidgetId,
}

impl MessageBox {
    pub fn new(max_lines: usize) -> Box<Self> {
        Box::new(MessageBox { lines: vec![], max_lines, id: WidgetId::new() })
    }

    pub fn add_line(&mut self, color: Color4, line: String) {
        self.lines.push((line, color));
        if self.lines.len() > self.max_lines {
            self.lines.remove(0);
        }
    }
}

impl Widget for MessageBox {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn draw(
        &self,
        context: &GlContext,
        rect: Rect<i32>,
        theme: &Theme,
        _draw_2d: &mut Draw2d,
        _cursor_pos: Option<Point2<f64>>,
        _is_active: bool,
    ) {
        let advance_y = theme.font.advance_y();
        for (i, &(ref line, color)) in self.lines.iter().enumerate() {
            theme.font.draw_string(
                context,
                &line,
                rect.start.cast().unwrap() + vec2(0, advance_y * i as i32),
                color,
            );
        }
    }

    fn min_size(
        &self,
        context: &GlContext,
        theme: &Theme,
        _min_sizes: &FnvHashMap<WidgetId, Vector2<i32>>,
        _window_size: Vector2<i32>,
    ) -> Vector2<i32> {
        let max_width =
            self.lines.iter().map(|x| theme.font.string_width(context, &x.0) as i32).max();
        if let Some(max_width) = max_width {
            vec2(max_width as i32, theme.font.advance_y() as i32 * self.lines.len() as i32)
        } else {
            vec2(0, 0)
        }
    }
}

pub struct Overlap {
    id: WidgetId,
    children: Vec<Box<dyn Widget>>,
}

impl Overlap {
    pub fn new() -> Box<Self> {
        Box::new(Overlap { id: WidgetId::new(), children: vec![] })
    }

    pub fn child(mut self: Box<Self>, child: Box<dyn Widget>) -> Box<Self> {
        self.children.push(child);
        self
    }

    pub fn children(mut self: Box<Self>, children: Vec<Box<dyn Widget>>) -> Box<Self> {
        self.children.extend(children.into_iter());
        self
    }
}

impl Widget for Overlap {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn draw(
        &self,
        _context: &GlContext,
        _rect: Rect<i32>,
        _theme: &Theme,
        _draw_2d: &mut Draw2d,
        _cursor_pos: Option<Point2<f64>>,
        _is_active: bool,
    ) {
    }

    fn min_size(
        &self,
        _context: &GlContext,
        _theme: &Theme,
        min_sizes: &FnvHashMap<WidgetId, Vector2<i32>>,
        _window_size: Vector2<i32>,
    ) -> Vector2<i32> {
        let mut min_size: Vector2<i32> = Vector2::zero();
        for child in &self.children {
            let child_min_size = min_sizes[&child.id()];
            min_size.x = min_size.x.max(child_min_size.x);
            min_size.y = min_size.y.max(child_min_size.y);
        }
        min_size
    }

    fn children(&self) -> Vec<&dyn Widget> {
        self.children.iter().map(|child| &**child as &dyn Widget).collect()
    }

    fn compute_rects(
        &self,
        rect: Rect<i32>,
        theme: &Theme,
        min_sizes: &FnvHashMap<WidgetId, Vector2<i32>>,
        widget_rects: &mut FnvHashMap<WidgetId, Rect<i32>>,
    ) {
        let own_rect = rect;
        widget_rects.insert(self.id(), own_rect);
        for child in &self.children {
            child.compute_rects(own_rect, theme, min_sizes, widget_rects);
        }
    }
}

pub struct Empty {
    id: WidgetId,
}

impl Empty {
    pub fn new() -> Box<Self> {
        Box::new(Empty { id: WidgetId::new() })
    }
}

impl Widget for Empty {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn draw(
        &self,
        _context: &GlContext,
        _rect: Rect<i32>,
        _theme: &Theme,
        _draw_2d: &mut Draw2d,
        _cursor_pos: Option<Point2<f64>>,
        _is_active: bool,
    ) {
    }

    fn min_size(
        &self,
        _context: &GlContext,
        _theme: &Theme,
        _min_sizes: &FnvHashMap<WidgetId, Vector2<i32>>,
        _window_size: Vector2<i32>,
    ) -> Vector2<i32> {
        Vector2::zero()
    }
}

pub struct Padding {
    id: WidgetId,
}

impl Padding {
    pub fn new() -> Box<Self> {
        Box::new(Padding { id: WidgetId::new() })
    }
}

impl Widget for Padding {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn draw(
        &self,
        _context: &GlContext,
        _rect: Rect<i32>,
        _theme: &Theme,
        _draw_2d: &mut Draw2d,
        _cursor_pos: Option<Point2<f64>>,
        _is_active: bool,
    ) {
    }

    fn min_size(
        &self,
        _context: &GlContext,
        theme: &Theme,
        _min_sizes: &FnvHashMap<WidgetId, Vector2<i32>>,
        _window_size: Vector2<i32>,
    ) -> Vector2<i32> {
        vec2(theme.padding, theme.padding)
    }
}

pub struct Inset {
    id: WidgetId,
    child: Box<dyn Widget>,
}

impl Inset {
    pub fn new(child: Box<dyn Widget>) -> Box<Self> {
        Box::new(Inset { id: WidgetId::new(), child })
    }
}

impl Widget for Inset {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn draw(
        &self,
        _context: &GlContext,
        _rect: Rect<i32>,
        _theme: &Theme,
        _draw_2d: &mut Draw2d,
        _cursor_pos: Option<Point2<f64>>,
        _is_active: bool,
    ) {
    }

    fn min_size(
        &self,
        _context: &GlContext,
        theme: &Theme,
        min_sizes: &FnvHashMap<WidgetId, Vector2<i32>>,
        _window_size: Vector2<i32>,
    ) -> Vector2<i32> {
        min_sizes[&self.child.id()] + vec2(theme.padding * 2, theme.padding * 2)
    }

    fn children(&self) -> Vec<&dyn Widget> {
        vec![&*self.child]
    }

    fn compute_rects(
        &self,
        rect: Rect<i32>,
        theme: &Theme,
        min_sizes: &FnvHashMap<WidgetId, Vector2<i32>>,
        widget_rects: &mut FnvHashMap<WidgetId, Rect<i32>>,
    ) {
        widget_rects.insert(
            self.id(),
            Rect::new(rect.start, rect.end + vec2(theme.padding * 2, theme.padding * 2)),
        );
        self.child.compute_rects(
            Rect::new(
                rect.start + vec2(theme.padding, theme.padding),
                rect.end - vec2(theme.padding, theme.padding),
            ),
            theme,
            min_sizes,
            widget_rects,
        );
    }
}
