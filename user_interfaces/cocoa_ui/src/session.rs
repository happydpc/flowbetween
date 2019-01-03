use super::cocoa_ui::*;

use flo_ui::*;
use flo_stream::*;
use flo_cocoa_pipe::*;

use futures::*;
use futures::executor;
use futures::executor::Spawn;

use cocoa::base::{id, nil};
use objc::rc::*;
use objc::runtime::*;

use std::sync::*;
use std::collections::HashMap;

#[link(name = "Foundation", kind = "framework")]
extern {
    pub static NSDefaultRunLoopMode: id;
    pub static NSModalPanelRunLoopMode: id;
    pub static NSEventTrackingRunLoopMode: id;
}

///
/// Basis class for a Cocoa session
///
pub struct CocoaSession {
    /// Reference to the FloControl we'll relay the stream via
    target_object: StrongPtr,

    /// Maps IDs to windows
    windows: HashMap<usize, StrongPtr>,

    /// Maps IDs to views
    views: HashMap<usize, StrongPtr>,

    /// Publisher where we send the actions to
    action_publisher: Publisher<Vec<AppAction>>,

    /// The stream of actions for this session (or None if we aren't monitoring for actions)
    actions: Option<Spawn<Subscriber<Vec<AppAction>>>>,

    /// The event publisher for this session
    events: Spawn<Publisher<Vec<AppEvent>>>
}

///
/// Object to notify when it's time to drain the action stream again
///
struct CocoaSessionNotify {
    notify_object: Mutex<NotifyRef>
}

///
/// Reference to an object to be notified
///
struct NotifyRef {
    target_object: WeakPtr
}

impl CocoaSession {
    ///
    /// Creates a new CocoaSession
    ///
    pub fn new(obj: &StrongPtr) -> CocoaSession {
        CocoaSession {
            target_object:      obj.clone(),
            windows:            HashMap::new(),
            views:              HashMap::new(),
            actions:            None,
            action_publisher:   Publisher::new(1),
            events:             executor::spawn(Publisher::new(20))
        }
    }

    ///
    /// Creates a user interface implementation for this session
    ///
    pub fn create_user_interface(&mut self) -> impl UserInterface<Vec<AppAction>, Vec<AppEvent>, ()> {
        // Start listening for actions if we aren't already, by spawning a subscriber to our publisher
        if self.actions.is_none() {
            self.actions = Some(executor::spawn(self.action_publisher.subscribe()));
            self.start_listening();
        }

        // Create the subscriber to receive events sent from the user interface
        let action_publisher    = self.action_publisher.republish();
        let events              = self.events.get_ref().republish();

        // Generate a cocoa user interface
        CocoaUserInterface::new(action_publisher, events)
    }

    ///
    /// Listens for actions from the specified stream
    ///
    fn start_listening(&mut self) {
        unsafe {
            autoreleasepool(|| {
                // Wake up the object on the main thread
                msg_send!(*self.target_object, performSelectorOnMainThread: sel!(actionStreamReady) withObject: nil waitUntilDone: NO);
            });
        }
    }

    ///
    /// Drains any pending messages from the actions stream
    ///
    pub fn drain_action_stream(&mut self) {
        autoreleasepool(move || {
            // Create the object to notify when there's an update
            let notify = Arc::new(CocoaSessionNotify::new(&self.target_object));

            // Drain the stream until it's empty or it blocks
            loop {
                let next = self.actions
                    .as_mut()
                    .map(|actions| actions.poll_stream_notify(&notify, 0))
                    .unwrap_or_else(|| Ok(Async::NotReady));

                match next {
                    Ok(Async::NotReady)     => { break; }
                    Ok(Async::Ready(None))  => {
                        // Session has finished
                        break;
                    }

                    Ok(Async::Ready(Some(actions))) => {
                        for action in actions {
                            // Perform the action
                            self.dispatch_app_action(action);
                        }
                    }

                    Err(_) => {
                        // Action stream should never produce any errors
                        unimplemented!("Action stream should never produce any errors")
                    }
                }
            }
        });
    }

    ///
    /// Performs an application action on this object
    ///
    pub fn dispatch_app_action(&mut self, action: AppAction) {
        use self::AppAction::*;

        match action {
            CreateWindow(window_id)             => { self.create_window(window_id); }
            Window(window_id, window_action)    => { self.windows.get(&window_id).map(|window| self.dispatch_window_action(window, window_action)); }
            CreateView(view_id, view_type)      => { self.create_view(view_id, view_type); },
            DeleteView(view_id)                 => { self.delete_view(view_id); }
            View(view_id, view_action)          => { self.views.get(&view_id).map(|view| self.dispatch_view_action(view, view_action)); }
        }
    }

    ///
    /// Creates a new window and assigns the specified ID to it
    ///
    fn create_window(&mut self, new_window_id: usize) {
        unsafe {
            // Fetch the window class to create
            let window_class = (**self.target_object).get_ivar::<*mut Class>("_windowClass");

            // Allocate and initialise it
            let window: *mut Object = msg_send!(*window_class, alloc);
            let window = msg_send!(window, init);
            let window = StrongPtr::new(window);

            // Store it away
            self.windows.insert(new_window_id, window);
        }
    }

    ///
    /// Dispatches an action to the specified window
    ///
    fn dispatch_window_action(&self, window: &StrongPtr, action: WindowAction) {
        use self::WindowAction::*;

        unsafe {
            match action {
                Open                    => { msg_send!((**window), windowOpen); }
                SetRootView(view_id)    => { self.views.get(&view_id).map(|view| msg_send!((**window), windowSetRootView: view)); }
            }
        }
    }

    ///
    /// Creates a new view and assigns the specified ID to it
    ///
    fn create_view(&mut self, new_view_id: usize, view_type: ViewType) {
        use self::ViewType::*;

        unsafe {
            // Fetch the view class to create
            let view_class = (**self.target_object).get_ivar::<*mut Class>("_viewClass");

            // Allocate and initialise it
            let view: *mut Object = msg_send!(*view_class, alloc);

            let view = match view_type {
                Empty => msg_send!(view, initAsEmpty)
            };

            let view = StrongPtr::new(view);

            // Store it away
            self.views.insert(new_view_id, view);
        }

    }

    ///
    /// Removes a view from this object
    ///
    pub fn delete_view(&mut self, old_view_id: usize) {
        self.views.remove(&old_view_id);
    }

    ///
    /// Dispatches an action to the specified view
    ///
    fn dispatch_view_action(&self, view: &StrongPtr, action: ViewAction) {
        use self::ViewAction::*;

        unsafe {
            match action {
                RemoveFromSuperview     => { msg_send!((**view), viewRemoveFromSuperview); }
                AddSubView(view_id)     => { self.views.get(&view_id).map(|subview| msg_send!((**view), viewAddSubView: subview)); }
            }
        }

    }
}

/// WeakPtr is not Send because Object is not Send... but we need to be able to send objective-C objects between threads so
/// we can schedule on the main thread and they are thread-safe at least in objective C itself, so let's assume this is
/// an oversight for now.
unsafe impl Send for CocoaSession { }
unsafe impl Send for NotifyRef { }

impl CocoaSessionNotify {
    ///
    /// Creates a notifier for the specified object
    ///
    pub fn new(obj: &StrongPtr) -> CocoaSessionNotify {
        CocoaSessionNotify {
            notify_object: Mutex::new(
                NotifyRef { target_object: obj.weak() }
            )
        }
    }
}

impl executor::Notify for CocoaSessionNotify {
    fn notify(&self, _: usize) {
        // Load the target object
        let target_object = self.notify_object.lock().unwrap();

        // If it still exists, send the message to the object on the main thread
        unsafe {
            autoreleasepool(move || {
                let target_object = target_object.target_object.load();

                if *target_object != nil {
                    let modes: *mut Object  = msg_send!(class!(NSMutableArray), alloc);
                    let modes               = msg_send!(modes, init);
                    let modes               = StrongPtr::new(modes);

                    msg_send!(*modes, addObject: NSDefaultRunLoopMode);
                    msg_send!(*modes, addObject: NSModalPanelRunLoopMode);
                    msg_send!(*modes, addObject: NSEventTrackingRunLoopMode);

                    msg_send![*target_object, performSelectorOnMainThread: sel!(actionStreamReady) withObject: nil waitUntilDone: NO modes: modes];
                }
            });
        }
    }
}
