use math::*;
use glium::{ VertexBuffer, Surface, Frame, Display, DrawParameters, PolygonMode, DepthTest, Depth , Program };
use glium::index::{ NoIndices, PrimitiveType };

#[derive(Clone, Copy, PartialEq)]
struct Vertex {
    pos: (f32, f32, f32),
    color: Color,
}

implement_vertex! {
    Vertex,
    pos normalize(false),
    color normalize(true)
}

pub struct LineRenderer {
    verts: Vec<Vertex>,
    program: Program,
}

impl LineRenderer {
    pub fn new(display: &Display) -> LineRenderer {
        let program = program!(display,
            150 => {
                vertex: include_str!("../shader/line_150.glslv"),
                fragment: include_str!("../shader/line_150.glslf")
            },
        ).unwrap();

        LineRenderer {
            verts: Vec::new(),
            program,
        }
    }

    pub fn line3d(&mut self, start: Point3<f32>, end: Point3<f32>, color: Color) {
        self.verts.push(Vertex {
            pos: start.into(),
            color
        });
        self.verts.push(Vertex {
            pos: end.into(),
            color
        });
    }

    pub fn render(&mut self, display: &Display, frame: &mut Frame, clip_from_world: &Matrix4<f32>, line_width: f32) {
        let params = DrawParameters {
            line_width: Some(line_width),
            polygon_mode: PolygonMode::Line,
            depth: Depth {
                test: DepthTest::IfLess,
                write: true,
                ..Depth::default()
            },
            ..DrawParameters::default()
        };

        let vbuf = VertexBuffer::new(display, &self.verts).unwrap();
        self.verts.clear();

        let uniforms = uniform! {
            uClipFromWorld: Into::<[[f32; 4]; 4]>::into(*clip_from_world),
        };

        frame.draw(&vbuf, NoIndices(PrimitiveType::LinesList), &self.program, &uniforms, &params).unwrap();
    }
}