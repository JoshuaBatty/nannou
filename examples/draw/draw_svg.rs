use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    svg: Svg,
}

fn model(app: &App) -> Model {
    app.new_window().size(1200, 800).view(view).build().unwrap();

    let assets = app.assets_path().unwrap();
    let svg_path = assets.join("svg").join("demo2.svg");
//    let svg_path = assets.join("svg").join("johnny_automatic_the_beer_snob.svg");
    let svg = Svg::load(svg_path).expect("failed to load svg");

    Model {
        svg
    }
}

fn view(app: &App, m: &Model, frame: Frame) {
    frame.clear(WHITE);
    let draw = app.draw();

    m.svg.paths.iter().for_each(|p| {
        if let Some(color) = p.fill_color {
            draw.path()
                .fill()
                .x_y(app.mouse.x, app.mouse.y)
                .color(color)
                .events(p.events.iter().cloned());
        }
    
        if let Some(ref stroke) = p.stroke_style {
            draw.path()
                .stroke()
                .x_y(app.mouse.x, app.mouse.y)
                .stroke_weight(stroke.weight)
                .color(stroke.color)
                .join(stroke.line_join)
                .caps(stroke.line_cap)
                .events(p.events.iter().cloned());
        }
    });

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
