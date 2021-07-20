use nannou::prelude::*;

fn main() {
    nannou::app(model)
        .event(event)
        .update(update)
        .view(view)
        .run();
}

struct Model {
    grid: Grid,
    camera: Camera,
    cam_controller: CameraController,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(1280, 720)
        .view(view)
        .build()
        .unwrap();

    let camera = Camera::new()
        .fov(std::f32::consts::PI / 4.0)
        .position(vec3(0.0, 100.0, -600.0));
    let speed = 1000.0;
    let sensitivity = 1.0;
    let cam_controller = CameraController::new(speed, sensitivity);

    Model { 
        grid: Grid::new(),
        camera,
        cam_controller,  
    }
}

fn event(app: &App, model: &mut Model, event: Event) {
    model.cam_controller.process_event(&app, &event);
}

fn update(_app: &App, model: &mut Model, update: Update) {
    model.cam_controller.update_camera(&mut model.camera, update.since_last);
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Begin drawing using a 3d Perspective Projection
    let draw = app.draw().camera(model.camera);
    draw.background().rgb(0.07, 0.07, 0.07);

    let grid_size = 6; 
    let cube_size = 150.0;

    for x in 0..grid_size {
        for y in 0..grid_size {
            for z in 0..grid_size {
                let position = pt3(
                    (cube_size / 2.0) + (x as f32 * cube_size) - (cube_size * grid_size as f32 / 2.0), 
                    (cube_size / 2.0) + (y as f32 * cube_size) - (cube_size * grid_size as f32 / 2.0), 
                    (cube_size / 2.0) + (z as f32 * cube_size) - (cube_size * grid_size as f32 / 2.0)
                );

                let dist = 1.0 - map_range(
                    position.distance(pt3(0.0,0.0,0.0)), 
                    cube_size / 2.0, 
                    (cube_size / 2.0) + (cube_size * grid_size as f32 / 2.0), 
                    0.0, 
                    1.0
                );

                let hsva = hsva(dist, 1.0, 0.5, 0.2);
                draw_cube(&draw, position, cube_size, dist, hsva);
            }
        }
    }
    
    // draw the grid
    model.grid.draw(&draw);

    // Draw the 3d Perspective visuals to the frame!
    draw.to_frame(app, &frame).unwrap();

    // Begin drawing using a 2d Orthographic Projection
    let draw = app.draw().orthographic();
    let win = app.window_rect();
    let pos = model.camera.position;
    let text = format!("Position: {:.1} {:.1} {:.1} \n\nYaw: {:.} \n\nPitch: {:.} ", pos.x, pos.y, pos.z, model.camera.yaw, model.camera.pitch);
    draw.text(&text)
        .left_justify()
        .color(WHITE)
        .font_size(14)
        .xy(vec2(win.left() + 180.0, win.top() - 55.0))
        .wh(vec2(300.0,50.0));

    // Draw the 2d Orthographic visuals to the frame!
    draw.to_frame(app, &frame).unwrap();
}

fn draw_cube(draw: &Draw, pos: Point3, size: f32, dist: f32, hsva: Hsva) {
    let cuboid = geom::Cuboid::from_xyz_whd(pos, vec3(size, size, size) * dist);
    let wireframe = create_wireframe(&cuboid, dist * 3.0);
    
    // draw the center
    let cpoints = cuboid.triangles_iter().flat_map(geom::Tri::vertices);
    draw.mesh()
        .points(cpoints)
        .color(hsva);

    // draw the wireframe
    for w in &wireframe {
        let wpoints = w.triangles_iter().flat_map(geom::Tri::vertices);
        draw.mesh()
            .points(wpoints)
            .color(BLACK);
    }    
}

fn create_wireframe(cuboid: &Cuboid, wire_width: f32) -> Vec<Cuboid> {
    let x = cuboid.x();
    let y = cuboid.y();
    let z = cuboid.z();
    let ww = cuboid.w();
    let xx = ww * 0.5;
    let hh = cuboid.h();
    let yy = hh * 0.5;
    let dd = cuboid.d();
    let zz = dd * 0.5;
    let w = wire_width; 
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

struct Grid {
    lines: Vec<Cuboid>,
}

impl Grid {
    pub fn new() -> Self {
        let num_lines = 2000;
        let world_size = 100000.0;
        let grid_thickness = 2.0;
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
        for (i,c) in self.lines.iter().enumerate() {
            let cpoints = c.triangles_iter().flat_map(geom::Tri::vertices);
            let a = if i % 5 == 0 {
                0.1
            } else {
                0.01
            };
            draw.mesh()
                .points(cpoints)
                .color(rgba(1.0, 1.0, 1.0, a));
        }
    }
}