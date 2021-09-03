use nannou::prelude::*;
use nannou::draw::camera::{Camera, CameraSettings, rotate_vector};
use nannou::event::Key;
use nannou::state::mouse::Button;
use std::ops::Mul;
use std::collections::HashSet;

#[derive(Eq, PartialEq, PartialOrd, Hash)]
pub enum Keys {
    Zoom, 
    Pan, 
    Orbit
}

/// Specifies key bindings and speed modifiers for OrbitZoomCamera
pub struct OrbitZoomCameraSettings {

    /// Which button to press to orbit with mouse
    pub orbit_button: Button,

    /// Which button to press to zoom with mouse
    pub zoom_button: Key,

    /// Which button to press to pan with mouse
    pub pan_button: Key,

    /// Modifier for orbiting speed (arbitrary unit)
    pub orbit_speed: f32,

    /// Modifier for pitch speed relative to orbiting speed (arbitrary unit).
    /// To reverse pitch direction, set this to -1.
    pub pitch_speed: f32,

    /// Modifier for panning speed (arbitrary unit)
    pub pan_speed: f32,

    /// Modifier for zoom speed (arbitrary unit)
    pub zoom_speed: f32,
}

impl OrbitZoomCameraSettings {

    /// Clicking and dragging OR two-finger scrolling will orbit camera,
    /// with LShift as pan modifer and LCtrl as zoom modifier
    pub fn default() -> OrbitZoomCameraSettings {
        OrbitZoomCameraSettings {
            orbit_button : Button::Middle,
            zoom_button : Key::LControl,
            pan_button : Key::LShift,
            orbit_speed: 0.005,
            pitch_speed: 0.9,
            pan_speed: 0.04,
            zoom_speed: 0.1,
        }
    }

    /// Set the button for orbiting
    pub fn orbit_button(self, button: Button) -> OrbitZoomCameraSettings {
        OrbitZoomCameraSettings {
            orbit_button: button,
            .. self
        }
    }

    /// Set the button for zooming
    pub fn zoom_button(self, button: Key) -> OrbitZoomCameraSettings {
        OrbitZoomCameraSettings {
            zoom_button: button,
            .. self
        }
    }

    /// Set the button for panning
    pub fn pan_button(self, button: Key) -> OrbitZoomCameraSettings {
        OrbitZoomCameraSettings {
            pan_button: button,
            .. self
        }
    }

    /// Set the orbit speed modifier
    pub fn orbit_speed(self, s: f32) -> OrbitZoomCameraSettings {
        OrbitZoomCameraSettings {
            orbit_speed: s,
            .. self
        }
    }

    /// Set the pitch speed modifier
    pub fn pitch_speed(self, s: f32) -> OrbitZoomCameraSettings {
        OrbitZoomCameraSettings {
            pitch_speed: s,
            .. self
        }
    }

    /// Set the pan speed modifier
    pub fn pan_speed(self, s: f32) -> OrbitZoomCameraSettings {
        OrbitZoomCameraSettings {
            pan_speed: s,
            .. self
        }
    }

    /// Set the zoom speed modifier
    pub fn zoom_speed(self, s: f32) -> OrbitZoomCameraSettings {
        OrbitZoomCameraSettings {
            zoom_speed: s,
            .. self
        }
    }
}


/// A 3dsMax / Blender-style camera that orbits around a target point
pub struct OrbitZoomCamera {

    /// origin of camera rotation
    pub target: Vec3,

    /// Rotation of camera
    pub rotation: Quat,

    /// Pitch up/down from target
    pub pitch: f32,

    /// Yaw left/right from target
    pub yaw: f32,

    /// camera distance from target
    pub distance: f32,

    /// Settings for the camera
    pub settings: OrbitZoomCameraSettings,

    /// Current keys that are pressed
    keys: HashSet<Keys>,

    prev_mouse_pos: Point2,

    camera: Camera,
}


impl OrbitZoomCamera {
    /// Create a new OrbitZoomCamera targeting the given coordinates
    pub fn new(target: Vec3, settings: OrbitZoomCameraSettings) -> OrbitZoomCamera {
        OrbitZoomCamera {
            target,
            rotation: Quat::from_xyzw(1.0,0.0,0.0,0.0),
            distance: 10.0,
            pitch: 0.0,
            yaw: 0.0,
            keys: HashSet::new(),
            settings: settings,
            prev_mouse_pos: Point2::new(0.0,0.0),
            camera: Default::default(),
        }
    }

    pub fn update(&mut self, camera_settings: CameraSettings) {
        let target_to_camera = rotate_vector(
            self.rotation,
            Vec3::new(0.0, 0.0, self.distance)
        );
        self.camera = Camera::new(self.target + target_to_camera, camera_settings);
        self.camera.set_rotation(self.rotation);
    }

    /// Return a Camera for the current OrbitZoomCamera configuration
    pub fn camera(&self) -> Camera {
        self.camera
    }

    /// Orbit the camera using the given horizontal and vertical params,
    /// or zoom or pan if the appropriate modifier keys are pressed
    fn control_camera(&mut self, x: f32, y: f32) {
        if self.keys.contains(&Keys::Pan) {
            // Pan target position along plane normal to camera direction
            let dx = x * self.settings.pan_speed;
            let dy = -y * self.settings.pan_speed;

            let right = rotate_vector(self.rotation, Vec3::new(1.0, 0.0, 0.0));
            let up = rotate_vector(self.rotation, Vec3::new(0.0, 1.0, 0.0));
            self.target = (self.target + (up * dy)) +
                (right * dx);
        } else if self.keys.contains(&Keys::Zoom) {
            // Zoom to / from target
            self.distance = self.distance + y * self.settings.zoom_speed;
        } else {
            // Orbit around target
            let dx = x * self.settings.orbit_speed;
            let dy = y * self.settings.orbit_speed;

            self.yaw = self.yaw + dx;
            self.pitch = self.pitch + dy*self.settings.pitch_speed;
            self.rotation = Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), self.yaw).mul(
                Quat::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), self.pitch)
            );
        }
    }

    /// Respond to scroll and key press/release events
    pub fn window_event(&mut self, event: WindowEvent) {
        match event {
            KeyPressed(key) => {
                if self.settings.zoom_button == key {
                    self.keys.insert(Keys::Zoom);
                }
                if self.settings.pan_button == key {
                    self.keys.insert(Keys::Pan);
                }
            }
            KeyReleased(key) => {
                if self.settings.zoom_button == key {
                    self.keys.remove(&Keys::Zoom);
                }
                if self.settings.pan_button == key {
                    self.keys.remove(&Keys::Pan);
                }  
            }
            MousePressed(button) => {
                if self.settings.orbit_button == button {
                    self.keys.insert(Keys::Orbit);
                }
            }
            MouseReleased(button) => {
                if self.settings.orbit_button == button {
                    self.keys.remove(&Keys::Orbit);
                }
            }
            MouseMoved(pos) => {
                if self.keys.contains(&Keys::Orbit){
                    let mx = pos.x - self.prev_mouse_pos.x;
                    let my = pos.y - self.prev_mouse_pos.y;
                    self.control_camera(-mx, my);
                    self.prev_mouse_pos = Point2::new(pos.x, pos.y);
                }
            }
            MouseWheel(amount, _phase) => {
                let (dx, dy) = match amount {
                    MouseScrollDelta::LineDelta(x,y) => {
                        (x,y)
                    }
                    MouseScrollDelta::PixelDelta(p) => {
                        (p.x as f32, p.y as f32)
                    }
                };
                self.control_camera(-dx, dy);
            }
            _ => (),
        }
    }
}
