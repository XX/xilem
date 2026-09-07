// Copyright 2025 the Xilem Authors
// SPDX-License-Identifier: Apache-2.0

use crate::core::Widget;

use crate::core::{EventCtx, Handled, PointerEvent, PropertiesMut};

/// The type of a new [`Layer`].
///
/// When adding a new [layer](crate::doc::masonry_concepts#layers) to the app,
/// this tells the app driver what type of item to add.
#[derive(Clone, Debug, Default)]
pub enum LayerType {
    /// A simple tooltip showing some text until the mouse moves.
    Tooltip(String),
    /// A menu showing the different options of a selector widget.
    Selector {
        /// The text of the options.
        options: Vec<String>,
        /// The initially selected option.
        selected_option: usize,
    },
    /// Unknown layer type. Always use the widget fallback.
    #[default]
    Other,
}

/// The trait implemented by widgets which are meant to be at the root of
/// a [layer](crate::doc::masonry_concepts#layers).
pub trait Layer: Widget {
    // TODO - Possible evolutions:
    // - Return flag to remove layer.
    // - Pass layer id to method.

    /// An event handler called for every layer for all pointer events, even those outside the layer's root widget.
    ///
    /// Returning [`Handled::Yes`] keeps the event out of the widget tree: no widget's
    /// [`Widget::on_pointer_event`] is called for it, and the event is reported as handled
    /// to the app driver. Short of holding [pointer capture] - which is only granted during
    /// a press - this is the only way to keep a pointer event from the tree.
    ///
    /// Two things a layer does not decide:
    ///
    /// - **The other layers still see the event.** Suppression hides it from the widget
    ///   tree only, so a layer that dismisses itself on any pointer event still does.
    /// - **Pointer capture wins.** While a widget holds the pointer, the event reaches it
    ///   whatever the layers return, so a widget in the middle of a gesture is always told
    ///   how that gesture ends.
    ///
    /// Hover is tracked from the pointer position rather than from this pass, so
    /// withholding a [`PointerEvent::Move`] does not leave a stale hovered widget behind.
    ///
    /// [pointer capture]: crate::doc::masonry_concepts#pointer-capture
    fn capture_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) -> Handled;
}
