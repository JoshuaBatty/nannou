// L-System
// Daniel Shiffman <http://www.shiffman.net>
// Nature of Code, Chapter 8

// Just demonstrating working with L-System strings
// No drawing
// Example 8-6: Simple Lsystem
use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    current: String,
    count: usize,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(200, 200)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .build()
        .unwrap();

    let current = "A".to_string();
    let count = 0;
    println!("Generation {}: {}", count, current);

    Model { current, count }
}

fn view(app: &App, _model: &Model, frame: Frame) {
    frame.clear(WHITE);
    let win = app.window_rect();
    let draw = app.draw();
    draw.text("Click mouse to generate")
        .x_y(win.left() + 80.0, win.bottom() + 20.0)
        .font_size(13)
        .color(BLACK);

    // Write to the window frame.
    draw.to_frame(app, &frame).unwrap();
}

fn mouse_pressed(_app: &App, model: &mut Model, _button: MouseButton) {
    let mut next = String::new();
    // Look through the current String to replace according to L-System rules
    for i in 0..model.current.len() {
        let c = model.current.chars().nth(i);
        if c == Some('A') {
            // If we find A replace with AB
            next = format!("{}AB", next);
        } else if c == Some('B') {
            // If we find B replace with A
            next = format!("{}A", next);
        }
    }
    // The current String is now the next one
    model.current = next;
    model.count += 1;
    // Print to console
    println!("Generation {}: {}", model.count, model.current);
    println!("{} {}", model.count, model.current.len());
}
