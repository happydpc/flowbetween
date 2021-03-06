use super::action::*;

use flo_ui::*;
use flo_canvas::*;

use itertools::*;

use std::iter;
use std::collections::HashMap;

///
/// Describes the canvases attached to a particular controller
///
pub struct CanvasModel {
    /// The canvas attached to the specified view
    canvas_for_view: HashMap<usize, Resource<BindingCanvas>>,

    /// The views that should receive updates for a particular canvas
    views_with_canvas: HashMap<String, Vec<usize>>
}

impl CanvasModel {
    ///
    /// Creates a new canvas model with no canvases in it
    ///
    pub fn new() -> CanvasModel {
        CanvasModel {
            canvas_for_view: HashMap::new(),
            views_with_canvas: HashMap::new()
        }
    }

    ///
    /// Retrieves the name of a canvas as a string
    ///
    pub fn name_for_canvas(canvas: &Resource<BindingCanvas>) -> String {
        if let Some(name) = canvas.name() {
            name
        } else {
            format!("{}", canvas.id())
        }
    }

    ///
    /// Associates a canvas with a particular view ID
    ///
    pub fn set_canvas_for_view(&mut self, view_id: usize, canvas: Resource<BindingCanvas>) {
        let canvas_name = Self::name_for_canvas(&canvas);

        self.canvas_for_view.insert(view_id, canvas);
        self.views_with_canvas.entry(canvas_name)
            .or_insert_with(|| vec![])
            .push(view_id);
    }

    ///
    /// Retrieves the actions to perform for an update on a canvas that (might be) in this model
    ///
    pub fn actions_for_update(&self, canvas_name: String, actions: Vec<Draw>) -> impl Iterator<Item=AppAction> {
        if let Some(views) = self.views_with_canvas.get(&canvas_name) {
            // Supply the actions to each view
            Either::Left(
                if views.len() == 1 {
                    // No need to clone the actions
                    Either::Left(iter::once(AppAction::View(views[0], ViewAction::Draw(actions))))
                } else {
                    // Each view needs its own set of drawing actions
                    Either::Right(views.clone()
                        .into_iter()
                        .map(move |view_id| AppAction::View(view_id, ViewAction::Draw(actions.clone()))))
                })
        } else {
            // No views attached to this canvas
            Either::Right(iter::empty())
        }
    }

    ///
    /// Removes a view from the canvas model
    ///
    pub fn remove_view(&mut self, view_id: usize) {
        if let Some(canvas_name) = self.canvas_for_view.get(&view_id).map(|canvas| Self::name_for_canvas(canvas)) {
            // Remove the association with the view ID
            self.canvas_for_view.remove(&view_id);

            // Remove from the list of views that use this canvas
            self.views_with_canvas.get_mut(&canvas_name)
                .map(|views| views.retain(|view_with_canvas| view_with_canvas != &view_id));
        }
    }
}
