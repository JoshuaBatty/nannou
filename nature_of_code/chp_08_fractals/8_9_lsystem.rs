// L-System
// Daniel Shiffman <http://www.shiffman.net>
// Nature of Code, Chapter 8

// Just demonstrating working with L-System strings
// No drawing
// Example 8-9: Lsystem
use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Turtle {
    todo: String,
    len: f32,
    theta: f32,
}

impl Turtle {
    fn new(s: String, l: f32, t: f32) -> Self {
        Turtle {
            todo: s,
            len: l,
            theta: t,
        }
    }

    fn render(&self, draw: Draw) {
        let mut draw = draw;
        for i in 0..self.todo.len() {
            match self.todo.chars().nth(i).unwrap() {
                'F' | 'G' => {
                    draw.line()
                        .start(pt2(0.0,0.0))
                        .end(pt2(self.len,0.0))
                        .rgba(0.0,0.0,0.0,0.8);
                    draw = draw.x_y(0.0,self.len);
                },
                '+' => {
                    draw = draw.rotate(self.theta);
                },
                '-' => {
                    draw = draw.rotate(-self.theta);
                },
                '[' => {
                    //draw2 = draw;
                }
                _ => (),
            }
        }
    }

    fn change_len(&mut self, percent: f32) {
        self.len *= percent;
    }
}

#[derive(Clone)]
struct Rule {
    a: char,
    b: String,
}

impl Rule {
    fn new(a: char, b: String) -> Self {
        Rule {
            a, 
            b,
        }
    }
}

/* LSystem Type */

// An LSystem has a starting sentence
// An a ruleset
// Each generation recursively replaces characteres in the sentence
// Based on the rulset

struct Lsystem {
    sentance: String,   // The sentence (a String)
    rule_set: Vec<Rule>,    // The ruleset (a vector of Rules)
    generation: usize,  // Keeping track of the generation #
}

impl Lsystem {
    // Construct an LSystem with a startin sentence and a ruleset
    fn new(axiom: String, r: Vec<Rule>) -> Self {
        Lsystem {
            sentance: axiom,
            rule_set: r,
            generation: 0,
        }
    }

    // Generate the next generation
    fn generate(&mut self) {
        let mut next_gen = String::new();
        // For every character in the sentence
        for i in 0..self.sentance.len() {
            // What is the character
            let curr = self.sentance.chars().nth(i).unwrap();
            // We will replace it with itself unless it matches one of our rules
            let mut replace = curr.to_string();
            // Check every rule
            for j in 0..self.rule_set.len() {
                let a = self.rule_set[j].a;
                // if we match the Rule, get the replacement String out of the Rule
                if a == curr {
                    replace = self.rule_set[j].b.clone();
                    break;
                }
            } 
            // Append replacement String
            next_gen = format!("{}{}", next_gen, replace);
        }
        // Replace sentance
        self.sentance = next_gen;
        // Increment generation
        self.generation += 1;
    }
}

struct Model {
    lsys: Lsystem,
    turtle: Turtle,
    counter: usize,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(800, 200)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .build()
        .unwrap();

    let rule_set = vec![Rule::new('F', "FF+[+F-F-F]-[-F+F+F]".to_string()); 1];
    let lsys = Lsystem::new("F".to_string(), rule_set);
    let turtle = Turtle::new(lsys.sentance.clone(), app.window_rect().h() / 3.0, 25.0.to_radians());

    Model { 
        lsys, 
        turtle,
        counter: 0,
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(WHITE);
    let win = app.window_rect();
    let draw = app.draw().rotate(-PI/2.0);
    model.turtle.render(draw);

    if model.counter < 5 {
        let draw = app.draw();
        model.turtle.render(draw);
    }

    // Write to the window frame.
    app.draw().to_frame(app, &frame).unwrap();
}

fn mouse_pressed(_app: &App, model: &mut Model, _button: MouseButton) {
    if model.counter < 5 {
        model.lsys.generate();
        model.turtle.todo = model.lsys.sentance.clone();
        model.turtle.change_len(0.5);
        model.counter += 1;
    }
}
