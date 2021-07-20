use crate::geom::{self, Point2, Point3, Rect};
use crate::glam::{Mat4, Vec2, Vec3};

// Default camera values
const CAM_SPEED_HZ: f64 = 0.5;

// A simple first person camera.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Camera {
    //---- CAMERA ATTRIBUTES
    // The position of the camera.
    pub eye: Point3,

    //---- EULER ANGLES
    // Rotation around the x axis.
    pub pitch: f32,
    // Rotation around the y axis.
    pub yaw: f32,
}

impl Camera {
    /// Create a new **Draw** instance.
    ///
    /// This is the same as calling **Draw::default**.
    pub fn new() -> Self {
        Self::default()
    }

    // Calculate the direction vector from the pitch and yaw.
    pub fn direction(&self) -> Vec3 {
        pitch_yaw_to_direction(self.pitch, self.yaw)
    }

    // The camera's "view" matrix.
    pub fn view(&self) -> Mat4 {
        let direction = self.direction();
        let up = Vec3::Y;
        Mat4::look_at_rh(self.eye, direction, up)
    }
}

pub fn pitch_yaw_to_direction(pitch: f32, yaw: f32) -> Vec3 {
    let xz_unit_len = pitch.cos();
    let x = xz_unit_len * yaw.cos();
    let y = pitch.sin();
    let z = xz_unit_len * (-yaw).sin();
    Vec3::new(x, y, z)
}

impl Default for Camera {
    fn default() -> Self {
        let eye = Point3::new(0.0, 0.0, 1.0);
        let pitch = 0.0;
        let yaw = std::f32::consts::PI * 0.5;
        Camera { 
            eye, 
            pitch, 
            yaw 
        }
    }
}
