//!
//! # Bindings
//!
//! This provides a means for building data-driven applications. The
//! basic model is similar to how spreadsheets work: we watch what
//! items a particular calculation depends on and generate an event
//! when any of these change.
//!

use std::sync::*;
use std::rc::*;
use std::cell::*;

///
/// Trait implemented by items with dependencies that need to be notified when they have changed
///
pub trait Notifiable : Sync+Send {
    ///
    /// Indicates that a dependency of this object has changed
    ///
    fn mark_as_changed(&self);
}

///
/// Trait implemented by an object that can be released
///
pub trait Releasable {
    ///
    /// Indicates that this object is finished with and should be released
    ///
    fn done(&mut self);
}

///
/// Trait implemented by items that can notify something when they're changed
///
pub trait Changeable {
    ///
    /// Supplies an item to be notified when this item is changed
    ///
    fn when_changed(&mut self, what: Arc<Notifiable>);
}

///
/// Trait implemented by something that is bound to a value
///
pub trait Bound<Value> : Changeable {
    ///
    /// Retrieves the value stored by this binding
    ///
    fn get(&self) -> Value;
}

///
/// Trait implemented by something that is bound to a value that can be changed
///
pub trait MutableBound<Value> : Bound<Value> {
    ///
    /// Sets the value stored by this binding
    ///
    fn set(&mut self, new_value: Value);
}

///
/// A notifiable that can be released (and then tidied up later)
///
#[derive(Clone)]
pub struct ReleasableNotifiable {
    target: Arc<Mutex<RefCell<Option<Arc<Notifiable>>>>>
}

impl ReleasableNotifiable {
    ///
    /// Creates a new releasable notifiable object
    ///
    fn new(target: Arc<Notifiable>) -> ReleasableNotifiable {
        ReleasableNotifiable {
            target: Arc::new(Mutex::new(RefCell::new(Some(target))))
        }
    }
}

impl Releasable for ReleasableNotifiable {
    fn done(&mut self) {
        // Reset the optional item so that it's 'None'
        let lock = self.target.lock().unwrap();

        *lock.borrow_mut() = None;
    }
}

impl Notifiable for ReleasableNotifiable {
    fn mark_as_changed(&self) {
        // Reset the optional item so that it's 'None'
        let lock = self.target.lock().unwrap();

        // Send to the target
        lock.borrow().as_ref().map(|target| target.mark_as_changed());
    }
}

///
/// Represents the dependencies of a binding context
///
#[derive(Clone)]
pub struct BindingDependencies {
    /// The list of changables that are dependent on this context
    dependencies: Rc<RefCell<Vec<Box<Changeable>>>>
}

impl BindingDependencies {
    ///
    /// Creates a new binding dependencies object
    ///
    pub fn new() -> BindingDependencies {
        BindingDependencies { dependencies: Rc::new(RefCell::new(vec![])) }
    }

    ///
    /// Adds a new dependency to this object
    ///
    pub fn add_dependency<TChangeable: Changeable+'static>(&mut self, dependency: TChangeable) {
        self.dependencies.borrow_mut().push(Box::new(dependency))
    }
}

impl Changeable for BindingDependencies {
    fn when_changed(&mut self, what: Arc<Notifiable>) {
        for dep in self.dependencies.borrow_mut().iter_mut() {
            dep.when_changed(what.clone());
        }
    }
}

///
/// Represents a binding context. Binding contexts are
/// per-thread structures, used to track 
///
#[derive(Clone)]
pub struct BindingContext {
    /// The dependencies for this context
    dependencies: BindingDependencies,

    /// None, or the binding context that this context was created within
    nested: Option<Box<BindingContext>>
}

thread_local! {
    static CURRENT_CONTEXT: RefCell<Option<BindingContext>> = RefCell::new(None);
}

impl BindingContext {
    ///
    /// Gets the active binding context
    ///
    pub fn current() -> Option<BindingContext> {
        CURRENT_CONTEXT.with(|current_context| {
            current_context
                .borrow()
                .as_ref()
                .map(|rc| rc.clone())
        })
    }

    ///
    /// Executes a function in a new binding context
    ///
    pub fn bind<TResult, TFn>(to_do: TFn) -> (TResult, BindingDependencies) 
    where TFn: FnOnce() -> TResult {
        // Remember the previous context
        let previous_context = Self::current();

        // Create a new context
        let dependencies    = BindingDependencies::new();
        let new_context     = BindingContext {
            dependencies:   dependencies.clone(),
            nested:         previous_context.clone().map(|ctx| Box::new(ctx))
        };

        // Make the current context the same as the new context
        CURRENT_CONTEXT.with(|current_context| *current_context.borrow_mut() = Some(new_context));

        // Perform the requested action with this context
        let result = to_do();

        // Reset to the previous context
        CURRENT_CONTEXT.with(|current_context| *current_context.borrow_mut() = previous_context);

        (result, dependencies)
    }

    pub fn add_dependency<TChangeable: Changeable+'static>(dependency: TChangeable) {
        Self::current().map(|mut ctx| ctx.dependencies.add_dependency(dependency));
    }
}

struct NotifyFn<TFn> {
    when_changed: Mutex<RefCell<TFn>>
}

impl<TFn> Notifiable for NotifyFn<TFn>
where TFn: Send+FnMut() -> () {
    fn mark_as_changed(&self) {
        let cell            = self.when_changed.lock().unwrap();
        let mut on_changed  = &mut *cell.borrow_mut();
        
        on_changed()
    }
}

///
/// An internal representation of a bound value
///
struct BoundValue<Value> {
    /// The current value of this binding
    value: Value,

    /// What to call when the value changes
    when_changed: Vec<ReleasableNotifiable>
}

impl<Value: Clone+PartialEq> BoundValue<Value> {
    ///
    /// Creates a new binding with the specified value
    ///
    pub fn new(val: Value) -> BoundValue<Value> {
        BoundValue {
            value:          val,
            when_changed:   vec![]
        }
    }

    ///
    /// Updates the value in this structure without calling the notifications, returns whether or not anything actually changed
    ///
    pub fn set_without_notifying(&mut self, new_value: Value) -> bool {
        let changed = self.value != new_value;

        self.value = new_value;

        changed
    }

    ///
    /// Retrieves a copy of the list of notifiable items for this value
    ///
    pub fn get_notifiable_items(&self) -> Vec<ReleasableNotifiable> {
        self.when_changed.clone()
    }
}

impl<Value> Changeable for BoundValue<Value> {
    fn when_changed(&mut self, what: Arc<Notifiable>) {
        self.when_changed.push(ReleasableNotifiable::new(what));
    }
}

impl<Value: Clone> Bound<Value> for BoundValue<Value> {
    fn get(&self) -> Value {
        self.value.clone()
    }
}

impl<Value: Clone+PartialEq> MutableBound<Value> for BoundValue<Value> {
    fn set(&mut self, new_value: Value) {
        if self.set_without_notifying(new_value) {
            for notify in self.when_changed.iter() {
                notify.mark_as_changed();
            }
        }
    }
}

///
/// Represents a thread-safe, sharable binding
///
#[derive(Clone)]
pub struct Binding<Value> {
    /// The value stored in this binding
    value: Arc<Mutex<RefCell<BoundValue<Value>>>>
}

impl<Value: Clone+PartialEq> Binding<Value> {
    fn new(value: Value) -> Binding<Value> {
        Binding {
            value: Arc::new(Mutex::new(RefCell::new(BoundValue::new(value))))
        }
    }
}

impl<Value> Changeable for Binding<Value> {
    fn when_changed(&mut self, what: Arc<Notifiable>) {
        let cell = self.value.lock().unwrap();
        cell.borrow_mut().when_changed(what);
    }
}

impl<Value: 'static+Clone> Bound<Value> for Binding<Value> {
    fn get(&self) -> Value {
        BindingContext::add_dependency(self.clone());

        let cell    = self.value.lock().unwrap();
        let value   = cell.borrow().get();

        value
    }
}

impl<Value: 'static+Clone+PartialEq> MutableBound<Value> for Binding<Value> {
    fn set(&mut self, new_value: Value) {
        // Update the value with the lock held
        let notifications = {
            let cell    = self.value.lock().unwrap();
            let changed = cell.borrow_mut().set_without_notifying(new_value);
        
            if changed {
                cell.borrow().get_notifiable_items()
            } else {
                vec![]
            }
        };

        // Call the notifications outside of the lock
        for to_notify in notifications.into_iter() {
            to_notify.mark_as_changed()
        }
    }
}

///
/// Core representation ofa computed binding
///
struct ComputedBindingCore<Value: 'static+Clone+PartialEq, TFn>
where TFn: 'static+Fn() -> Value {
    /// Function to call to recalculate this item
    calculate_value: TFn,

    /// Most recent cached value
    latest_value: RefCell<Option<Value>>,

    /// What to call when the value changes
    when_changed: Vec<ReleasableNotifiable>
}

impl<Value: 'static+Clone+PartialEq, TFn> ComputedBindingCore<Value, TFn>
where TFn: 'static+Fn() -> Value {
    ///
    /// Creates a new computed binding core item
    ///
    pub fn new(calculate_value: TFn) -> ComputedBindingCore<Value, TFn> {
        ComputedBindingCore {
            calculate_value:    calculate_value,
            latest_value:       RefCell::new(None),
            when_changed:       vec![]
        }
    }

    ///
    /// Marks the value as changed, returning true if the value was removed
    ///
    pub fn mark_changed(&self) -> bool {
        let mut latest_value = self.latest_value.borrow_mut();

        if *latest_value == None {
            false
        } else {
            *latest_value = None;
            true
        }
    }

    ///
    /// Returns the current value (or 'None' if it needs recalculating)
    ///
    pub fn get(&self) -> Option<Value> {
        self.latest_value.borrow().clone()
    }

    ///
    /// Recalculates the latest value
    ///
    pub fn recalculate(&self) -> (Value, BindingDependencies) {
        // Perform the binding in a context to get the value and the dependencies
        let (result, dependencies) = BindingContext::bind(|| (self.calculate_value)());

        // Update the latest value
        let mut latest_value = self.latest_value.borrow_mut();
        *latest_value = Some(result.clone());

        // Pass on the result
        (result, dependencies)
    }
}

///
/// Represents a binding to a value that is computed by a function
///
#[derive(Clone)]
pub struct ComputedBinding<Value: 'static+Clone+PartialEq, TFn>
where TFn: 'static+Fn() -> Value {
    core: Arc<Mutex<RefCell<ComputedBindingCore<Value, TFn>>>>
}

impl<Value: 'static+Clone+PartialEq, TFn> ComputedBinding<Value, TFn>
where TFn: 'static+Fn() -> Value {
    ///
    /// Creates a new computable binding
    ///
    pub fn new(calculate_value: TFn) -> ComputedBinding<Value, TFn> {
        ComputedBinding {
            core: Arc::new(Mutex::new(RefCell::new(ComputedBindingCore::new(calculate_value))))
        }
    }
}

impl<Value: 'static+Clone+PartialEq, TFn> Changeable for ComputedBinding<Value, TFn>
where TFn: 'static+Fn() -> Value {
    fn when_changed(&mut self, what: Arc<Notifiable>) {
        // Lock the core and push this as a thing to perform when this value changes
        let core = self.core.lock().unwrap();
        (*core.borrow_mut()).when_changed.push(ReleasableNotifiable::new(what));
    }
}

impl<Value: 'static+Clone+PartialEq, TFn> Bound<Value> for ComputedBinding<Value, TFn>
where TFn: 'static+Fn() -> Value {
    fn get(&self) -> Value {
        // Borrow the core
        let lock = self.core.lock().unwrap();
        let core = lock.borrow_mut();

        if let Some(value) = core.get() {
            // The value already exists in this item
            value
        } else {
            // TODO: really want to recalculate without locking the core - can do this by moving the function out and doing the recalculation here
            // TODO: locking the core and calling a function can result in deadlocks due to user code structure in particular against other bindings
            // TODO: when we do recalculate without locking, we need to make sure that no extra invalidations arrived between when we started the calculation and when we stored the result
            // TODO: probably fine to return the out of date result rather than the newer one here

            // Need to re-calculate the core
            let (value, _dependencies) = core.recalculate();

            // TODO: need to unhook any previous dependencies and attach to the new set
            // TODO: also need to make sure that any hooks we have are removed if we're only referenced via a hook

            // Return the value
            value
        }
    }
}

///
/// Creates a notifiable reference from a function
///
pub fn notify<TFn>(when_changed: TFn) -> Arc<Notifiable>
where TFn: 'static+Send+FnMut() -> () {
    Arc::new(NotifyFn { when_changed: Mutex::new(RefCell::new(when_changed)) })
}

///
/// Creates a simple bound value with the specified initial value
///
pub fn bind<Value: Clone+PartialEq>(val: Value) -> Binding<Value> {
    Binding::new(val)
}

pub fn computed<Value, TFn>(calculate_value: TFn) -> ComputedBinding<Value, TFn>
where Value: Clone+PartialEq, TFn: 'static+Send+Sync+Fn() -> Value {
    ComputedBinding::new(calculate_value)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_create_binding() {
        let bound = bind(1);
        assert!(bound.get() == 1);
    }

    #[test]
    fn can_update_binding() {
        let mut bound = bind(1);

        bound.set(2);
        assert!(bound.get() == 2);
    }

    #[test]
    fn notified_on_change() {
        let mut bound   = bind(1);
        let changed     = bind(false);

        let mut notify_changed = changed.clone();
        bound.when_changed(notify(move || notify_changed.set(true)));

        assert!(changed.get() == false);
        bound.set(2);
        assert!(changed.get() == true);
    }

    #[test]
    fn not_notified_on_no_change() {
        let mut bound   = bind(1);
        let changed     = bind(false);

        let mut notify_changed = changed.clone();
        bound.when_changed(notify(move || notify_changed.set(true)));

        assert!(changed.get() == false);
        bound.set(1);
        assert!(changed.get() == false);
    }

    #[test]
    fn binding_context_is_notified() {
        let mut bound = bind(1);

        bound.set(2);

        let (value, mut context) = BindingContext::bind(|| bound.get());
        assert!(value == 2);

        let changed = bind(false);
        let mut notify_changed = changed.clone();
        context.when_changed(notify(move || notify_changed.set(true)));

        assert!(changed.get() == false);
        bound.set(3);
        assert!(changed.get() == true);
    }

    #[test]
    fn can_compute_value() {
        let bound           = bind(1);

        let computed_from   = bound.clone();
        let computed        = computed(move || computed_from.get() + 1);

        assert!(computed.get() == 2);
    }

    #[test]
    fn can_recompute_value() {
        let mut bound       = bind(1);

        let computed_from   = bound.clone();
        let computed        = computed(move || computed_from.get() + 1);

        assert!(computed.get() == 2);

        bound.set(2);
        assert!(computed.get() == 3);
    }

    #[test]
    fn computed_notifies_of_changes() {
        let mut bound       = bind(1);

        let computed_from   = bound.clone();
        let mut computed    = computed(move || computed_from.get() + 1);

        let changed = bind(false);
        let mut notify_changed = changed.clone();
        computed.when_changed(notify(move || notify_changed.set(true)));

        assert!(computed.get() == 2);
        assert!(changed.get() == false);

        bound.set(2);
        assert!(changed.get() == true);
    }
}
