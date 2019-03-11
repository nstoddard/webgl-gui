use cgmath::*;
use std::ops::Neg;
use web_sys::WebGlProgram;
use webgl_wrapper::uniforms::*;
use webgl_wrapper::*;

use crate::color::*;
use crate::shader_header::*;

struct PlainVert {
    pos: Point2<f32>,
    color: Color4,
}

impl Vertex for PlainVert {
    const ATTRIBUTES: Attributes = &[("pos", 2), ("color", 4)];
}

impl VertexComponent for PlainVert {
    fn add_to_mesh(&self, f: &mut dyn FnMut(f32)) {
        self.pos.add_to_mesh(f);
        self.color.add_to_mesh(f);
    }
}

struct PlainUniforms {
    matrix: Matrix4<f32>,
    color: Color4,
}

struct PlainUniformsGl {
    matrix: Matrix4Uniform,
    color: Color4Uniform,
}

impl Uniforms for PlainUniforms {
    type GlUniforms = PlainUniformsGl;

    fn update(&self, context: &GlContext, gl_uniforms: &Self::GlUniforms) {
        gl_uniforms.matrix.set(context, &self.matrix);
        gl_uniforms.color.set(context, &self.color, false);
    }
}

impl GlUniforms for PlainUniformsGl {
    fn new(context: &GlContext, program: &WebGlProgram) -> Self {
        let matrix = Matrix4Uniform::new("matrix", context, program);
        let color = Color4Uniform::new("uniColor", context, program);
        PlainUniformsGl { matrix, color }
    }
}

struct ImageVert {
    pos: Point2<f32>,
    uv: Point2<f32>,
    color: Color4,
}

impl Vertex for ImageVert {
    const ATTRIBUTES: Attributes = &[("pos", 2), ("uv", 2), ("color", 4)];
}

impl VertexComponent for ImageVert {
    fn add_to_mesh(&self, f: &mut dyn FnMut(f32)) {
        self.pos.add_to_mesh(f);
        self.uv.add_to_mesh(f);
        self.color.add_to_mesh(f);
    }
}

struct ImageUniforms<'a> {
    matrix: Matrix4<f32>,
    color: Color4,
    tex: &'a Texture2d,
}

struct ImageUniformsGl {
    matrix: Matrix4Uniform,
    color: Color4Uniform,
    tex: TextureUniform,
}

impl<'a> Uniforms for ImageUniforms<'a> {
    type GlUniforms = ImageUniformsGl;

    fn update(&self, context: &GlContext, gl_uniforms: &Self::GlUniforms) {
        gl_uniforms.matrix.set(context, &self.matrix);
        gl_uniforms.color.set(context, &self.color, false);
        gl_uniforms.tex.set(context, self.tex, 0);
    }
}

impl GlUniforms for ImageUniformsGl {
    fn new(context: &GlContext, program: &WebGlProgram) -> Self {
        let matrix = Matrix4Uniform::new("matrix", context, program);
        let color = Color4Uniform::new("uniColor", context, program);
        let tex = TextureUniform::new("tex", context, program);
        ImageUniformsGl { matrix, color, tex }
    }
}

/// A struct for drawing 2D shapes.
///
/// All distance units are pixels, from the top-left corner of the screen.
///
/// This is expensive to create, so try to only create one of them.
// TODO: put program creation somewhere else so this isn't so expensive to create
// TODO: many of the methods here should be on MeshBuilder<PlainVert, Triangles>
pub struct Draw2d {
    triangle_mesh_builder: MeshBuilder<PlainVert, Triangles>,
    triangle_mesh: Mesh<PlainVert, PlainUniformsGl, Triangles>,
    image_mesh_builder: MeshBuilder<ImageVert, Triangles>,
    image_mesh_srgb: Mesh<ImageVert, ImageUniformsGl, Triangles>,
    image_mesh_linear: Mesh<ImageVert, ImageUniformsGl, Triangles>,
}

impl Draw2d {
    /// Creates an object that can render a few types of basic geometric shapes.
    pub fn new(context: &GlContext) -> Self {
        let plain_program: GlProgram<PlainVert, PlainUniformsGl> = GlProgram::new_with_header(
            &context,
            include_str!("../shaders/plain_vert.glsl"),
            include_str!("../shaders/plain_frag.glsl"),
            true,
        );
        let image_program_srgb: GlProgram<ImageVert, ImageUniformsGl> = GlProgram::new_with_header(
            &context,
            include_str!("../shaders/image_vert.glsl"),
            include_str!("../shaders/image_frag.glsl"),
            true,
        );
        let image_program_linear: GlProgram<ImageVert, ImageUniformsGl> =
            GlProgram::new_with_header(
                &context,
                include_str!("../shaders/image_vert.glsl"),
                include_str!("../shaders/image_frag.glsl"),
                false,
            );
        let triangle_mesh_builder = MeshBuilder::new();
        let triangle_mesh = Mesh::new(context, &plain_program, DrawMode::Draw2D);
        let image_mesh_builder = MeshBuilder::new();
        let image_mesh_srgb = Mesh::new(context, &image_program_srgb, DrawMode::Draw2D);
        let image_mesh_linear = Mesh::new(context, &image_program_linear, DrawMode::Draw2D);
        Self {
            triangle_mesh_builder,
            triangle_mesh,
            image_mesh_builder,
            image_mesh_srgb,
            image_mesh_linear,
        }
    }

    /// Render all queued shapes. Until this is called nothing is actually rendered.
    ///
    /// This should typically be called once per frame to minimize the number of draw calls.
    pub fn render_queued(&mut self, surface: &impl Surface) {
        let surface_size = surface.size();
        let matrix = Matrix4::from_nonuniform_scale(1.0, -1.0, 1.0)
            * ortho(0.0, surface_size.x as f32, 0.0, surface_size.y as f32, 0.0, 1.0);

        self.triangle_mesh.build_from(&self.triangle_mesh_builder, MeshUsage::DynamicDraw);
        self.triangle_mesh.draw(surface, &PlainUniforms { matrix, color: Color4::WHITE });

        self.triangle_mesh_builder.clear();
    }

    /// Draws a filled convex polygon.
    pub fn fill_poly(&mut self, verts: &[Point2<f32>], color: Color4) {
        assert!(verts.len() >= 3);
        let mesh_builder = &mut self.triangle_mesh_builder;
        let a = mesh_builder.vert(PlainVert { pos: verts[0], color });
        let mut b = mesh_builder.vert(PlainVert { pos: verts[1], color });
        for c in verts.iter().skip(2) {
            let c = mesh_builder.vert(PlainVert { pos: *c, color });
            mesh_builder.triangle(a, b, c);
            b = c;
        }
    }

    /// Draws a line strip.
    // TODO: change all coords to i32? Then ensure that all lines are rendered in a pixel-perfect way; the coordinates will have to be adjusted by half for either even or odd line widths; I don't remember which. But first, check how quantizing this affects hex grid rendering; does it make it better or worse?
    pub fn draw_line_strip(&mut self, verts: &[Point2<f32>], color: Color4, width: f32) {
        assert!(verts.len() >= 2);
        let mesh_builder = &mut self.triangle_mesh_builder;
        let half_width = width * 0.5;
        for (a, b) in verts.iter().zip(verts.iter().skip(1)) {
            let perp = ccw_perp(*b - *a).normalize();
            let vert_a = mesh_builder.vert(PlainVert { pos: *a + perp * half_width, color });
            let vert_b = mesh_builder.vert(PlainVert { pos: *a - perp * half_width, color });
            let vert_c = mesh_builder.vert(PlainVert { pos: *b + perp * half_width, color });
            let vert_d = mesh_builder.vert(PlainVert { pos: *b - perp * half_width, color });
            mesh_builder.triangle(vert_a, vert_b, vert_c);
            mesh_builder.triangle(vert_b, vert_c, vert_d);
        }
    }

    pub fn fill_rect(&mut self, rect: Rect<i32>, color: Color4) {
        let rect = rect.cast().unwrap();
        self.fill_poly(
            &[
                rect.start,
                point2(rect.end.x, rect.start.y),
                rect.end,
                point2(rect.start.x, rect.end.y),
            ],
            color,
        );
    }

    pub fn outline_rect(&mut self, rect: Rect<i32>, color: Color4, width: f32) {
        let rect = rect.cast().unwrap();
        self.draw_line_strip(
            &[
                rect.start,
                point2(rect.end.x, rect.start.y),
                rect.end,
                point2(rect.start.x, rect.end.y),
            ],
            color,
            width,
        );
    }

    /// Draws an image. Unlike the other functions on `Draw2d`, this draws the image immediately.
    pub fn draw_image(&mut self, surface: &impl Surface, tex: &Texture2d, pos: Point2<f32>) {
        let surface_size = surface.size();
        let matrix = Matrix4::from_nonuniform_scale(1.0, -1.0, 1.0)
            * ortho(0.0, surface_size.x as f32, 0.0, surface_size.y as f32, 0.0, 1.0);

        let a = self.image_mesh_builder.vert(ImageVert {
            pos,
            uv: point2(0.0, 0.0),
            color: Color4::WHITE,
        });
        let b = self.image_mesh_builder.vert(ImageVert {
            pos: pos + vec2(tex.size().x as f32, 0.0),
            uv: point2(1.0, 0.0),
            color: Color4::WHITE,
        });
        let c = self.image_mesh_builder.vert(ImageVert {
            pos: pos + vec2(0.0, tex.size().y as f32),
            uv: point2(0.0, 1.0),
            color: Color4::WHITE,
        });
        let d = self.image_mesh_builder.vert(ImageVert {
            pos: pos + vec2(tex.size().x as f32, tex.size().y as f32),
            uv: point2(1.0, 1.0),
            color: Color4::WHITE,
        });
        self.image_mesh_builder.triangle(a, b, c);
        self.image_mesh_builder.triangle(b, c, d);

        let image_mesh =
            if tex.is_srgb() { &mut self.image_mesh_srgb } else { &mut self.image_mesh_linear };
        image_mesh.build_from(&self.image_mesh_builder, MeshUsage::DynamicDraw);
        image_mesh.draw(surface, &ImageUniforms { matrix, color: Color4::WHITE, tex });

        self.image_mesh_builder.clear();
    }
}

/// Returns the vector 90 degrees counterclockwise from the given vector.
#[inline]
pub fn ccw_perp<T: Neg<Output = T>>(x: Vector2<T>) -> Vector2<T> {
    vec2(x.y, -x.x)
}
