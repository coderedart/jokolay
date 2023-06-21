use glam::*;

pub const MARKER_MAX_VISIBILITY_DISTANCE: f32 = 10000.0;
#[derive(Debug, Default, Clone, Copy)]
pub struct MarkerQuad {
    pub position: Vec3,
    pub texture: u32,
    pub width: u16,
    pub height: u16,
}
impl MarkerQuad {
    pub fn get_vertices(self, camera_position: Vec3) -> [MarkerVertex; 6] {
        let MarkerQuad {
            position,
            texture: _,
            width,
            height,
        } = self;
        let mut billboard_direction = position - camera_position;
        billboard_direction.y = 0.0;
        let rotation = Quat::from_rotation_arc(Vec3::Z, billboard_direction.normalize());
        // let rotation = Quat::IDENTITY;
        let model_matrix = Mat4::from_scale_rotation_translation(
            vec3(width as f32 / 100.0, height as f32 / 100.0, 1.0),
            rotation,
            position,
        );
        let bottom_left = MarkerVertex {
            position: model_matrix * DEFAULT_QUAD[0],
            texture_coordinates: vec2(0.0, 1.0),
            padding: Vec2::default(),
        };

        let top_left = MarkerVertex {
            position: model_matrix * DEFAULT_QUAD[1],
            texture_coordinates: vec2(0.0, 0.0),
            padding: Vec2::default(),
        };
        let top_right = MarkerVertex {
            position: model_matrix * DEFAULT_QUAD[2],
            texture_coordinates: vec2(1.0, 0.0),
            padding: Vec2::default(),
        };
        let bottom_right = MarkerVertex {
            position: model_matrix * DEFAULT_QUAD[3],
            texture_coordinates: vec2(1.0, 1.0),
            padding: Vec2::default(),
        };
        [
            top_left,
            bottom_left,
            bottom_right,
            bottom_right,
            top_right,
            top_left,
        ]
    }
}
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MarkerVertex {
    pub position: Vec4,
    pub texture_coordinates: Vec2,
    pub padding: Vec2,
}

pub const DEFAULT_QUAD: [Vec4; 4] = [
    // bottom left
    vec4(-50.0, -50.0, 0.0, 1.0),
    // top left
    vec4(-50.0, 50.0, 0.0, 1.0),
    // top right
    vec4(50.0, 50.0, 0.0, 1.0),
    // bottom right
    vec4(50.0, -50.0, 0.0, 1.0),
];
