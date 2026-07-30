// static

use std::{
    cell::{Ref, RefCell},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
    sync::atomic::{AtomicU64, Ordering::SeqCst},
    vec,
};
use wasm_bindgen::{
    JsCast, JsError, JsValue, prelude::{Closure, wasm_bindgen},
};
use web_sys::{Comment, Element, Node, Text};

use crate::generated::RootFrag;

mod generated;

pub static DIRTY_FLAGS: AtomicU64 = AtomicU64::new(0);

pub struct MutateTracker<T> {
    value: T,
    id: u32,
}

impl<T> MutateTracker<T> {
    pub fn new(value: T, id: u32) -> Self {
        Self { value, id }
    }
}

impl<T> Deref for MutateTracker<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for MutateTracker<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        web_sys::console::log_1(&format!("DerefMut called for id {}", self.id).into());
        DIRTY_FLAGS.fetch_or(1 << self.id, std::sync::atomic::Ordering::SeqCst);
        &mut self.value
    }
}

fn try_upgrade<S>(weak: &Weak<RefCell<S>>) -> Result<Rc<RefCell<S>>, JsValue> {
    weak.upgrade()
        .ok_or_else(|| JsValue::from_str("Failed to upgrade Weak reference"))
}

// TODO: router stuff
/*struct Router {
    children: Box<dyn RouterSnippetTrait>,
}

trait RouterSnippetTrait {
    type State;
    type Props; // ex. (n, word)
    type PropClosures; // (|| -> n, || -> word)
    type PropMasks: Copy; // (mask, mask)
}*/

/// Each route maps a path pattern to a component constructor
/*pub struct Route {
    pattern: &'static str,
    // Params extracted from URL (e.g., /users/:id)
    param_names: &'static [&'static str],
}*/

/// Simple pattern matching: "/users/:id" matches "/users/42"
pub fn match_pattern(
    pattern: &str,
    path: &str,
    param_names: &[&str],
) -> Option<Vec<(String, String)>> {
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let path_parts: Vec<&str> = path.split('/').collect();

    if pattern_parts.len() != path_parts.len() {
        return None;
    }

    let mut params = Vec::new();
    let mut param_idx = 0;

    for (p, actual) in pattern_parts.iter().zip(path_parts.iter()) {
        if p.starts_with(':') {
            params.push((param_names[param_idx].to_string(), actual.to_string()));
            param_idx += 1;
        } else if p != actual {
            return None;
        }
    }

    Some(params)
}

trait ComponentState {
    type Props; // ex. (n, word), should be owned

    fn new(props: Self::Props) -> Rc<RefCell<Self>>;
    fn init(&mut self);
    fn update_derived(&mut self);
}

trait ComponentStateExt: ComponentState {
    fn startup(props: Self::Props) -> Rc<RefCell<Self>>;
}

impl<T: ComponentState> ComponentStateExt for T {
    fn startup(props: Self::Props) -> Rc<RefCell<Self>> {
        let state = Self::new(props);
        {
            let mut state_borrowed = state.borrow_mut();
            state_borrowed.init();
            state_borrowed.update_derived();
        }
        state
    }
}

struct Component<T: GenericFragment> {
    contents: T,
    state: Rc<RefCell<T::State>>,
    parent: Option<Element>, // This is needed for updates, but is only set during mount, so it's an Option
}

impl<'a, T: GenericFragment<Scope = ()>> Component<T>
where
    T::State: ComponentState,
{
    fn new(state: &Rc<RefCell<T::State>>, current_path: &Vec<u32>) -> Result<Self, JsValue> {
        web_sys::console::log_1(&"Initializing Page component".into());

        let contents = T::new(state, &(), current_path)?;
        let new_page = Self {
            contents,
            state: state.clone(),
            parent: None,
        };

        Ok(new_page)
    }

    fn mount(&mut self, parent: &'a Element, add_method: &dyn AddMethod) -> Result<(), JsValue> {
        self.parent = Some(parent.clone());
        self.contents.mount(parent, add_method)
    }

    /// Process an event and return patches to apply to the DOM
    ///
    /// The target_path describes where the event took place.
    /// This function will check if the current component is the target,
    /// and if so run the corresponding user code for that event.
    /// Otherwise, it will propagate the event to the correct child component to process.
    ///
    /// If handled by a child, the function checks for changes in bindable props and updates
    /// the state of the current component accordingly, and marking them to be excluded from
    /// propagation back down to the child to avoid feedback loops.
    ///
    /// Finally, it runs the apply function, to derived, generate patches,
    /// and propagate any changes to children as needed.
    fn proc(&mut self, e: web_sys::Event, target_path: Vec<u32>) -> Result<(), JsValue> {
        web_sys::console::log_1(
            &format!(
                "Processing event: {}, target path: {:?}",
                e.type_(),
                target_path
            )
            .into(),
        );
        // Event handling
        self.contents.proc(&self.state, e, target_path)?;

        self.apply()?;

        Ok(())
    }

    /// Apply changes to the DOM based on the current state and dirty flags
    fn apply(&mut self) -> Result<(), JsValue> {
        // update derived
        self.state.borrow_mut().update_derived();

        // generate patches based on dirty flags
        let flag_snapshot = DIRTY_FLAGS.load(SeqCst);

        self.contents.update(
            self.parent.as_ref().unwrap(),
            &self.state.borrow(),
            flag_snapshot,
        )?;

        // Restore snapshot
        DIRTY_FLAGS.store(flag_snapshot, SeqCst);

        Ok(())
    }

    fn unmount(&mut self) {
        self.contents.unmount();
        self.parent = None;
    }
}

trait GenericFragment {
    type State;

    /// Scope is a nested tuple of items which are only accessible within
    /// the current branch of the fragment tree (moving down the tree adds
    /// more layers to the tuple, without the outmost layer being exclusive
    /// to the fragment branch).
    ///
    /// Scope items are always wrapped in `Rc<T>`s or `DynamicArg<T>`s, and
    /// are created in two ways: via #each blocks, #snippet elements, and
    /// #snippet scopes.
    ///
    /// #each blocks create scope items when new items are added. These items
    /// never change: if the #each block's iterator changes, old items are
    /// unmounted (and their scope items destroyed) and new items are added
    /// (adding new scope items for that item's children).
    ///
    /// #snippet elements create scope items whenever a changes is made to its
    /// arguments. These elements _can_ change, so they are wrapped in
    /// DynamicArg structs which track when they have been changed.
    ///
    /// #snippet scopes create factories on `new`, and while those factories
    /// change internally and may need to trigger updates of the underlying
    /// elements, the actual value of the factory does not change, so it is
    /// wrapped in an Rc<T>.
    type Scope: Clone;

    /// On new, the owning Rc of State & Scope are passed down the call stack so
    /// snippet factories can gain copies of State and dynamic elements can gain
    /// copies of scope.
    fn new(
        state: &Rc<RefCell<Self::State>>,
        scope: &Self::Scope,
        current_path: &Vec<u32>,
    ) -> Result<Self, JsValue>
    where
        Self: Sized;
    fn mount(&mut self, parent: &Element, add_method: &dyn AddMethod) -> Result<(), JsValue>;

    /// Here, only a reference to State is provided, as no additional cloning of State is
    /// necessary.
    fn proc(
        &mut self,
        state: &Rc<RefCell<Self::State>>,
        e: web_sys::Event,
        target_path: Vec<u32>,
    ) -> Result<(), JsValue>;
    fn update(&mut self, parent: &Element, state: &Self::State, flags: u64) -> Result<(), JsValue>;
    fn unmount(&self);
}

struct DynamicText {
    text: Text, // this is a cloneable reference to a DOM element
    thunk: Rc<dyn Fn() -> String>,
    gate_closure: Rc<RefCell<dyn FnMut(u64) -> bool>>,
}

impl DynamicText {
    fn new(
        thunk: impl Fn() -> String + 'static,
        gate_closure: impl FnMut(u64) -> bool + 'static,
    ) -> Self {
        let text_str = thunk();

        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window");

        let text = document.create_text_node(&text_str);

        DynamicText {
            text,
            thunk: Rc::new(thunk),
            gate_closure: Rc::new(RefCell::new(gate_closure)),
        }
    }

    fn mount(&mut self, add_method: &dyn AddMethod) -> Result<(), JsValue> {
        add_method(&self.text)
    }

    fn update(&mut self, flags: u64) -> Result<(), JsValue> {
        if (self.gate_closure.borrow_mut())(flags) {
            self.text.set_text_content(Some(&(self.thunk)()));
        }

        Ok(())
    }

    fn unmount(&self) {
        self.text.remove();
    }
}

#[derive(Clone)]
struct DynamicArg<T> {
    thunk: Rc<dyn Fn() -> T>,
    gate_closure: Rc<RefCell<dyn FnMut(u64) -> bool>>,
    pub version: Rc<RefCell<u64>>,
    value: Rc<RefCell<T>>,
}

impl<T> DynamicArg<T> {
    fn new(
        callback: impl Fn() -> T + 'static,
        gate_closure: impl FnMut(u64) -> bool + 'static,
    ) -> Self {
        let value = Rc::new(RefCell::new(callback()));
        DynamicArg {
            thunk: Rc::new(callback),
            gate_closure: Rc::new(RefCell::new(gate_closure)),
            version: Rc::new(RefCell::new(0)),
            value,
        }
    }

    fn update(&mut self, flags: u64) {
        if (self.gate_closure.borrow_mut())(flags) {
            *self.version.borrow_mut() += 1;
            *self.value.borrow_mut() = (self.thunk)();
        }
    }

    fn get(&self) -> Ref<'_, T> {
        self.value.borrow()
    }

    fn version(&self) -> u64 {
        *self.version.borrow()
    }
}

/// A snippet scope is a branch in the fragment tree where snippets can
/// be accessed. This struct is responsible for creating the snippet
/// factory (which captures the state and scope), and building that
/// factory into the scope so it can be used into the branch's children.
///
/// When update is called on the scope (meaning state or scope may have
/// changed), the update flags are stored in the factory so references to
/// the factory in other components can use them to evaluate changes within
/// the snippets.
struct SnippetScope<
    FC: SnippetContentTrait,
    T: GenericFragment<Scope = (FC::Scope, Rc<SnippetFactory<FC>>), State = FC::State>,
> {
    pub content: T,
    pub factory: Rc<SnippetFactory<FC>>,
}

impl<
        FC: SnippetContentTrait,
        T: GenericFragment<Scope = (FC::Scope, Rc<SnippetFactory<FC>>), State = FC::State>,
    > GenericFragment for SnippetScope<FC, T>
{
    type Scope = FC::Scope;
    type State = FC::State;

    fn new(
        state_rc: &Rc<RefCell<Self::State>>,
        scope: &Self::Scope,
        current_path: &Vec<u32>,
    ) -> Result<Self, JsValue>
    where
        Self: Sized,
    {
        let factory = Rc::new(SnippetFactory::<FC>::new(
            Rc::downgrade(&state_rc),
            scope.clone(),
            current_path,
        ));
        let content = T::new(state_rc, &(scope.clone(), factory.clone()), current_path)?;

        Ok(SnippetScope { content, factory })
    }

    fn mount(&mut self, parent: &Element, add_method: &dyn AddMethod) -> Result<(), JsValue> {
        self.content.mount(parent, add_method)?;

        Ok(())
    }

    fn proc(
        &mut self,
        state_rc: &Rc<RefCell<Self::State>>,
        e: web_sys::Event,
        target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        self.content.proc(state_rc, e, target_path)?;

        Ok(())
    }

    fn update(
        &mut self,
        parent: &Element,
        state_rc: &Self::State,
        flags: u64,
    ) -> Result<(), JsValue> {
        self.content.update(parent, state_rc, flags)?;

        Ok(())
    }

    fn unmount(&self) {
        self.content.unmount();
    }
}

struct SnippetElement<T: SnippetContentTrait> {
    pub content: T,
    factory: SnippetFactory<T>,
    pub comment: web_sys::Comment,
    args: T::Args, // These are all dynamic args
    target_path: Vec<u32>,
}

trait SnippetElementTrait<Args> {
    fn mount(&mut self, parent: &Element, add_method: &dyn AddMethod) -> Result<(), JsValue>;
    fn proc(&mut self, e: web_sys::Event, target_path: Vec<u32>) -> Result<(), JsValue>;
    fn update(&mut self, parent: &Element, flags: u64) -> Result<(), JsValue>;
    fn unmount(&self);

    fn extract_for_swap(&self) -> (Args, Vec<u32>, web_sys::Comment);
}

impl<T: SnippetContentTrait> SnippetElementTrait<T::Args> for SnippetElement<T> {
    fn mount(&mut self, parent: &Element, add_method: &dyn AddMethod) -> Result<(), JsValue> {
        add_method(&self.comment)?;
        self.content
            .mount(parent, &comment_insert_closure(&self.comment, parent))?;

        Ok(())
    }

    fn proc(&mut self, e: web_sys::Event, target_path: Vec<u32>) -> Result<(), JsValue> {
        self.content
            .proc(&try_upgrade(&self.factory.state)?, e, target_path)
    }

    fn update(&mut self, parent: &Element, flags: u64) -> Result<(), JsValue> {
        self.content.update_args(&mut self.args, flags);
        self.content
            .update(parent, &try_upgrade(&self.factory.state)?.borrow(), flags);

        Ok(())
    }

    fn unmount(&self) {
        self.content.unmount();
    }

    fn extract_for_swap(&self) -> (T::Args, Vec<u32>, web_sys::Comment) {
        (
            self.args.clone(),
            self.target_path.clone(),
            self.comment.clone(),
        )
    }
}

trait SnippetContentTrait {
    type State;
    type Scope: Clone;
    type Args: Clone;

    fn update_args(&mut self, args: &mut Self::Args, flags: u64);

    fn new(
        state_rc: &Rc<RefCell<Self::State>>,
        scope: &(Self::Scope, Self::Args),
        current_path: &Vec<u32>,
    ) -> Result<Self, JsValue>
    where
        Self: Sized;

    fn mount(&mut self, parent: &Element, add_method: &dyn AddMethod) -> Result<(), JsValue>;
    fn proc(
        &mut self,
        state_rc: &Rc<RefCell<Self::State>>,
        e: web_sys::Event,
        target_path: Vec<u32>,
    ) -> Result<(), JsValue>;
    fn update(&mut self, parent: &Element, state: &Self::State, flags: u64) -> Result<(), JsValue>;
    fn unmount(&self);
}

/// A snippet factory stores a weak reference to the state and scope
/// a snippet was defined in, and contains in its type the struct which
/// defines the content and behavior of the snippet. The references to
/// State is weak so that factories themselves can be stored in state
/// without causing memory leaks due to ownership cycles.
///
/// Snippet factories can be cloned and passed around to different
/// components and used to create snippets. Snippets created from a
/// factory store a clone of that factory inside them, and when the
/// state or scope of the stored factory changes (as indicated by
/// update_flags), the snippet will update itself.
struct SnippetFactory<T: SnippetContentTrait> {
    state: Weak<RefCell<T::State>>,
    scope: T::Scope, // nested Rcs/DynamicArgs
    update_flags: Rc<RefCell<u64>>,
}

impl<T: SnippetContentTrait> Clone for SnippetFactory<T>
where
    T::Scope: Clone,
{
    fn clone(&self) -> Self {
        SnippetFactory {
            state: self.state.clone(), // Weak can always be cloned
            scope: self.scope.clone(), // T::Scope implements Clone
            update_flags: self.update_flags.clone(),
        }
    }
}

trait SnippetFactoryTrait<Args> {
    fn init(
        &self,
        args: Args,
        current_path: &Vec<u32>,
    ) -> Result<Box<dyn SnippetElementTrait<Args>>, JsValue>;

    fn init_swap(
        &self,
        parent: &Element,
        old_element: &Box<dyn SnippetElementTrait<Args>>,
    ) -> Result<Box<dyn SnippetElementTrait<Args>>, JsValue>;

    fn update(&mut self, flags: u64);
}

impl<T: SnippetContentTrait> SnippetFactory<T> {
    fn new(state: Weak<RefCell<T::State>>, scope: T::Scope, current_path: &Vec<u32>) -> Self {
        SnippetFactory {
            state: state.clone(),
            scope: scope.clone(),
            update_flags: Rc::new(RefCell::new(0)),
        }
    }
}

impl<T: SnippetContentTrait + 'static> SnippetFactoryTrait<T::Args> for SnippetFactory<T> {
    fn init(
        &self,
        args: T::Args,
        current_path: &Vec<u32>,
    ) -> Result<Box<dyn SnippetElementTrait<T::Args>>, JsValue> {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window exists");

        let factory: SnippetFactory<T> = self.clone();
        Ok(Box::new(SnippetElement {
            content: T::new(
                &try_upgrade(&self.state)?,
                &(self.scope.clone(), args.clone()),
                current_path,
            )?,
            comment: document.create_comment(""),
            factory,
            args,
            target_path: current_path.clone(),
        }))
    }

    fn init_swap(
        &self,
        parent: &Element,
        old_element: &Box<dyn SnippetElementTrait<T::Args>>,
    ) -> Result<Box<dyn SnippetElementTrait<T::Args>>, JsValue> {
        let factory: SnippetFactory<T> = self.clone();
        old_element.unmount();
        let (args, current_path, comment) = old_element.extract_for_swap();
        let mut new_content = T::new(
            &try_upgrade(&self.state)?,
            &(self.scope.clone(), args.clone()),
            &current_path,
        )?;
        new_content.mount(parent, &comment_insert_closure(&comment, parent))?;
        Ok(Box::new(SnippetElement {
            content: new_content,
            comment,
            factory,
            args,
            target_path: current_path.clone(),
        }))
    }

    /// Store changes to state so snippets using this factory know what to update
    fn update(&mut self, flags: u64) {
        self.update_flags = Rc::new(RefCell::new(flags));
    }
}

struct DummyFactory<T> {
    _phantom: PhantomData<T>,
}

impl<T> DummyFactory<T> {
    fn new() -> Self {
        DummyFactory { _phantom: PhantomData }
    }
}

impl<T> SnippetFactoryTrait<T> for DummyFactory<T> {
    fn init(
        &self,
        args: T,
        current_path: &Vec<u32>,
    ) -> Result<Box<dyn SnippetElementTrait<T>>, JsValue> {
        Err(JsError::new("Tried to get snippet from dummy factory").into())
    }

    fn init_swap(
        &self,
        parent: &Element,
        old_element: &Box<dyn SnippetElementTrait<T>>,
    ) -> Result<Box<dyn SnippetElementTrait<T>>, JsValue>
    {
        Err(JsError::new("Tried to get snippet from dummy factory").into())
    }

    fn update(&mut self, flags: u64) {}
}

struct IfElement<T: IfContentTrait> {
    pub comment: Comment,
    pub content_enum: T,
    current_path: Vec<u32>,
    state_rc: Rc<RefCell<T::State>>,
    scope: T::Scope,
}

trait IfContentTrait {
    type State;
    type Scope: Clone; // this can implement Copy 'cause it's all references

    // State, Scope (internal references in nested tuples)
    fn branch_changed(&self, state: &Self::State, scope: &Self::Scope, flags: u64) -> bool;
    fn new(
        state: &Rc<RefCell<Self::State>>,
        scope: &Self::Scope,
        current_path: &Vec<u32>,
    ) -> Result<Self, JsValue>
    where
        Self: Sized;
    fn mount(&mut self, parent: &Element, add_method: &dyn AddMethod) -> Result<(), JsValue>;
    fn proc(
        &mut self,
        state: &Rc<RefCell<Self::State>>,
        e: web_sys::Event,
        target_path: Vec<u32>,
    ) -> Result<(), JsValue>;
    fn update(&mut self, parent: &Element, state: &Self::State, flags: u64) -> Result<(), JsValue>;
    fn unmount(&self);
}

impl<T: IfContentTrait> GenericFragment for IfElement<T> {
    type State = T::State;
    type Scope = T::Scope;

    fn new(
        state: &Rc<RefCell<Self::State>>,
        scope: &Self::Scope,
        current_path: &Vec<u32>,
    ) -> Result<Self, JsValue> {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window exists");

        Ok(Self {
            comment: document.create_comment(""),
            content_enum: T::new(state, scope, current_path)?,
            current_path: current_path.clone(),
            state_rc: state.clone(),
            scope: scope.clone(),
        })
    }

    fn mount(&mut self, parent: &Element, add_method: &dyn AddMethod) -> Result<(), JsValue> {
        add_method(&self.comment)?;
        self.content_enum
            .mount(parent, &comment_insert_closure(&self.comment, parent))
    }

    fn proc(
        &mut self,
        state: &Rc<RefCell<Self::State>>,
        e: web_sys::Event,
        target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        self.content_enum.proc(state, e, target_path)
    }

    fn update(&mut self, parent: &Element, state: &Self::State, flags: u64) -> Result<(), JsValue> {
        let changed = { self.content_enum.branch_changed(&state, &self.scope, flags) };

        if changed {
            // Unmount old content
            self.content_enum.unmount();

            // Mount new content
            self.content_enum = T::new(&self.state_rc, &self.scope, &self.current_path)?;
            self.content_enum
                .mount(parent, &comment_insert_closure(&self.comment, parent))?;
        }
        self.content_enum.update(parent, state, flags)
    }

    fn unmount(&self) {
        self.content_enum.unmount();
        self.comment.remove();
    }
}

/// Represents the content of an each block, which may have multiple instances
/// Each instance is identified by a unique key produced by a hash function
struct EachElement<T: EachContentTrait> {
    pub comment: web_sys::Comment,
    pub content: Vec<(u64, T, Rc<T::Item>)>, // (hash, DOM ref, item)
    current_path: Vec<u32>,
    state_rc: Rc<RefCell<T::State>>,
    scope: T::Scope,
}

/// Functions which the fragment inside an each block must implement to be used as content for an EachElement
trait EachContentTrait {
    type State;
    type Scope: Clone; // this can implement Clone 'cause it's all references

    // State, Scope (internal references in nested tuples)
    type Item: std::hash::Hash;

    fn generate(state: &Self::State, scope: &Self::Scope, flags: u64) -> Option<Vec<Self::Item>>;
    fn new(
        state: &Rc<RefCell<Self::State>>,
        scope: &(Self::Scope, Rc<Self::Item>),
        current_path: &Vec<u32>,
    ) -> Result<Self, JsValue>
    where
        Self: Sized;
    fn mount(&mut self, parent: &Element, add_method: &dyn AddMethod) -> Result<(), JsValue>;
    fn proc(
        &mut self,
        state: &Rc<RefCell<Self::State>>,
        e: web_sys::Event,
        target_path: Vec<u32>,
    ) -> Result<(), JsValue>;
    fn update(&mut self, parent: &Element, state: &Self::State, flags: u64) -> Result<(), JsValue>;
    fn unmount(&self);
}

impl<T: EachContentTrait> GenericFragment for EachElement<T> {
    type State = T::State;
    type Scope = T::Scope;

    fn new(
        state_rc: &Rc<RefCell<Self::State>>,
        scope: &Self::Scope,
        current_path: &Vec<u32>,
    ) -> Result<Self, JsValue> {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window exists");

        Ok(Self {
            comment: document.create_comment(""),
            content: T::generate(&state_rc.borrow(), scope, u64::MAX)
                .unwrap()
                .into_iter()
                .map(|item| {
                    let hash = hash_item(&item);
                    let item_rc = Rc::new(item);
                    let content = T::new(state_rc, &(scope.clone(), item_rc.clone()), current_path)
                        .expect("Failed to create content");
                    (hash, content, item_rc)
                })
                .collect(),
            current_path: current_path.clone(),
            state_rc: state_rc.clone(),
            scope: scope.clone(),
        })
    }

    fn mount(&mut self, parent: &Element, add_method: &dyn AddMethod) -> Result<(), JsValue> {
        add_method(&self.comment)?;
        for (_, content, _) in &mut self.content {
            content.mount(parent, &comment_insert_closure(&self.comment, parent))?;
        }
        Ok(())
    }

    fn proc(
        &mut self,
        state_rc: &Rc<RefCell<Self::State>>,
        e: web_sys::Event,
        target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        for (_, content, _item) in &mut self.content {
            content.proc(state_rc, e.clone(), target_path.clone())?;
        }
        Ok(())
    }

    fn update(&mut self, parent: &Element, state: &Self::State, flags: u64) -> Result<(), JsValue> {
        // Diff & update each list if necessary
        if let Some(new_items) = T::generate(state, &self.scope, flags) {
            let new_hashes: Vec<u64> = new_items.iter().map(|item| hash_item(item)).collect();
            let mut takeable_new_items: Vec<Option<T::Item>> =
                new_items.into_iter().map(Some).collect();

            // Find head and tail of middle batch
            let mut head = 0;
            while head < self.content.len()
                && head < new_hashes.len()
                && self.content[head].0 == new_hashes[head]
            {
                head += 1;
            }
            let mut tail = 0;
            while tail < self.content.len() - head
                && tail < new_hashes.len() - head
                && self.content[self.content.len() - 1 - tail].0
                    == new_hashes[new_hashes.len() - 1 - tail]
            {
                tail += 1;
            }

            // Build a map of new hashes to their indices for quick lookup
            let mut new_item_source_map = std::collections::HashMap::new();
            for i in head..(new_hashes.len() - tail) {
                new_item_source_map.insert(new_hashes[i], i);
            }

            // Create a new array to hold the source indices for the new items
            // TODO: items before and after middle batch should be handled separately to avoid unnecessary moves
            let mut new_item_source_array = vec![None; new_hashes.len()];
            for i in 0..head {
                new_item_source_array[i] = Some(i);
            }
            for i in head..(self.content.len() - tail) {
                if let Some(&new_index) = new_item_source_map.get(&self.content[i].0) {
                    new_item_source_array[new_index] = Some(i);
                } else {
                    // Unmount code for removed item
                    self.content[i].1.unmount();
                }
            }
            for i in (new_hashes.len() - tail)..new_hashes.len() {
                new_item_source_array[i] = Some(self.content.len() - (new_hashes.len() - i));
            }

            // Find longest increasing subsequence of source indices in new_item_source_array
            let mut subs = vec![Vec::new()]; // list of all increasing subsequences found so far
            let mut last_index: i32 = -1; // index of the last item in the longest increasing subsequence
            for (new_index, source_index_opt) in new_item_source_array.iter().enumerate() {
                let current_sub = subs.last_mut().unwrap();
                if let Some(source_index) = source_index_opt {
                    if current_sub.is_empty() || *source_index as i32 > last_index {
                        current_sub.push(new_index);
                    } else {
                        subs.push(vec![new_index]);
                    }
                    last_index = *source_index as i32;
                }
            }
            let longest_sub = subs
                .into_iter()
                .max_by_key(|sub| sub.len())
                .unwrap_or_default();

            let mut takeable_old_items: Vec<Option<(u64, T, Rc<T::Item>)>> =
                self.content.drain(..).map(Some).collect();

            // Move & mount items into new list
            let mut new_list = Vec::new();
            for (new_index, source_index_opt) in new_item_source_array.into_iter().enumerate() {
                if let Some(source_index) = source_index_opt {
                    if !longest_sub.contains(&new_index) {
                        // Move existing item to correct position
                        let mut old_item = takeable_old_items[source_index].take().unwrap();
                        old_item.1.unmount();
                        old_item
                            .1
                            .mount(parent, &comment_insert_closure(&self.comment, parent))?;
                        new_list.push(old_item);
                    } else {
                        // Item is already in correct position, just update anchor for next iteration
                        let old_item = takeable_old_items[source_index].take().unwrap();
                        new_list.push(old_item);
                    }
                } else {
                    // Mount new item
                    let new_item = Rc::new(takeable_new_items[new_index].take().unwrap());
                    let mut new_contents = T::new(
                        &self.state_rc,
                        &(self.scope.clone(), new_item.clone()),
                        &self.current_path,
                    )?;
                    new_contents.mount(parent, &comment_insert_closure(&self.comment, parent))?;
                    new_list.push((new_hashes[new_index], new_contents, new_item));
                }
            }
            self.content = new_list.into_iter().collect();
        }

        // Update all items (including moved ones) with new scope and flags
        for (_, content, _item) in &mut self.content {
            content.update(parent, state, flags)?;
        }

        Ok(())
    }

    fn unmount(&self) {
        for (_, content, _) in &self.content {
            content.unmount();
        }
        self.comment.remove();
    }
}

fn hash_item<T: std::hash::Hash>(item: &T) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;

    let mut hasher = DefaultHasher::new();
    item.hash(&mut hasher);
    hasher.finish()
}

pub fn prepend_path(base: &Vec<u32>, addition: u32) -> Vec<u32> {
    let mut out = vec![addition];
    out.extend_from_slice(base);
    out
}

pub fn add_listener(
    el: &web_sys::Element,
    event: &str,
    target_path: Vec<u32>,
) -> Result<(), JsValue> {
    let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
        handle_event(e, target_path.clone());
    }) as Box<dyn FnMut(_)>);
    el.add_event_listener_with_callback(event, closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

pub trait AddMethod: Fn(&Node) -> Result<(), JsValue> {}
impl<F: Fn(&Node) -> Result<(), JsValue>> AddMethod for F {}

fn child_append_closure(parent: &Element) -> impl AddMethod + '_ {
    let closure = move |el: &Node| {
        parent.append_child(el)?;
        Ok(())
    };
    closure
}

fn comment_insert_closure<'a>(comment: &'a Comment, parent: &'a Element) -> impl AddMethod + 'a {
    let closure = move |el: &Node| {
        parent.insert_before(el, Some(comment))?;
        Ok(())
    };
    closure
}

thread_local! {
    pub static PAGE: RefCell<Option<Component<RootFrag>>> = RefCell::new(None);
}

#[wasm_bindgen]
pub fn mount() -> Result<(), JsValue> {
    web_sys::console::log_1(&"Mounting application".into());
    PAGE.with(|page| {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window");
        let body = document.body().expect("document should have a body");

        let state = <RootFrag as GenericFragment>::State::startup(());
        let mut new_page = Component::<RootFrag>::new(&state, &vec![])?;
        new_page.mount(&body, &child_append_closure(&body))?;
        *page.borrow_mut() = Some(new_page);
        web_sys::console::log_1(&"Page component mounted".into());
        Ok(())
    })
}

pub fn handle_event(e: web_sys::Event, target: Vec<u32>) {
    DIRTY_FLAGS.store(0, SeqCst);
    let _ = PAGE.with(|page| {
        let page = &mut *page.borrow_mut();
        let page = page.as_mut().expect("Page component should be initialized");
        page.proc(e, target)
            .or_else(|e| {
                web_sys::console::error_1(&format!("Error processing event: {:?}", e).into());
                Err(e)
            })
            .ok();
    });
}

fn scope_example(scope: (((), Rc<RefCell<u32>>), Rc<RefCell<String>>)) -> () {
    let (((), _counter), _text) = scope;
    let _counter_ref = _counter.borrow();
    let _text_ref = _text.borrow();
}
