use nannou::prelude::*;
use nannou::draw::Camera;
use nannou::winit;

fn main() {
    nannou::app(model)
        .event(event)
        .update(update)
        .view(view)
        .run();
}

struct Model {
    texture: wgpu::Texture,
    grid: Grid,
    camera_is_active: bool,
    camera: Camera,
}

struct Grid {
    lines: Vec<Cuboid>,
}

impl Grid {
    pub fn new() -> Self {
        let num_lines = 1000;
        let world_size = 100000.0;
        let grid_thickness = 3.0;
        let mut lines = Vec::new();
        for i in 0..num_lines {
            let pos = map_range(i, 0, num_lines, -world_size, world_size);
            // lines X
            let centre = pt3(pos, 0.0, 0.0);
            let size = vec3(grid_thickness, grid_thickness, world_size);
            lines.push(geom::Cuboid::from_xyz_whd(centre, size));

            // lines Z
            let centre = pt3(0.0, 0.0, pos);
            let size = vec3(world_size, grid_thickness, grid_thickness);
            lines.push(geom::Cuboid::from_xyz_whd(centre, size));
        }

        Grid {
            lines
        }
    }

    pub fn draw(&self, draw: &Draw) {
        for c in &self.lines {
            let cpoints = c.triangles_iter().flat_map(geom::Tri::vertices);
            draw.mesh()
                .points(cpoints)
                .color(rgba(0.0, 1.0, 0.0, 0.1));
        }
    }
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(720, 720)
        .view(view)
        .key_pressed(key_pressed)
        .key_released(key_released)
        .mouse_moved(mouse_moved)
        .mouse_pressed(mouse_pressed)
        .mouse_released(mouse_released)
        .build()
        .unwrap();

    // Load the image from disk and upload it to a GPU texture.
    let assets = app.assets_path().unwrap();
    let img_path = assets.join("images").join("nature").join("nature_1.jpg");
    let texture = wgpu::Texture::from_path(app, img_path).unwrap();

    let grid = Grid::new();
    let camera = Camera::new();
    let camera_is_active = true;

    Model { texture, grid, camera_is_active, camera }
}

fn update(app: &App, model: &mut Model, update: Update) {
    const CAM_SPEED_HZ: f64 = 2.5;
    use nannou::draw::camera::pitch_yaw_to_direction;
    if model.camera_is_active {
        let velocity = (update.since_last.secs() * CAM_SPEED_HZ) as f32;
        // Go forwards on W.
        if app.keys.down.contains(&Key::W) {
            model.camera.eye += model.camera.direction() * velocity;
        }
        // Go backwards on S.
        if app.keys.down.contains(&Key::S) {
            model.camera.eye -= model.camera.direction() * velocity;
        }
        // Strafe left on A.
        if app.keys.down.contains(&Key::A) {
            let pitch = 0.0;
            let yaw = model.camera.yaw + std::f32::consts::PI * 0.5;
            let direction = pitch_yaw_to_direction(pitch, yaw);
            model.camera.eye += direction * velocity;
        }
        // Strafe right on D.
        if app.keys.down.contains(&Key::D) {
            let pitch = 0.0;
            let yaw = model.camera.yaw - std::f32::consts::PI * 0.5;
            let direction = pitch_yaw_to_direction(pitch, yaw);
            model.camera.eye += direction * velocity;
        }
        // Float down on Q.
        if app.keys.down.contains(&Key::Q) {
            let pitch = model.camera.pitch - std::f32::consts::PI * 0.5;
            let direction = pitch_yaw_to_direction(pitch, model.camera.yaw);
            model.camera.eye += direction * velocity;
        }
        // Float up on E.
        if app.keys.down.contains(&Key::E) {
            let pitch = model.camera.pitch + std::f32::consts::PI * 0.5;
            let direction = pitch_yaw_to_direction(pitch, model.camera.yaw);
            model.camera.eye += direction * velocity;
        }
    }
}

// Use raw device motion event for camera pitch and yaw.
// TODO: Check device ID for mouse here - not sure if possible with winit currently.
fn event(app: &App, model: &mut Model, event: Event) {
    if model.camera_is_active {
        if let Event::DeviceEvent(_device_id, event) = event {
            if let winit::event::DeviceEvent::Motion { axis, value } = event {
                let sensitivity = 0.004;
                match axis {
                    // Yaw left and right on mouse x axis movement.
                    0 => model.camera.yaw -= (value * sensitivity) as f32,
                    // Pitch up and down on mouse y axis movement.
                    _ => {
                        let max_pitch = std::f32::consts::PI * 0.5 - 0.0001;
                        let min_pitch = -max_pitch;
                        model.camera.pitch = (model.camera.pitch + (-value * sensitivity) as f32)
                            .min(max_pitch)
                            .max(min_pitch)
                    }
                }
            }
        }
    }
}


// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model, frame: Frame) {
    let win_rect = app.window_rect();
    let draw = app.draw().camera(model.camera);
    draw.background().rgb(0.1, 0.1, 0.1);
    // println!("camera = {:#?}", &model.camera);

    let centre = pt3(0.0, 0.0, 0.0);
    let size = vec3(100.0, 100.0, 100.0);
    let cuboid = geom::Cuboid::from_xyz_whd(centre, size);
    let wireframe = create_wireframe(&cuboid);

    let rot = vec3(
        // individual rotation
        app.time * 0.0,//1.11,
        app.time * 0.0,//1.22,
        app.time * 0.0,//1.33,
    );
    // draw the center
    let cpoints = cuboid.triangles_iter().flat_map(geom::Tri::vertices);
    draw.radians(rot)
        .mesh()
        .points(cpoints)
        .color(rgba(1.0, 0.0, 0.0, 0.1));

    // draw the wireframe
    for w in &wireframe {
        let wpoints = w.triangles_iter().flat_map(geom::Tri::vertices);
        draw.radians(rot)
            .mesh()
            .points(wpoints)
            .color(BLACK);
    }    

    // draw the grid
    model.grid.draw(&draw);

    // // Generate the triangulated points for a cuboid to use for out mesh.
    // let centre = pt3(0.0, 0.0, 0.0);
    // let size = vec3(1.0, 1.0, 1.0);
    // let cuboid = geom::Cuboid::from_xyz_whd(centre, size);
    // let points = cuboid
    //     .triangles_iter()
    //     .flat_map(geom::Tri::vertices)
    //     .map(|point| {
    //         // Tex coords should be in range (0.0, 0.0) to (1.0, 1.0);
    //         // This will have the logo show on the front and back faces.
    //         let [x, y, _] = point;
    //         let tex_coords = [x + 0.5, 1.0 - (y + 0.5)];
    //         (point, tex_coords)
    //     });

    // // Scale the points up to half the window size.
    // let cube_side = win_rect.w().min(win_rect.h()) * 0.5;
    // draw.scale(cube_side)
    //     .mesh()
    //     .points_textured(&model.texture, points)
    //     .z_radians(app.time * 0.33)
    //     .x_radians(app.time * 0.166 + -app.mouse.y / 100.0)
    //     .y_radians(app.time * 0.25 + app.mouse.x / 100.0);

    // Draw to the frame!
    draw.to_frame(app, &frame).unwrap();
}

fn create_wireframe(cuboid: &Cuboid) -> Vec<Cuboid> {
    let x = cuboid.x();
    let y = cuboid.y();
    let z = cuboid.z();
    let ww = cuboid.w();
    let xx = ww * 0.5;
    let hh = cuboid.h();
    let yy = hh * 0.5;
    let dd = cuboid.d();
    let zz = dd * 0.5;
    let w = 5.0; // wire width
    vec![
        //top
        Cuboid::from_x_y_z_w_h_d(x + -xx, y + yy, z + 0.0, w, w, dd),
        Cuboid::from_x_y_z_w_h_d(x + 0.0, y + yy, z + zz, ww, w, w),
        Cuboid::from_x_y_z_w_h_d(x + xx, y + yy, z + 0.0, w, w, dd),
        Cuboid::from_x_y_z_w_h_d(x + 0.0, y + yy, z + -zz, ww, w, w),
        //bottom
        Cuboid::from_x_y_z_w_h_d(x + -xx, y + -yy, z + 0.0, w, w, dd),
        Cuboid::from_x_y_z_w_h_d(x + 0.0, y + -yy, z + zz, ww, w, w),
        Cuboid::from_x_y_z_w_h_d(x + xx, y + -yy, z + 0.0, w, w, dd),
        Cuboid::from_x_y_z_w_h_d(x + 0.0, y + -yy, z + -zz, ww, w, w),
        //sides
        Cuboid::from_x_y_z_w_h_d(x + -xx, y + 0.0, z + -zz, w, hh, w),
        Cuboid::from_x_y_z_w_h_d(x + -xx, y + 0.0, z + zz, w, hh, w),
        Cuboid::from_x_y_z_w_h_d(x + xx, y + 0.0, z + zz, w, hh, w),
        Cuboid::from_x_y_z_w_h_d(x + xx, y + 0.0, z + -zz, w, hh, w),
    ]
}


// Toggle cursor grabbing and hiding on Space key.
fn key_pressed(app: &App, model: &mut Model, key: Key) {
    if let Key::Space = key {
        let window = app.main_window();
        if !model.camera_is_active {
            if window.set_cursor_grab(true).is_ok() {
                model.camera_is_active = true;
            }
        } else {
            if window.set_cursor_grab(false).is_ok() {
                model.camera_is_active = false;
            }
        }
        window.set_cursor_visible(!model.camera_is_active);
    }
}

fn key_released(_app: &App, _model: &mut Model, _key: Key) {}

fn mouse_moved(_app: &App, _model: &mut Model, _pos: Point2) {}

fn mouse_pressed(_app: &App, _model: &mut Model, _button: MouseButton) {}

fn mouse_released(_app: &App, _model: &mut Model, _button: MouseButton) {}
