use super::layout::*;
use super::widget::*;
use super::custom_style::*;
use super::super::gtk_event::*;
use super::super::gtk_action::*;
use super::super::gtk_thread::*;
use super::super::gtk_event_parameter::*;
use super::super::gtk_widget_event_type::*;

use flo_ui::*;

use gtk;
use gtk::prelude::*;

use std::rc::*;
use std::cell::*;

///
/// Represents a basic widget
/// 
pub struct BasicWidget(pub WidgetId, pub gtk::Widget);

impl BasicWidget {
    ///
    /// Creates a basic widget
    /// 
    pub fn new<Src: Cast+IsA<gtk::Widget>>(id: WidgetId, widget: Src) -> BasicWidget {
        BasicWidget(id, widget.upcast::<gtk::Widget>())
    }
}

impl GtkUiWidget for BasicWidget {
    fn id(&self) -> WidgetId {
        let BasicWidget(id, ref _widget) = *self;
        id
    }

    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        process_basic_widget_action(self, flo_gtk, action);
    }

    fn set_children(&mut self, children: Vec<Rc<RefCell<GtkUiWidget>>>) {
        let BasicWidget(_id, ref widget) = *self;

        // If this widget is a container, add this as a child widget
        let container = widget.clone().dynamic_cast::<gtk::Container>();
        if let Ok(container) = container {
            // Remove any existing child widgets
            container.get_children().iter().for_each(|child| container.remove(child));

            for new_child in children {
                // Remove the child widget from its existing parent
                let new_child = new_child.borrow();
                let new_child = new_child.get_underlying();

                new_child.unparent();

                // Add to the container
                container.add(new_child);
            }
        }
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        let BasicWidget(_id, ref widget) = *self;
        widget
    }
}

///
/// Performs the basic processing associated with a widget action (using a generic Gtk widget as the target)
/// 
pub fn process_basic_widget_action<W: GtkUiWidget>(widget: &mut W, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
    use self::GtkWidgetAction::*;

    match action {
        &Layout(ref layout)                         => process_basic_widget_layout(widget.id(), widget.get_underlying(), flo_gtk, layout),
        &RequestEvent(event_type, ref action_name)  => process_basic_event_request(widget, flo_gtk, event_type, action_name),
        &Content(ref content)                       => process_basic_widget_content(widget, flo_gtk, content),
        &Appearance(ref appearance)                 => process_basic_widget_appearance(widget, flo_gtk, appearance),
        &State(ref state)                           => process_basic_widget_state(widget, flo_gtk, state),
        &Font(ref font)                             => process_basic_widget_font(widget, flo_gtk, font),
        &Scroll(ref scroll)                         => process_basic_widget_scroll(widget.get_underlying(), flo_gtk, scroll),

        &New(_widget_type)                          => (),
        &Delete                                     => { widget.get_underlying().unparent(); },

        &SetRoot(window_id)                         => { 
            let widget = widget.get_underlying().clone();
            flo_gtk.get_window(window_id).map(|window| window.borrow_mut().set_root(flo_gtk, &widget));
        },

        &Box                                        => { }
    }
}

///
/// Processes a layout command for a widget being managed by FlowBetween
/// 
pub fn process_basic_widget_layout<W: WidgetExt>(id: WidgetId, widget: &W, flo_gtk: &mut FloGtk, layout: &WidgetLayout) { 
    // Fetch or create the layout for this widget
    let widget_data     = flo_gtk.widget_data();
    let widget_layout   = widget_data.get_widget_data_or_insert(id, || Layout::new());

    // Update it with the content of the command
    widget_layout.map(move |widget_layout| widget_layout.borrow_mut().update(layout));

    // Tell the parent of this widget it needs relayout
    widget.get_parent().map(|parent| parent.queue_resize());
}

///
/// Performs the actions required to set a widget's parent
/// 
pub fn set_widget_parent<W: GtkUiWidget>(widget: &mut W, children: &Vec<WidgetId>, flo_gtk: &mut FloGtk) {
    // Fetch the widget information
    let widget_data     = flo_gtk.widget_data();
    let children        = children.iter()
        .map(|child_id| widget_data.get_widget(*child_id))
        .filter(|child| !child.is_none())
        .map(|child| child.unwrap())
        .collect();
    
    widget.set_children(children);
}

///
/// Processes a content command for a widget being managed by FlowBetween
/// 
pub fn process_basic_widget_content<W: GtkUiWidget>(widget: &mut W, flo_gtk: &mut FloGtk, content: &WidgetContent) {
    use self::WidgetContent::*;

    match content {
        &SetChildren(ref children)      => set_widget_parent(widget, children, flo_gtk),
        &SetText(ref _text)             => () /* Standard gtk widgets can't have text in them */,
        &Draw(ref canvas)               => unimplemented!(),

        &AddClass(ref class_name)       => {
            let widget          = widget.get_underlying();
            let style_context   = widget.get_style_context();
            style_context.map(|context| context.add_class(&*class_name));
        },

        &RemoveClass(ref class_name)    => {
            let widget          = widget.get_underlying();
            let style_context   = widget.get_style_context();
            style_context.map(|context| context.remove_class(&*class_name));
        }
    }
}

///
/// Generic appearance command for a widget being managed by FlowBetween
/// 
pub fn process_basic_widget_appearance<W: GtkUiWidget>(widget: &W, flo_gtk: &mut FloGtk, appearance: &Appearance) {
    use self::Appearance::*;

    match appearance {
        &Foreground(ref color)      => {
            let custom_style = flo_gtk.widget_data().get_custom_style(widget);
            custom_style.borrow_mut().set_foreground(color);
        },

        &Background(ref color)      => {
            let custom_style = flo_gtk.widget_data().get_custom_style(widget);
            custom_style.borrow_mut().set_background(color);
        },

        &Image(ref image)           => ()
    }
}

///
/// Processes a basic state command for a widget being managed by FlowBetween
/// 
pub fn process_basic_widget_state<W: GtkUiWidget>(widget: &W, flo_gtk: &mut FloGtk, state: &WidgetState) {
    use self::WidgetState::*;

    match state {
        &SetSelected(selected)      => { widget.get_underlying().clone().dynamic_cast::<gtk::ToggleButton>().ok().map(|toggle| { toggle.set_active(selected); }); },
        &SetBadged(badged)          => (),
        &SetValueFloat(value)       => (),
        &SetRangeMin(from)          => (),
        &SetRangeMax(to)            => ()
    }
}

///
/// Processes a font command for a widget being managed by FlowBetween
/// 
pub fn process_basic_widget_font<W: GtkUiWidget>(widget: &W, flo_gtk: &mut FloGtk, font: &Font) {
    use self::Font::*;

    match font {
        &Align(_align)          => (),
        &Size(size_pixels)      => {
            let custom_style = flo_gtk.widget_data().get_custom_style(widget);
            custom_style.borrow_mut().set_font_size(size_pixels);
        },
        &Weight(weight)         =>  {
            let custom_style = flo_gtk.widget_data().get_custom_style(widget);
            custom_style.borrow_mut().set_font_weight(weight as u32);
        }
    }
}

///
/// Processes a scroll command for a widget
/// 
pub fn process_basic_widget_scroll<W: WidgetExt>(widget: &W, flo_gtk: &mut FloGtk, scroll: &Scroll) {
    use self::Scroll::*;

    match scroll {
        &MinimumContentSize(width, height)      => (),
        &HorizontalScrollBar(ref visibility)    => (),
        &VerticalScrollBar(ref visibility)      => (),
        &Fix(ref axis)                          => ()
    }
}

///
/// Performs the actions associated with basic event registration for a widget
/// 
pub fn process_basic_event_request<W: GtkUiWidget>(widget: &W, flo_gtk: &mut FloGtk, event_type: GtkWidgetEventType, action_name: &String) {
    use self::GtkWidgetEventType::*;
    use self::GtkEvent::Event;
        
    let widget_id   = widget.id();
    let action_name = action_name.clone();
    let event_sink  = RefCell::new(flo_gtk.get_event_sink());

    match event_type {
        Click => {
            // For basic widgets with no explicit click action, we just detect the button press event
            widget.get_underlying()
                .connect_button_press_event(move |_, button| { 
                    if button.get_state().is_empty() && button.get_button() == 1 {
                        // Left mouse button down with no modifiers = click
                        event_sink.borrow_mut().start_send(Event(widget_id, action_name.clone(), GtkEventParameter::None)).unwrap();
                        Inhibit(true)
                    } else if button.get_button() == 1 {
                        // Not a click but we stil want to inhibit actions here
                        Inhibit(true)
                    } else { 
                        // Other button down = continue with other event handlers
                        Inhibit(false) 
                    } 
                }); 
            }
    }
}
