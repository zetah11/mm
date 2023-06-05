//! A plate is an infinite canvas that can be dragged around and where arbitrary
//! entities can be positioned and interacted with.

use std::hash::Hash;

use egui::{
    vec2, Area, Context, CursorIcon, Id, Pos2, Rect, Response, Rounding, Sense, Shape, Style, Ui,
    Vec2,
};

/// A course is put on a plate.
///
/// Courses consist of *meals* and *decoration*. Meals are the main thing that's
/// displayed, and can be selected, dragged around, and generally interacted
/// with. Decorations are visual things drawn below (`background`) or above
/// (`decoration`) the main meals.
pub trait Course {
    type MealId: Clone + Eq + Hash + Send + Sync + 'static;

    fn meals(&self) -> Vec<(Self::MealId, Meal)>;

    /// Returns `true` if anything changed by this meal being dragged.
    fn drag(&mut self, _meal: &Self::MealId, _delta: Vec2) -> bool {
        false
    }

    fn hover(&mut self, _meal: Option<&Self::MealId>) {}

    fn background(&self, _style: &Style, _shapes: &mut Vec<Shape>, _bounds: Bounds) {}
    fn decoration(&self, _style: &Style, _shapes: &mut Vec<Shape>, _bounds: Bounds) {}
    fn draw_meal(&self, _id: &Self::MealId, _ui: &mut Ui) {}
}

#[derive(Clone, Copy, Debug)]
pub struct Meal {
    pub top_left: Pos2,
    pub size: Vec2,
}

#[derive(Clone, Copy, Debug)]
pub struct Bounds {
    pub top_left: Vec2,
    pub visible: Rect,
}

impl Bounds {
    /// Transform "absolute" coordinates to screen-space coordinates correctly
    /// offset.
    pub fn apply(&self, pos: Pos2) -> Pos2 {
        pos + self.top_left + self.visible.min.to_vec2()
    }
}

#[derive(Debug)]
pub struct Plate<C> {
    id: Id,
    phantom: std::marker::PhantomData<C>,
}

impl<C: Course> Plate<C> {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn show(mut self, ui: &mut Ui, course: &mut C) -> Response {
        let (rect, response) = ui.allocate_at_least(ui.available_size(), Sense::click_and_drag());

        let meals = course.meals();

        let mut state = PlateState::load(ui.ctx(), self.id);

        let response = self.handle_input(ui.ctx(), course, &meals, rect, &mut state, response);
        self.draw(ui, course, meals, &state, rect);

        state.save(ui.ctx(), self.id);
        response
    }

    fn draw(
        &self,
        ui: &mut Ui,
        course: &mut C,
        meals: Vec<(C::MealId, Meal)>,
        state: &PlateState<C::MealId>,
        rect: Rect,
    ) {
        let painter = ui.painter().with_clip_rect(rect);

        let fill_color = ui.visuals().error_fg_color;
        let hover_stroke = ui.visuals().widgets.hovered.fg_stroke;
        let rounding = Rounding::same(3.0);

        let mut shapes = Vec::new();

        let bounds = Bounds {
            top_left: state.top_left,
            visible: rect,
        };

        course.background(ui.style(), &mut shapes, bounds);

        for (id, meal) in meals.iter() {
            let min = state.apply(rect, meal.top_left);
            let rect = Rect::from_min_size(min, meal.size);
            shapes.push(Shape::rect_filled(rect, rounding, fill_color));

            if state.hovered.as_ref() == Some(id) {
                shapes.push(Shape::rect_stroke(rect, rounding, hover_stroke));
            }
        }

        course.decoration(ui.style(), &mut shapes, bounds);

        painter.extend(shapes);

        for (id, meal) in meals.iter() {
            let min = state.apply(rect, meal.top_left);
            Area::new(self.id.with(id))
                .fixed_pos(min)
                .show(ui.ctx(), |ui| course.draw_meal(id, ui));
        }
    }

    fn handle_input(
        &mut self,
        ctx: &Context,
        course: &mut C,
        meals: &[(C::MealId, Meal)],
        rect: Rect,
        state: &mut PlateState<C::MealId>,
        response: Response,
    ) -> Response {
        state.cursor = response.hover_pos().or(state.cursor);
        state.hovered = None;

        for (id, meal) in meals {
            if let Some(cursor) = state.cursor {
                let min = state.apply(rect, meal.top_left);
                let rect = Rect::from_min_size(min, meal.size);

                if rect.contains(cursor) {
                    if response.drag_started() {
                        state.dragged = Some(id.clone());
                    }

                    state.hovered = Some(id.clone());
                }
            }
        }

        if response.dragged() {
            let drag_plate = if let Some(dragged) = &state.dragged {
                !course.drag(dragged, response.drag_delta())
            } else {
                true
            };

            if drag_plate {
                state.top_left += response.drag_delta();
                state.dragged = None;
            }

            ctx.set_cursor_icon(CursorIcon::Grabbing);
        }

        if response.hovered() {
            course.hover(state.hovered.as_ref());
        }

        if response.drag_released() {
            state.dragged = None;
        }

        response
    }
}

#[derive(Clone, Debug)]
struct PlateState<I> {
    top_left: Vec2,
    cursor: Option<Pos2>,
    dragged: Option<I>,
    hovered: Option<I>,
}

impl<I> Default for PlateState<I> {
    fn default() -> Self {
        Self {
            top_left: vec2(0.0, 0.0),
            cursor: None,
            dragged: None,
            hovered: None,
        }
    }
}

impl<I: Clone + Send + Sync + 'static> PlateState<I> {
    /// Load a `PlateState` from the given context. Does not modify the current
    /// memory.
    fn load(ctx: &Context, id: Id) -> Self {
        ctx.memory_mut(|mem| mem.data.get_temp(id).unwrap_or_default())
    }

    fn save(self, ctx: &Context, id: Id) {
        ctx.memory_mut(|mem| mem.data.insert_temp(id, self));
    }

    fn apply(&self, rect: Rect, pos: Pos2) -> Pos2 {
        pos + rect.min.to_vec2() + self.top_left
    }
}
