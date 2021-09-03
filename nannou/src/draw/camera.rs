use crate::geom::Point3;
use crate::glam::{Mat4, Vec3, Quat};

/// Models camera perspective settings.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CameraSettings {
    /// Field of view (in degrees).
    pub fov: f32,
    /// The near clip distance.
    pub near_clip: f32,
    /// The far clip distance.
    pub far_clip: f32,
}

/// Models a camera with position and directions.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Camera {
    /// The camera position.
    pub position: Vec3,
    /// The up direction.
    pub up: Vec3,
    /// The right direction.
    pub right: Vec3,
    /// The forward direction.
    pub forward: Vec3,
    /// Perspective settings.
    pub settings: CameraSettings,
}

impl Camera {
    /// Constructs a new camera.
    ///
    /// Places the camera at [x, y, z], looking towards pozitive z.
    pub fn new(position: Vec3, settings: CameraSettings) -> Self {
        Camera {
            position,
            settings,
            ..Camera::default()
        }
    }

    /// Computes an orthogonal matrix for the camera.
    ///
    /// This matrix can be used to transform coordinates to the screen.
    pub fn orthogonal(&self) -> Mat4 {
        let p = self.position;
        let r = self.right;
        let u = self.up;
        let f = self.forward;
        Mat4::from_cols_array_2d(&[
            [r[0], u[0], f[0], 0.0],
            [r[1], u[1], f[1], 0.0],
            [r[2], u[2], f[2], 0.0],
            [-r.dot(p), -u.dot(p), -f.dot(p), 1.0]
        ])
    }

    /// Computes a projection matrix for the camera perspective.
    pub fn projection(&self, aspect_ratio: f32) -> Mat4 {
        Mat4::perspective_rh_gl(self.settings.fov, aspect_ratio, self.settings.near_clip, self.settings.far_clip)
    }

    /// Orients the camera to look at a point.
    pub fn look_at(&mut self, point: Vec3) {
        self.forward = (self.position - point).normalize();
        self.update_right();
    }

    /// Sets yaw and pitch angle of camera in radians.
    pub fn set_yaw_pitch(&mut self, yaw: f32, pitch: f32) {
        let (y_s, y_c, p_s, p_c) = (yaw.sin(), yaw.cos(), pitch.sin(), pitch.cos());
        self.forward = Vec3::new(y_s * p_c, p_s, y_c * p_c);
        self.up = Vec3::new(y_s * -p_s, p_c, y_c * -p_s);
        self.update_right();
    }

    /// Sets forward, up, and right vectors from a Quaternion rotation
    /// relative to the positive z-axis
    pub fn set_rotation(&mut self, rotation: Quat)
    {
        let forward: Vec3 = Vec3::new(0.0, 0.0, 1.0);
        let up: Vec3 = Vec3::new(0.0, 1.0, 0.0);
        self.forward = rotate_vector(rotation, forward);
        self.up = rotate_vector(rotation, up);
        self.update_right();
    }

    fn update_right(&mut self) {
        self.right = self.up.cross(self.forward);
    }
}

impl Default for Camera {
    fn default() -> Self {
        let position = Point3::new(0.0, 0.0, 1.0);
        Camera {
            position,
            right: Vec3::new(1.0, 0.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            forward: Vec3::new(0.0, 0.0, 1.0),
            settings: Default::default(),
        }
    }
}

impl Default for CameraSettings {
    fn default() -> Self {
        CameraSettings {
            fov: std::f32::consts::FRAC_PI_2,
            near_clip: 0.01,
            far_clip: 100.0,
        }
    }
}

/// Rotate the given vector using the given quaternion
pub fn rotate_vector(q: Quat, v: Vec3) -> Vec3 {
    // Extract the vector part of the quaternion
    let u: Vec3 = Vec3::new(q.x, q.y, q.z);

    // Extract the scalar part of the quaternion
    let s = q.w;

    2.0 * u.dot(v) * u
          + (s*s - u.dot(u)) * v
          + 2.0 * s * u.cross(v)
}