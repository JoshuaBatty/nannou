// P_2_0_02
//
// Generative Gestaltung – Creative Coding im Web
// ISBN: 978-3-87439-902-9, First Edition, Hermann Schmidt, Mainz, 2018
// Benedikt Groß, Hartmut Bohnacker, Julia Laub, Claudius Lazzeroni
// with contributions by Joey Lee and Niels Poldervaart
// Copyright 2018
//
// http://www.generative-gestaltung.de
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/**
 * drawing with a changing shape by draging the mouse.
 *
 * MOUSE
 * position x          : length
 * position y          : thickness and number of lines
 * drag                : draw
 *
 * KEYS
 * del, backspace      : erase
 * s                   : save png
 */

extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model;

fn model(app: &App) -> Model {
    //app.set_loop_mode(LoopMode::rate_fps(5.0));
    let _window = app.new_window()
        .view(view)
        .build()
        .unwrap();
    Model
}

fn update(_app: &App, _model: &mut Model, update: Update) {
    // println!("{:?}", update);
}

fn view(app: &App, _model: &Model, frame: Frame) -> Frame {
    app.main_window().set_inner_size_points(720.0, 720.0);

    // Prepare to draw.
    let draw = app.draw();
    let win = app.window_rect();
    
    let circle_resolution = map_range(app.mouse.y, win.top(), win.bottom(), 3, 10);
    let radius = app.mouse.x - win.left();
    let angle = TAU / circle_resolution as f32;

    // NOTE WE NEED VULKANO INTAMEDIARY IMAGES BEFORE WE CAN DRAW BACKGROUND ON INIT AND THEN LAYER FROM THERE
    //draw.background().color(BLACK);



    let mut points = Vec::new();
    for i in 0..circle_resolution {
        let x = (angle * i as f32).cos() * radius;
        let y = (angle * i as f32).sin() * radius;
        points.push(pt2(x, y));
    }

    let norm_mouse_y = (app.mouse.y / app.window_rect().h()) + 0.5;
    if(app.elapsed_frames() % 5 == 0){
        draw.rect()
            .wh(app.window_rect().wh())
            .rgba(0.0, 0.0, 0.0, 0.03);

        draw.polygon()
            .points(points)
            .hsva(norm_mouse_y,1.0,1.0,0.1);
    }
    // Write to the window frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
