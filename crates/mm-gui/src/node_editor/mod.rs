pub use self::graph::GraphEditor;

mod graph;

use std::hash::Hash;

use egui::{
    Align2, Id, Pos2, Rect, Response, Rounding, Sense, Shape, Stroke, TextStyle, Ui, Vec2, Widget,
};

use self::graph::NodeId;

pub struct GraphView<'a> {
    graph: &'a mut GraphEditor,
    id: Id,
}

impl<'a> GraphView<'a> {
    pub fn new(id: impl Hash, graph: &'a mut GraphEditor) -> Self {
        Self {
            graph,
            id: Id::new(id),
        }
    }
}

impl Widget for GraphView<'_> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_at_least(ui.available_size(), Sense::click_and_drag());
        let painter = ui.painter().with_clip_rect(rect);

        let radius = 15.0;
        let offset = 1.5 * radius;

        let (response, input) = self.handle_input(ui, response, rect, radius, offset);

        let mut shapes = Vec::new();
        self.draw_graph(ui, &mut shapes, radius, offset, input);

        painter.extend(shapes);
        response
    }
}

impl GraphView<'_> {
    fn draw_graph(
        &self,
        ui: &Ui,
        shapes: &mut Vec<Shape>,
        radius: f32,
        offset: f32,
        input: InputResult,
    ) {
        let fill_color = ui.style().visuals.warn_fg_color;
        let stroke = ui.style().visuals.text_color();
        let stroke = Stroke::new(1.0, stroke);
        let hover_stroke = ui.style().visuals.widgets.hovered.bg_stroke;
        let font = TextStyle::Small.resolve(ui.style());
        let text_color = ui.style().visuals.extreme_bg_color;

        for (id, node) in self.graph.nodes() {
            let hovered = input.hovered.map_or(false, |hovered| hovered == id);

            let name_pos = if node.focused {
                let pos = input.bounds.offset(node.pos) - Vec2::splat(radius);
                let size = node.size;
                let rounding = Rounding::same(5.0);
                let rect = Rect::from_min_size(pos, size);

                shapes.push(Shape::rect_filled(rect, rounding, fill_color));

                if hovered {
                    shapes.push(Shape::rect_stroke(rect, rounding, hover_stroke));
                }

                pos + Vec2::splat(2.0 * rounding.nw)
            } else {
                let center = input.bounds.offset(node.pos);
                shapes.push(Shape::circle_filled(center, radius, fill_color));

                if hovered {
                    shapes.push(Shape::circle_stroke(center, radius, hover_stroke));
                }

                center
            };

            ui.fonts(|fonts| {
                shapes.push(Shape::text(
                    fonts,
                    name_pos,
                    Align2::CENTER_CENTER,
                    &node.name,
                    font.clone(),
                    text_color,
                ));
            });
        }

        for edge in self.graph.edges() {
            let start = input.bounds.offset(edge.from.pos);
            let end = input.bounds.offset(edge.to.pos);

            let delta = end - start;
            let start = start + offset * delta.normalized();
            let end = end - radius * delta.normalized();

            shapes.push(Shape::line_segment([start, end], stroke));
        }

        if let (Some(from), Some(pointer)) = (
            input.edge_from.and_then(|node| self.graph.get_node(node)),
            input.cursor,
        ) {
            let start = input.bounds.offset(from.pos);
            let delta = pointer - start;
            let start = start + offset * delta.normalized();

            shapes.push(Shape::line_segment([start, pointer], stroke));
        }
    }

    fn handle_input(
        &mut self,
        ui: &Ui,
        response: Response,
        rect: Rect,
        actual_radius: f32,
        select_radius: f32,
    ) -> (Response, InputResult) {
        self.update_and_save_state(ui, |this, state| {
            let radius = select_radius * select_radius;

            state.cursor = response.hover_pos().or(state.cursor);

            let mut hovered_node = None;
            for (id, node) in this.graph.nodes_mut() {
                if state.dragged.map_or(false, |dragged| id == dragged) {
                    node.pos += response.drag_delta();
                }

                if let Some(pos) = state.cursor {
                    let node_pos = state.bounds.offset(node.pos);

                    let over_this = if node.focused {
                        let select = Vec2::splat(select_radius);
                        let min = node_pos - select;
                        let size = node.size + 2.0 * (select - Vec2::splat(actual_radius));
                        let rect = Rect::from_min_size(min, size);

                        rect.contains(pos)
                    } else {
                        (node_pos - pos).length_sq() <= radius
                    };

                    if over_this {
                        hovered_node = Some((id, node));

                        if response.drag_started() {
                            state.dragged = Some(id);
                        }
                    }
                }
            }

            if response.drag_released() {
                state.dragged = None;
            }

            if response.dragged() && state.dragged.is_none() {
                state.bounds.translate(response.drag_delta());
            }

            if response.double_clicked() {
                if let Some((_, clicked)) = &mut hovered_node {
                    clicked.focused = !clicked.focused;
                }
            }

            let hovered_node = hovered_node.map(|(id, _)| id);

            if response.clicked() {
                if let Some(clicked) = hovered_node {
                    if ui.input(|input| input.modifiers.shift_only()) {
                        state.edge = Some(clicked);
                    } else if let Some(from) = state.edge {
                        this.graph.add_edge(from, clicked);
                        state.edge = None;
                    }
                } else {
                    state.edge = None;
                }
            }

            let pos = state
                .bounds
                .undo(state.cursor.unwrap_or_else(|| rect.center()));

            let response = response.context_menu(|ui| {
                // FIXME: the `hovered_node` here updates as the cursor moves,
                // kind of invalidating the context menu. It should probably be
                // kept track of between frames.
                if let Some(id) = hovered_node {
                    if ui.button("Remove").clicked() {
                        this.graph.remove_node(id);
                        state.remove(id);

                        ui.close_menu();
                    }
                } else if ui.button("Add").clicked() {
                    this.graph.add_node("added".into(), pos);
                    ui.close_menu();
                }
            });

            let input = InputResult {
                hovered: hovered_node,
                edge_from: state.edge,
                cursor: state.cursor,
                bounds: state.bounds,
            };

            (response, input)
        })
    }

    fn update_and_save_state<F, R>(&mut self, ui: &Ui, f: F) -> R
    where
        F: FnOnce(&mut Self, &mut State) -> R,
    {
        let mut state = ui
            .ctx()
            .memory_mut(|mem| mem.data.get_temp(self.id).unwrap_or_default());

        let result = f(self, &mut state);

        ui.ctx()
            .memory_mut(|mem| mem.data.insert_temp(self.id, state));

        result
    }
}

struct InputResult {
    hovered: Option<NodeId>,
    edge_from: Option<NodeId>,
    cursor: Option<Pos2>,
    bounds: Bounds,
}

#[derive(Clone, Copy, Default)]
struct State {
    bounds: Bounds,
    dragged: Option<NodeId>,
    cursor: Option<Pos2>,
    edge: Option<NodeId>,
}

impl State {
    pub fn remove(&mut self, node: NodeId) {
        if self.dragged == Some(node) {
            self.dragged = None;
        }

        if self.edge == Some(node) {
            self.edge = None;
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Bounds {
    top_left: Vec2,
}

impl Bounds {
    fn offset(&self, pos: Pos2) -> Pos2 {
        pos + self.top_left
    }

    /// Undo a translation of these bounds.
    fn undo(&self, pos: Pos2) -> Pos2 {
        pos - self.top_left
    }

    fn translate(&mut self, delta: Vec2) {
        self.top_left += delta;
    }
}
