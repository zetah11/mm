use std::fmt;

use hypermelon::build::raw;
use hypermelon::prelude::Elem;
use hypermelon::{attrs, build};

use super::draw::Canvas;

const STYLE: &str = r#"
    .grid {
        stroke: gray;
        stroke-width: 0.5;
    }

    .grid .minor {
        stroke-width: 0.2;
    }

    .labels {
        text-anchor: end;
    }

    .notes {
        fill: blue;
    }
"#;

pub fn render(canvas: Canvas, w: impl fmt::Write) -> fmt::Result {
    let width = canvas.max_x;
    let height = canvas.max_y - canvas.min_y;

    let label_size = canvas.pitch_height;
    let label_width = label_size * 2.0;

    let svg = build::elem("svg").with(attrs!(
        ("xmlns", "http://www.w3.org/2000/svg"),
        ("width", width + label_width),
        ("height", height)
    ));

    let style = build::elem("style").append(STYLE);

    let cols = (width / canvas.unit_width) as usize;
    let rows = ((height + canvas.min_y) / canvas.pitch_height) as usize;

    let labels = canvas.pitches.into_iter().map(|label| {
        build::elem("text")
            .with(attrs!(
                ("x", label_width - 2.0),
                ("y", label.y + label_size),
                ("font-size", label_size)
            ))
            .append(raw(format!("{}", label.pitch)))
    });

    let labels = build::elem("g")
        .with(("class", "labels"))
        .append(build::from_iter(labels));

    let verticals = (0..=4 * cols).map(|i| {
        let class = if i % 4 == 0 { "major" } else { "minor" };
        let x = canvas.unit_width * i as f64 / 4.0 + label_width;

        build::single("line").with(attrs!(
            ("x1", x),
            ("x2", x),
            ("y1", 0),
            ("y2", height),
            ("class", class)
        ))
    });

    let horizontals = (0..rows).map(|i| {
        let y = canvas.pitch_height * i as f64 - canvas.min_y;
        build::single("line").with(attrs!(
            ("x1", label_width),
            ("x2", width + label_width),
            ("y1", y),
            ("y2", y),
            ("class", "minor")
        ))
    });

    let grid = build::elem("g")
        .with(("class", "grid"))
        .append(build::from_iter(verticals))
        .append(build::from_iter(horizontals));

    let rectangles = build::elem("g")
        .with(("class", "notes"))
        .append(build::from_iter(canvas.rectangles.into_iter().map(
            |rect| {
                build::single("rect").with(attrs!(
                    ("x", rect.x + label_width),
                    ("y", rect.y),
                    ("width", rect.width),
                    ("height", rect.height)
                ))
            },
        )));

    let all = svg
        .append(style)
        .append(labels)
        .append(grid)
        .append(rectangles);

    hypermelon::render(all, w)
}
