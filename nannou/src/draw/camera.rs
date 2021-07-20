use crate::geom::Point3;
use crate::glam::{Mat4, Vec3, Vec3Swizzles};
use crate::event::{Event, Key};

use std::f32::consts::FRAC_PI_2;
use std::time::Duration;
use winit::dpi::PhysicalPosition;
use winit::event::*;

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Clone, Debug, PartialEq)]
pub enum Projection {
    Orthographic,
    Perspective(Camera),
}

/// Models a camera with position and directions.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Camera {
    /// The camera position.
    pub position: Vec3,
    /// The camera's yaw rotation.
    pub yaw: f32,
    /// The camera's pitch rotation.
    pub pitch: f32,
    /// Field of view
    pub fov: f32,
    /// The near clip distance.
    pub znear: f32,
    /// The far clip distance.
    pub zfar: f32,
}

impl Camera {
    /// Constructs a new camera.
    pub fn new() -> Self {
        Default::default()
    }

    /// Places the camera at [x, y, z], looking towards pozitive z.
    pub fn position<V: Into<Vec3>>(&mut self, position: V) -> Self {
        self.position = position.into().zyx();
        *self
    }

    /// Sets the yaw angle of camera in radians.
    pub fn yaw(&mut self, yaw: f32) -> Self {
        self.yaw = yaw;
        *self
    }

    /// Sets the pitch angle of camera in radians.
    pub fn pitch(&mut self, pitch: f32) -> Self {
        self.pitch = pitch;
        *self
    }

    /// Sets the field of view in radians.
    pub fn fov(&mut self, fov: f32) -> Self {
        self.fov = fov;
        *self
    }

    /// Sets the camera's near clip distance
    pub fn near_clip(&mut self, znear: f32) -> Self {
        self.znear = znear;
        *self
    }

    /// Sets the camera's far clip distance
    pub fn far_clip(&mut self, zfar: f32) -> Self {
        self.zfar = zfar;
        *self
    }

    pub fn calc_camera_matrix(&self) -> Mat4 {
        let (sin_pitch, cos_pitch) = (self.pitch.sin(), self.pitch.cos());
        let (sin_yaw, cos_yaw) = (self.yaw.sin(), self.yaw.cos());

        look_to_rh(
            self.position,
            Vec3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vec3::new(0.0, 1.0, 0.0),
        )
    }

    /// Computes a projection matrix for the camera perspective.
    pub fn calc_projection_matrix(&self, aspect: f32) -> Mat4 {
        let opengl_to_wgpu_matrix = Mat4::from_cols_array(
            &[1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.0,
            0.0, 0.0, 0.5, 1.0]
        ).transpose();
        opengl_to_wgpu_matrix * Mat4::perspective_rh(self.fov, aspect, self.znear, self.zfar)
    }
}

/// Create a homogeneous transformation matrix that will cause a vector to point at
/// `dir`, using `up` for orientation.
pub fn look_to_rh(eye: Vec3, dir: Vec3, up: Vec3) -> Mat4 {
    let f = dir.normalize();
    let s = f.cross(up).normalize();
    let u = s.cross(f);

    Mat4::from_cols_array(
        &[s.x.clone(), u.x.clone(), -f.x.clone(), 0.0,
        s.y.clone(), u.y.clone(), -f.y.clone(), 0.0,
        s.z.clone(), u.z.clone(), -f.z.clone(), 0.0,
        -eye.dot(s), -eye.dot(u), eye.dot(f), 1.0],
    )
}

impl Default for Camera {
    fn default() -> Self {
        let position = Point3::new(0.0, 0.0, 1.0);
        Camera {
            position,
            yaw: 0.0,
            pitch: 0.0,
            fov: std::f32::consts::PI / 4.0,
            znear: 0.01,
            zfar: 1000.0,
        }
    }
}

#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_event(&mut self, app: &crate::app::App, event: &Event) {
        match event {
            Event::WindowEvent { simple, .. } => {
                if let Some(event) = simple {
                    match event {
                        crate::event::WindowEvent::KeyPressed(key) => {
                            self.process_keyboard(*key, ElementState::Pressed);
                        }
                        crate::event::WindowEvent::KeyReleased(key) => {
                            self.process_keyboard(*key, ElementState::Released);
                        }
                        crate::event::WindowEvent::MouseWheel(dt, _) => {
                            self.process_scroll(&dt);
                        }
                        _ => (),
                    }
                }
            },
            Event::DeviceEvent(_device_id, device_event) => {
                if let winit::event::DeviceEvent::MouseMotion { delta } = device_event {
                    if app.mouse.buttons.pressed().next().is_some() {
                        self.process_mouse(delta.0 as f32, delta.1 as f32);
                    }
                }
            }
            _ => ()
        }
    }
    
    pub fn process_keyboard(&mut self, key: Key, state: ElementState) {
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };
        match key {
            Key::W | Key::Up => {
                self.amount_forward = amount;
            },
            Key::S | Key::Down => {
                self.amount_backward = amount;
            },
            Key::A | Key::Left => {
                self.amount_left = amount;
            },
            Key::D | Key::Right => {
                self.amount_right = amount;
            },
            Key::E => {
                self.amount_up = amount;
            },
            Key::Q => {
                self.amount_down = amount;
            },
            _ => (),
        }
    }

    pub fn process_mouse(&mut self, mouse_x: f32, mouse_y: f32) {
        self.rotate_horizontal = mouse_x;
        self.rotate_vertical = mouse_y;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, scroll) => -scroll * 0.5,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => -*scroll as f32,
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = (camera.yaw.sin(), camera.yaw.cos());
        let forward = Vec3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        let (pitch_sin, pitch_cos) = (camera.pitch.sin(), camera.pitch.cos());
        let scrollward =
            Vec3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        // Rotate
        camera.yaw += self.rotate_horizontal * self.sensitivity * dt;
        camera.pitch += -self.rotate_vertical * self.sensitivity * dt;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        if camera.pitch < -SAFE_FRAC_PI_2 {
            camera.pitch = -SAFE_FRAC_PI_2;
        } else if camera.pitch > SAFE_FRAC_PI_2 {
            camera.pitch = SAFE_FRAC_PI_2;
        }
    }
}