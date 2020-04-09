// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 3-3: Pointing Velocity
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Mover {
    location: Point2,
    velocity: Vector2,
    acceleration: Vector2,
    top_speed: f32,
}

impl Mover {
    fn new() -> Self {
        Mover {
            location: pt2(0.0, 0.0),
            velocity: pt2(0.0, 0.0),
            acceleration: pt2(0.0, 0.0),
            top_speed: 4.0,
        }
    }

    fn update(&mut self, mouse: Point2) {
        let mut dir = mouse - self.location;
        dir = dir.normalize();
        dir *= 0.5;
        self.acceleration = dir;
        self.velocity += self.acceleration;
        self.velocity = self.velocity.limit_magnitude(self.top_speed);
        self.location += self.velocity;
    }

    fn display(&self, draw: &Draw) {
        let draw = draw.xy(self.location).rotate(self.velocity.angle());
        draw.rect()
            .x_y(0.0, 0.0)
            .w_h(30.0, 10.0)
            .gray(0.5)
            .stroke(BLACK)
            .stroke_weight(2.0);
    }

    fn check_edges(&mut self, rect: Rect) {
        if self.location.x > rect.right() {
            self.location.x = rect.right();
            self.velocity.x *= -1.0;
        } else if self.location.x < rect.left() {
            self.velocity.x *= -1.0;
            self.location.x = rect.left();
        }
        if self.location.y < rect.bottom() {
            self.velocity.y *= -1.0;
            self.location.y = rect.bottom();
        }
    }
}

struct Model {
    mover: Mover,
}

fn model(app: &App) -> Model {
    app.new_window().size(800, 200).view(view).build().unwrap();

    Model {
        mover: Mover::new(),
    }
}

fn update(app: &App, m: &mut Model, _update: Update) {
    m.mover.update(pt2(app.mouse.x, app.mouse.y));
    m.mover.check_edges(app.window_rect());
}

fn view(app: &App, m: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    m.mover.display(&draw);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
