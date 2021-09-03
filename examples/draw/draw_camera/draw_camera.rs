use nannou::prelude::*;
use nannou::draw::CameraSettings;

use nannou_conrod as ui;
use nannou_conrod::prelude::*;

mod first_person;
use first_person::{FirstPerson, FirstPersonSettings};

mod orbit_zoom;
use orbit_zoom::{OrbitZoomCamera, OrbitZoomCameraSettings};

fn main() {
    nannou::app(model)
        .update(update)
        .view(view)
        .run();
}

widget_ids! {
    struct Ids {
        fov,
        near_clip,
        far_clip,
    }
}

struct Model {
    grid: Grid,
    first_person: FirstPerson,
    orbit_zoom: OrbitZoomCamera,
    camera_settings: CameraSettings,
    ui: Ui,
    ids: Ids,
}

fn model(app: &App) -> Model {
    let window_id = app.new_window()
        .size(1280, 720)
        .event(window_event)
        .raw_event(raw_window_event)
        .view(view)
        .build()
        .unwrap();
    
    // Create the UI for our window.
    let mut ui = ui::builder(app).window(window_id).build().unwrap();

    // Generate some ids for our widgets.
    let ids = Ids::new(ui.widget_id_generator());

    let orbit_zoom = OrbitZoomCamera::new(
        Vec3::new(0.0, 0.0, 0.0), 
        OrbitZoomCameraSettings::default()
    );    
    
    let mut first_person = FirstPerson::new(
        Vec3::new(0.0, 3.0, 6.0), 
        FirstPersonSettings::default()
    );
    first_person.pitch = 0.6;

    Model { 
        grid: Grid::new(),  
        first_person,
        orbit_zoom,
        camera_settings: Default::default(), 
        ui,
        ids, 
    }
}

fn window_event(_app: &App, model: &mut Model, event: WindowEvent) {
    model.first_person.window_event(event.clone());
    model.orbit_zoom.window_event(event);
}

fn raw_window_event(app: &App, model: &mut Model, event: &ui::RawWindowEvent) {
    model.ui.handle_raw_event(app, event);
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    // Calling `set_widgets` allows us to instantiate some widgets.
    let ui = &mut model.ui.set_widgets();

    fn slider(val: f32, min: f32, max: f32) -> widget::Slider<'static, f32> {
        widget::Slider::new(val, min, max)
            .w_h(200.0, 30.0)
            .label_font_size(15)
            .rgb(0.3, 0.3, 0.3)
            .label_rgb(1.0, 1.0, 1.0)
            .border(0.0)
    }

    for value in slider(model.camera_settings.fov, 0.1, std::f32::consts::FRAC_PI_2)
        .top_left_with_margin(20.0)
        .label("FOV")
        .set(model.ids.fov, ui)
    {
        model.camera_settings.fov = value;
    }

    for value in slider(model.camera_settings.near_clip, 0.0, 40.0)
        .down(10.0)
        .label("Near Clip")
        .set(model.ids.near_clip, ui)
    {
        model.camera_settings.near_clip = value;
    }

    for value in slider(model.camera_settings.far_clip, 40.0, 200.0)
        .down(10.0)
        .label("Far Clip")
        .set(model.ids.far_clip, ui)
    {
        model.camera_settings.far_clip = value;
    }

    model.first_person.update(model.camera_settings);
    model.orbit_zoom.update(model.camera_settings);
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model, frame: Frame) {
//    let draw = app.draw().camera(model.first_person.camera());
    let draw = app.draw().camera(model.orbit_zoom.camera());
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

    // Draw to the frame!
    draw.to_frame(app, &frame).unwrap();

    // Draw the state of the `Ui` to the frame.
    model.ui.draw_to_frame(app, &frame).unwrap();
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