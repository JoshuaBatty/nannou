use nannou::prelude::*;
use nannou::draw::camera::{Camera, CameraSettings};
use nannou::event::Key;
use std::collections::HashSet;

pub const VELOCITY: f32 = 0.03;
pub const MAX_VELOCITY: f32 = 0.2;

#[derive(Eq, PartialEq, PartialOrd, Hash)]
pub enum Keys {
    MoveForward, 
    MoveBackward, 
    StrafeLeft,
    StrafeRight,
    FlyUp,
    FlyDown,
}

/// First person camera settings.
pub struct FirstPersonSettings {
    /// Which button to press to move forward.
    pub move_forward_button: Key,
    /// Which button to press to move backward.
    pub move_backward_button: Key,
    /// Which button to press to strafe left.
    pub strafe_left_button: Key,
    /// Which button to press to strafe right.
    pub strafe_right_button: Key,
    /// Which button to press to fly up.
    pub fly_up_button: Key,
    /// Which button to press to fly down.
    pub fly_down_button: Key,
    /// Which button to press to move faster.
    pub move_faster_button: Key,
    /// The horizontal movement speed.
    ///
    /// This is measured in units per second.
    pub speed_horizontal: f32,
    /// The vertical movement speed.
    ///
    /// This is measured in units per second.
    pub speed_vertical: f32,
    /// The horizontal mouse sensitivity.
    ///
    /// This is a multiplier applied to horizontal mouse movements.
    pub mouse_sensitivity_horizontal: f32,
    /// The vertical mouse sensitivity.
    ///
    /// This is a multiplier applied to vertical mouse movements.
    pub mouse_sensitivity_vertical: f32,
}

impl FirstPersonSettings {

    /// Clicking and dragging OR two-finger scrolling will orbit camera,
    /// with LShift as pan modifer and LCtrl as zoom modifier
    pub fn default() -> FirstPersonSettings {
        FirstPersonSettings {
            move_forward_button : Key::W,
            move_backward_button: Key::S,
            strafe_left_button: Key::A,
            strafe_right_button: Key::D,
            fly_up_button: Key::E,
            fly_down_button: Key::Q,
            move_faster_button: Key::LShift,
            speed_horizontal: 1.0,
            speed_vertical: 1.0,
            mouse_sensitivity_horizontal: std::f32::consts::PI,
            mouse_sensitivity_vertical: std::f32::consts::PI,
        }
    }
}


/// Models a flying first person camera.
pub struct FirstPerson {
    /// The first person camera settings.
    pub settings: FirstPersonSettings,
    /// The yaw angle (in radians).
    pub yaw: f32,
    /// The pitch angle (in radians).
    pub pitch: f32,
    /// The direction we are heading.
    pub direction: Vec3,
    /// The position of the camera.
    pub position: Vec3,
    /// The velocity we are moving in the direction.
    pub velocity: f32,
    /// The keys that are pressed.
    keys: HashSet<Keys>,

    prev_mouse_pos: Point2,

    camera: Camera,
}

impl FirstPerson {
    /// Creates a new first person camera.
    pub fn new(position: Vec3, settings: FirstPersonSettings) -> FirstPerson {
        FirstPerson {
            settings,
            yaw: 0.0,
            pitch: 0.0,
            direction: Vec3::new(0.0, 0.0, 0.0),
            position,
            velocity: VELOCITY,
            keys: HashSet::new(),
            prev_mouse_pos: Point2::new(0.0,0.0),
            camera: Default::default(),
        }
    }

    pub fn camera(&self) -> Camera {
        self.camera
    }

    pub fn update(&mut self, camera_settings: CameraSettings) {
        let dh = self.velocity * self.settings.speed_horizontal;
        let (dx, dy, dz) = (self.direction.x, self.direction.y, self.direction.z);
        let (s, c) = (self.yaw.sin(), self.yaw.cos());
        self.camera = Camera::new(Vec3::new(
            self.position.x + (s * dx - c * dz) * dh,
            self.position.y + dy * self.velocity * self.settings.speed_vertical,
            self.position.z + (s * dz + c * dx) * dh
        ), camera_settings);
        self.camera.set_yaw_pitch(self.yaw, self.pitch);
        self.position = self.camera.position;
    }

    /// Respond to scroll and key press/release events
    pub fn window_event(&mut self, event: WindowEvent) {
        let &mut FirstPerson {
            ref mut yaw,
            ref mut pitch,
            ref mut keys,
            ref mut direction,
            ref mut velocity,
            ref settings,
            ..
        } = self;

        let pi = std::f32::consts::PI;

        match event {
            MouseMoved(pos) => {
                let x = pos.x - self.prev_mouse_pos.x;
                let y = pos.y - self.prev_mouse_pos.y;

                let dx = x * settings.mouse_sensitivity_horizontal;
                let dy = 1.0-(y * settings.mouse_sensitivity_vertical);

                *yaw = (*yaw - dx / 360.0 * pi / 4.0) % (2.0 * pi);
                *pitch = *pitch + dy / 360.0 * pi / 4.0;
                *pitch = (*pitch).min(pi / 2.0).max(-pi / 2.0);

                self.prev_mouse_pos = Point2::new(pos.x, pos.y);
            }
            KeyPressed(key) => {
                let (dx, dy, dz) = (direction.x, direction.y, direction.z);
                let sgn = |x: f32| if x == 0.0 { 0.0 } else { x.signum() };
                let mut set = |k, x: f32, y: f32, z: f32| {
                    let (x, z) = (sgn(x), sgn(z));
                    let (x, z) = if x != 0.0 && z != 0.0 {
                        (x / 2.0.sqrt(), z / 2.0.sqrt())
                    } else {
                        (x, z)
                    };
                    *direction = Vec3::new(x, y, z);
                    keys.insert(k);
                };

                if self.settings.move_forward_button == key {
                    set(Keys::MoveForward, -1.0, dy, dz)
                }
                if self.settings.move_backward_button == key {
                    set(Keys::MoveBackward, 1.0, dy, dz)
                }
                if self.settings.strafe_left_button == key {
                    set(Keys::StrafeLeft, dx, dy, 1.0)
                }
                if self.settings.strafe_right_button == key {
                    set(Keys::StrafeRight, dx, dy, -1.0)
                }
                if self.settings.fly_up_button == key {
                    set(Keys::FlyUp, dx, 1.0, dz)
                }
                if self.settings.fly_down_button == key {
                    set(Keys::FlyDown, dx, -1.0, dz)
                }
                if self.settings.move_faster_button == key {
                    *velocity = MAX_VELOCITY;
                }
            }
            KeyReleased(key) => {
                let (dx, dy, dz) = (direction.x, direction.y, direction.z);
                let sgn = |x: f32| if x == 0.0 { 0.0 } else { x.signum() };
                let mut set = |x: f32, y: f32, z: f32| {
                    let (x, z) = (sgn(x), sgn(z));
                    let (x, z) = if x != 0.0 && z != 0.0 {
                        (x / 2.0.sqrt(), z / 2.0.sqrt())
                    } else {
                        (x, z)
                    };
                    *direction = Vec3::new(x, y, z);
                };
                let mut release = |key, rev_key, rev_val| {
                    keys.remove(key);
                    if keys.contains(rev_key) { rev_val } else { 0.0 }
                };

                if self.settings.move_forward_button == key {
                    set(release(&Keys::MoveForward, &Keys::MoveBackward, 1.0), dy, dz);
                }
                if self.settings.move_backward_button == key {
                    set(release(&Keys::MoveBackward, &Keys::MoveForward, -1.0), dy, dz);
                }
                if self.settings.strafe_left_button == key {
                    set(dx, dy, release(&Keys::StrafeLeft, &Keys::StrafeRight, -1.0));
                }
                if self.settings.strafe_right_button == key {
                    set(dx, dy, release(&Keys::StrafeRight, &Keys::StrafeLeft, 1.0))
                }
                if self.settings.fly_up_button == key {
                    set(dx, release(&Keys::FlyUp, &Keys::FlyDown, -1.0), dz);
                }
                if self.settings.fly_down_button == key {
                    set(dx, release(&Keys::FlyDown, &Keys::FlyUp, 1.0), dz);
                }
                if self.settings.move_faster_button == key {
                    *velocity = VELOCITY;
                }
            }
            _ => (),
        }
    }
}
