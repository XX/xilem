// Copyright 2025 the Xilem Authors
// SPDX-License-Identifier: Apache-2.0

//! A list of widgets implementing the [`Layer`](crate::core::Layer) trait.

#![expect(
    missing_debug_implementations,
    reason = "Widgets are not expected to implement Debug"
)]

mod selector_menu;
mod tooltip;

pub use selector_menu::*;
pub use tooltip::*;

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use masonry_testing::{ModularWidget, Record, Recorder, Recording, TestHarness};

    use crate::core::{Handled, NewWidget, PointerEvent, Widget};
    use crate::theme::default_property_set;
    use crate::widgets::Button;

    /// A layer root over one recording child, suppressing pointer events when told to.
    fn harness(
        suppress: &Rc<Cell<bool>>,
    ) -> (
        TestHarness<ModularWidget<crate::core::WidgetPod<dyn Widget>>>,
        Recording,
    ) {
        let recording = Recording::default();
        let child = Recorder::new(Button::with_text("hello"), &recording);
        let suppress = suppress.clone();
        let layer = ModularWidget::new_parent(NewWidget::new(child).erased())
            .capture_pointer_event_fn(move |_child, _ctx, _props, _event| {
                if suppress.get() {
                    Handled::Yes
                } else {
                    Handled::No
                }
            });
        let mut harness = TestHarness::create_with_size(
            default_property_set(),
            NewWidget::new(layer),
            (200, 100),
        );
        let _ = harness.redraw();
        let _ = recording.drain();
        (harness, recording)
    }

    fn saw_pointer_events(recording: &Recording) -> bool {
        recording
            .drain()
            .iter()
            .any(|record| matches!(record, Record::PointerEvent(_)))
    }

    #[test]
    fn a_layer_can_keep_a_pointer_event_from_the_tree() {
        let suppress = Rc::new(Cell::new(true));
        let (mut harness, recording) = harness(&suppress);
        let _ = recording.drain();

        let handled = harness.mouse_move((5., 5.));

        assert_eq!(
            handled,
            Handled::Yes,
            "the driver is told the event was used"
        );
        assert!(
            !saw_pointer_events(&recording),
            "no widget sees an event a layer suppressed"
        );
    }

    #[test]
    fn a_layer_that_does_not_suppress_changes_nothing() {
        let suppress = Rc::new(Cell::new(false));
        let (mut harness, recording) = harness(&suppress);
        let _ = recording.drain();

        harness.mouse_move((5., 5.));

        assert!(
            saw_pointer_events(&recording),
            "the tree still gets what no layer took"
        );
    }

    #[test]
    fn a_captured_pointer_is_not_withheld() {
        let suppress = Rc::new(Cell::new(false));
        let (mut harness, recording) = harness(&suppress);

        // The button captures the pointer on the press.
        harness.mouse_move((5., 5.));
        harness.mouse_button_press(None);
        let _ = recording.drain();

        // From here on the layer wants everything, and gets nothing: the widget holding
        // the pointer has to be told how its gesture ends.
        suppress.set(true);
        harness.mouse_move((7., 5.));
        harness.mouse_button_release(None);

        assert!(
            saw_pointer_events(&recording),
            "pointer capture outranks a layer"
        );
    }

    #[test]
    fn suppression_does_not_strand_hover() {
        let suppress = Rc::new(Cell::new(true));
        let (mut harness, recording) = harness(&suppress);

        harness.mouse_move((5., 5.));
        let _ = recording.drain();

        // Hover is computed from the pointer position rather than from the event pass, so
        // it follows the pointer even while the events themselves are withheld.
        assert!(
            harness
                .root_widget()
                .children()
                .first()
                .is_some_and(|child| child.ctx().is_hovered()),
            "the widget under a withheld pointer is still hovered"
        );
        assert!(matches!(&*event_kinds(&recording), []));
    }

    #[test]
    fn a_layer_that_asks_for_layout_gets_one() {
        let recording = Recording::default();
        let child = Recorder::new(Button::with_text("hello"), &recording);
        let layer = ModularWidget::new_parent(NewWidget::new(child).erased())
            .capture_pointer_event_fn(move |child, ctx, _props, _event| {
                // A layer root handling an event is an event handler like any other, and
                // what it touches is usually a child rather than itself. Those flags have
                // to reach the root, or the frame the layer asked for never runs.
                let (_, mut child_ctx) = ctx.get_raw(child);
                child_ctx.request_layout();
                Handled::Yes
            });
        let mut harness = TestHarness::create_with_size(
            default_property_set(),
            NewWidget::new(layer),
            (200, 100),
        );
        let _ = harness.redraw();
        let _ = recording.drain();

        // Twice to the same spot, away from the button: the first move settles hover and
        // whatever it changes, so that the second one can only cause a pass through what
        // the layer asked for.
        harness.mouse_move((190., 90.));
        let _ = harness.redraw();
        let _ = recording.drain();

        harness.mouse_move((190., 90.));
        let _ = harness.redraw();

        assert!(
            recording
                .drain()
                .iter()
                .any(|record| matches!(record, Record::Measure(_) | Record::Layout(_))),
            "the layout pass the layer asked for actually runs"
        );
    }

    fn event_kinds(recording: &Recording) -> Vec<PointerEvent> {
        recording
            .drain()
            .into_iter()
            .filter_map(|record| match record {
                Record::PointerEvent(event) => Some(event),
                _ => None,
            })
            .collect()
    }
}
