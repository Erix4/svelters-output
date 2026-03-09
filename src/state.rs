use wasm_bindgen::JsValue;
use web_sys::{Element, Text};

use crate::{
    add_listener, prepend_path, AddMethod, EachContentTrait, EachElement, IfContentTrait,
    IfElement, MutateTracker, DIRTY_FLAGS,
};
use std::{sync::atomic::Ordering::SeqCst, vec};

// User generated agnostic code
struct MyStruct {
    a: i32,
    b: String,
}

fn add<T: std::ops::Add<Output = T>>(a: T, b: T) -> T {
    a + b
}

struct PageState {
    // Prop state: none

    // Reactive state:
    counter: MutateTracker<i32>,        // id: 0
    my_struct: MutateTracker<MyStruct>, // id: 1

    // Derived state:
    counter_plus_one: i32,
}

impl PageState {
    fn init(&mut self) {
        //let mut other_var = 42;
        //other_var += 1;

        *self.counter += 1;
    }

    // User defined functions for Page
    fn increment(&mut self) {
        *self.counter += 1;
        add(*self.counter, 5);
    }

    fn update_struct(&mut self, inc: bool) {
        if inc {
            self.increment();
        }
        self.my_struct.a += *self.counter; // NOTE: no need to deref when a field in a state variable is accessed (it already does it implicitly)
        self.my_struct.b = format!("Count is {}", *self.counter);
    }

    pub fn new() -> Self {
        let mut state = PageState {
            counter: MutateTracker::new(0, 0),
            my_struct: MutateTracker::new(
                MyStruct {
                    a: 10,
                    b: "hello".to_string(),
                },
                1,
            ),
            counter_plus_one: 0,
        };
        state.init();

        state
    }
}

/// Global state for the page component
///
/// Each component has the following sections recursively:
/// - Content array: array of all elements in the component, used for patching
/// - Prop state: state variables passed from parent component
/// - Reactive state: state variables that trigger re-renders when changed
/// - Derived state: state variables that are computed from reactive state
/// - Child component local state: recurse
/// - Fragment state: state variables for managing fragments (e.g., #if, #each)
///
/// All values except derived are user defined. Derived state is auto-generated.
pub struct Page {
    contents: PageRootFrag,
    state: PageState,
}

impl Page {
    pub fn new(parent: &Element) -> Result<Self, JsValue> {
        web_sys::console::log_1(&"Initializing Page component".into());

        let state = PageState::new();
        let contents = PageRootFrag::new(&state)?;
        let mut new_page = Self { contents, state };
        new_page.mount(parent)?;

        DIRTY_FLAGS.store(u64::MAX, SeqCst); // mark all as dirty for initial render
        new_page.apply()?;

        Ok(new_page)
    }

    fn mount(&mut self, parent: &Element) -> Result<(), JsValue> {
        self.contents.mount(parent)
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
    pub fn proc(&mut self, e: web_sys::Event, target_path: Vec<u32>, _: ()) -> Result<(), JsValue> {
        web_sys::console::log_1(
            &format!(
                "Processing event: {}, target path: {:?}",
                e.type_(),
                target_path
            )
            .into(),
        );
        // Event handling
        self.contents.proc(&mut self.state, e, target_path)?;

        self.apply()?;

        Ok(())
    }

    /// Generate patches based on changes to reactive state and derived state
    ///
    /// The flag_exclude parameter is a bitmask of any bindable props that should
    /// not be included in the propagation to children to avoid feedback loops.
    pub fn apply(&mut self) -> Result<(), JsValue> {
        let state = &mut self.state;

        // update derived
        if DIRTY_FLAGS.load(SeqCst) & 1 << 0 != 0 {
            state.counter_plus_one = *state.counter + 1;
            DIRTY_FLAGS.fetch_or(1 << 3, SeqCst);
        }

        // generate patches based on dirty flags
        let flag_snapshot = DIRTY_FLAGS.load(SeqCst);

        self.contents.update(state, flag_snapshot)?;

        // Restore snapshot
        DIRTY_FLAGS.store(flag_snapshot, SeqCst);

        Ok(())
    }
}

struct PageRootFrag {
    a: Element,
    b: Element,
    c: Element,
    d: Element,
    e: Element,                           // 4
    f: IfElement<PageState, If1Content>,  // 5 - #if block
    g: EachElement<PageState, EachFrag1>, // 6 - #each
    h: Element,
    i: Element,
    j: Button,
}

impl PageRootFrag {
    fn new(state: &PageState) -> Result<Self, JsValue> {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window");

        let el0 = document.create_element("div")?;
        let el1 = document.create_element("button")?;
        let el2 = document.create_element("button")?;
        let el3 = document.create_element("p")?;
        let el4 = document.create_element("p")?;
        let el5 = IfElement::new(state, ())?;
        let el6 = EachElement::new(state, ())?;
        let el7 = document.create_element("div")?;
        let el8 = document.create_element("p")?;
        let el9 = Button::new()?;

        // target paths are static and unique to each fragment
        // listeners are in new() to preserve them if moved (unmounted & remounted)
        add_listener(&el1, "click", vec![1])?;
        add_listener(&el2, "click", vec![2])?;

        Ok(Self {
            a: el0,
            b: el1,
            c: el2,
            d: el3,
            e: el4,
            f: el5,
            g: el6,
            h: el7,
            i: el8,
            j: el9,
        })
    }

    fn mount(&self, parent: &Element) -> Result<(), JsValue> {
        parent.append_child(&self.a)?;
        self.a.append_child(&self.b)?;
        self.a.append_child(&self.c)?;
        self.j.mount(vec![3], |el| {
            self.a.append_child(el)?;
            Ok(())
        })?; // TODO: remove parent path & update button
        self.a.append_child(&self.d)?;
        self.a.append_child(&self.e)?;
        self.f.mount(&self.a)?;
        self.g.mount(&self.a)?;
        self.a.append_child(&self.h)?;
        self.h.append_child(&self.i)?;
        Ok(())
    }

    fn proc(
        &mut self,
        state: &mut PageState,
        e: web_sys::Event,
        mut target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        let target = target_path.pop().unwrap();
        match e.type_().as_str() {
            "click" if target == 1 => {
                state.increment();
            }
            "click" if target == 2 => {
                (|state: &mut PageState| state.update_struct(true))(state);
            }
            // target is in button component
            _ if target == 3 => {
                self.j.proc(e, target_path)?;
                let child_bindable_flags = DIRTY_FLAGS.load(SeqCst);
                DIRTY_FLAGS.store(0, SeqCst); // reset for parent processing

                // Update bindable props based on child changes
                if child_bindable_flags & 1 << 1 != 0 {
                    // Ex.
                    // state.bindable_prop = self.button_1.some_prop;
                    // DIRTY_FLAGS.fetch_or(1 << 0, SeqCst); // manually mark parent prop as dirty if it was changed by child

                    // Run user defined closure with bound function call
                    (|state: &mut PageState, new_val| {
                        *state.counter = new_val;
                    })(state, *self.j.func_call);
                }
            }
            _ if target == 6 => {
                // Example of an #each block nested handler

                // Get item specific scoped variable
                let target = target_path.pop().unwrap();
                let item = self.g.content[target as usize].2; // get the item from the #each content array based on index

                // Find target element in #each content
                let target = target_path.pop().unwrap();
                match e.type_().as_str() {
                    "click" if target == 1 => {
                        // User closure:
                        (|state: &mut PageState| {
                            *state.counter = item;
                        })(state);
                    }
                    // Could also be a component in here
                    _ if target == 3 => {
                        // Propagate downward & handle bindings just as shown above
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn update(&mut self, state: &PageState, flags: u64) -> Result<(), JsValue> {
        web_sys::console::log_1(&format!("Updating PageRootFrag with flags: {:b}", flags).into());
        // counter changed
        if flags & 1 << 0 != 0 {
            self.b
                .set_inner_html(&format!("Counter: {}", *state.counter));

            self.d.set_inner_html(&format!("Count: {}", *state.counter));
        }

        self.f.update(&self.a, state, (), flags)?;
        self.g.update(&self.a, state, (), flags)?;

        // my_struct changed
        if flags & 1 << 1 != 0 {
            self.c.set_inner_html(&format!(
                "Struct A: {} (Click to update)",
                state.my_struct.a
            ));
            self.e
                .set_inner_html(&format!("Struct B: {}", state.my_struct.b));
        }

        // counter_plus_one changed (derived)
        if flags & 1 << 3 != 0 {
            self.i
                .set_inner_html(&format!("Counter plus one: {}", state.counter_plus_one));
        }

        // Propagate to children

        // Button 1
        if flags & 1 << 1 != 0 {
            DIRTY_FLAGS.store(0, SeqCst); // reset before child processing

            // Update props for button
            if flags & 1 << 1 != 0 {
                self.j.text = state.my_struct.b.clone(); // my_struct.b is the given expression
                DIRTY_FLAGS.fetch_or(1 << 0, SeqCst); // manually mark prop as dirty
            }

            self.j.apply()?;
        }

        Ok(())
    }
}

enum If1Content {
    If(IfBranch1),
    Else(IfBranch2),
}

impl IfContentTrait<PageState> for If1Content {
    type Scope<'a> = ();

    fn branch_changed(&self, state: &PageState, _scope: Self::Scope<'_>, flags: u64) -> bool {
        if flags & 1 << 0 != 0 {
            match self {
                If1Content::If(_) if *state.counter > 5 => false,
                If1Content::Else(_) if !(*state.counter > 5) => false,
                _ => true,
            }
        } else {
            false
        }
    }

    fn new(state: &PageState, scope: Self::Scope<'_>) -> Result<Self, JsValue> {
        Ok(if *state.counter > 5 {
            If1Content::If(IfBranch1::new(state)?)
        } else {
            If1Content::Else(IfBranch2::new(state)?)
        })
    }

    fn mount(&self, parent: &Element, comment: &web_sys::Comment) -> Result<(), JsValue> {
        match &self {
            If1Content::If(contents) => contents.mount(parent, comment),
            If1Content::Else(contents) => contents.mount(parent, comment),
        }
    }

    fn proc(
        &self,
        state: &PageState,
        scope: Self::Scope<'_>,
        e: web_sys::Event,
        target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        // this could call into the branches but in this example it wouldn't do anything anyway
        match &self {
            If1Content::If(contents) => Ok(()),
            If1Content::Else(contents) => Ok(()),
        }
    }

    fn update(
        &mut self,
        parent: &Element,
        state: &PageState,
        scope: Self::Scope<'_>,
        flags: u64,
    ) -> Result<(), JsValue> {
        // Check for changes in content of active branch
        match &self {
            If1Content::If(contents) => contents.update(state, scope, flags),
            If1Content::Else(contents) => contents.update(state, scope, flags),
        }
    }

    fn unmount(&self) {
        match self {
            If1Content::If(contents) => contents.unmount(),
            If1Content::Else(contents) => contents.unmount(),
        }
    }
}

struct IfBranch1 {
    a: Element,
    b: Text,
    c: Text,
}

impl IfBranch1 {
    fn new(state: &PageState) -> Result<Self, JsValue> {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window exists");

        let el5_if = document.create_element("p")?;
        let el5_text_1 = document.create_text_node("Counter is greater than 5! ");
        let el5_text_2 = document.create_text_node(&format!("{}", state.my_struct.a));

        Ok(Self {
            a: el5_if,
            b: el5_text_1,
            c: el5_text_2,
        })
    }

    fn mount(&self, parent: &Element, comment: &web_sys::Comment) -> Result<(), JsValue> {
        parent.insert_before(&self.a, Some(&comment))?;
        self.a.append_child(&self.b)?;
        self.a.append_child(&self.c)?;
        Ok(())
    }

    fn update(&self, state: &PageState, scope: (), flags: u64) -> Result<(), JsValue> {
        if flags & 1 << 1 != 0 {
            self.b
                .set_text_content(Some(&format!("{}", state.my_struct.a)));
        }
        Ok(())
    }

    fn unmount(&self) {
        self.a.remove();
    }
}

struct IfBranch2 {
    a: Element,
}

impl IfBranch2 {
    fn new(state: &PageState) -> Result<Self, JsValue> {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window exists");

        let el5_else = document.create_element("p")?;
        el5_else.set_inner_html("Counter is 5 or less.");

        Ok(Self { a: el5_else })
    }

    fn mount(&self, parent: &Element, comment: &web_sys::Comment) -> Result<(), JsValue> {
        parent.insert_before(&self.a, Some(&comment))?;
        Ok(())
    }

    fn update(&self, _state: &PageState, _scope: (), _flags: u64) -> Result<(), JsValue> {
        Ok(())
    }

    fn unmount(&self) {
        self.a.remove();
    }
}

#[derive(Clone)]
struct EachFrag1 {
    // No need to store the comment here since it's stored in the EachElement struct
    a: Element,
    b: IfElement<PageState, If2Content>,
}

impl EachContentTrait<PageState> for EachFrag1 {
    type Item = i32;
    type Scope<'a> = ();

    fn generate(state: &PageState, _scope: Self::Scope<'_>, flags: u64) -> Option<Vec<Self::Item>> {
        if flags & 1 << 0 != 0 {
            Some((0..*state.counter).collect())
        } else {
            None
        }
    }

    fn new(state: &PageState, scope: (Self::Scope<'_>, &Self::Item)) -> Result<Self, JsValue> {
        let (_, item) = scope;

        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window exists");

        let node_1 = document.create_element("p")?;
        node_1.set_inner_html(&format!("Number: {}", item));
        let node_2 = IfElement::new(state, scope)?;
        Ok(Self {
            a: node_1,
            b: node_2,
        })
    }

    fn mount(&self, parent: &Element, anchor: &web_sys::Comment) -> Result<(), JsValue> {
        parent.insert_before(&self.a, Some(anchor))?;
        Ok(())
    }

    fn proc(
        &self,
        state: &PageState,
        scope: (Self::Scope<'_>, &Self::Item),
        e: web_sys::Event,
        target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        // Handle events for content inside #each block if needed
        // TODO: unwrap target & go deeper

        Ok(())
    }

    fn update(
        &mut self,
        parent: &Element,
        _state: &PageState,
        _scope: (Self::Scope<'_>, &Self::Item),
        _flags: u64,
    ) -> Result<(), JsValue> {
        // No reactive stuff inside, otherwise updates would go here
        Ok(())
    }

    fn unmount(&self) {
        self.a.remove();
    }
}

// TODO: this should be an enum
#[derive(Clone)]
struct If2Content {
    a: Element,
}

impl IfContentTrait<PageState> for If2Content {
    type Scope<'a> = ((), &'a i32);

    fn branch_changed(&self, state: &PageState, _scope: Self::Scope<'_>, flags: u64) -> bool {
        false // no dynamic content in this example, so branch never changes after initial render
    }

    fn new(state: &PageState, scope: Self::Scope<'_>) -> Result<Self, JsValue> {
        let (_, item) = scope;
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window exists");

        let el = document.create_element("p")?;
        el.set_inner_html(&format!("Item is {}", item));

        Ok(Self { a: el })
    }

    fn mount(&self, parent: &Element, comment: &web_sys::Comment) -> Result<(), JsValue> {
        parent.insert_before(&self.a, Some(&comment))?;
        Ok(())
    }

    fn proc(
        &self,
        state: &PageState,
        scope: Self::Scope<'_>,
        e: web_sys::Event,
        target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        // Handle events for content inside #each block if needed
        Ok(())
    }

    fn update(
        &mut self,
        parent: &Element,
        state: &PageState,
        scope: Self::Scope<'_>,
        flags: u64,
    ) -> Result<(), JsValue> {
        let (_, item) = scope;

        if flags & 1 << 3 != 0 {
            self.a.set_inner_html(&format!("Item is {}", item));
        }
        Ok(())
    }

    fn unmount(&self) {
        self.a.remove();
    }
}
struct Button {
    // Element array:
    contents: (Element, Element, Element),

    // Props:
    text: String,
    // Bindable props (any func prop affecting state is automatically bindable):
    func_call: MutateTracker<i32>,

    // Reactive state:
    button_counter: MutateTracker<i32>,
}

impl Button {
    fn new() -> Result<Self, JsValue> {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window");

        let el0 = document.create_element("div")?;
        let el1 = document.create_element("button")?;
        let el2 = document.create_element("button")?;
        el2.set_inner_html("Set parent");

        let contents = (el0, el1, el2);

        let mut new_self = Self {
            contents,
            text: "".to_string(),
            func_call: MutateTracker::new(0, 1),
            button_counter: MutateTracker::new(0, 0),
        };

        DIRTY_FLAGS.store(u64::MAX, SeqCst); // mark all as dirty for initial render
        new_self.apply()?;

        Ok(new_self)
    }

    fn mount(&self, parent_path: Vec<u32>, add_method: impl AddMethod) -> Result<(), JsValue> {
        add_method(&self.contents.0)?; // add root element to parent using provided method
        self.contents.0.append_child(&self.contents.1)?;
        self.contents.0.append_child(&self.contents.2)?;

        add_listener(&self.contents.1, "click", prepend_path(&parent_path, 1))?;
        add_listener(&self.contents.2, "click", prepend_path(&parent_path, 2))?;

        Ok(())
    }

    fn unmount(&mut self) {
        // Removing the root of the component will remove all children, so we only need to remove contents.0
        self.contents.0.remove();
    }

    fn proc(&mut self, e: web_sys::Event, mut target_path: Vec<u32>) -> Result<(), JsValue> {
        // Event handling
        let target = target_path.pop().unwrap();
        match e.type_().as_str() {
            "click" if target == 1 => {
                *self.button_counter += 1;
            }
            "click" if target == 2 => {
                *self.func_call = *self.button_counter;
            }
            _ => {}
        }

        self.apply()?;

        Ok(())
    }

    fn apply(&mut self) -> Result<(), JsValue> {
        // Update derived (none in this example)

        // Generate patches for any state changes in this component
        if DIRTY_FLAGS.load(SeqCst) & (1 << 0 | 1 << 1) != 0 {
            self.contents
                .1
                .set_inner_html(&format!("{}: {}", self.text, *self.button_counter));
        }

        let flag_snapshot = DIRTY_FLAGS.load(SeqCst);

        // Propagate to children (none in this example)

        // Restore snapshot
        DIRTY_FLAGS.store(flag_snapshot, SeqCst);

        // Return patches to parent for processing
        Ok(())
    }
}

/*struct NotFoundPage {
    // Element array:
    contents: (Element,),
    // Prop state: none

    // Reactive state: none

    // Derived state: none

    // Child component local state: none

    // Fragment state: none
}

impl NotFoundPage {
    fn new(parent_path: Vec<u32>, body: Element) -> Result<Self, JsValue> {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window");

        let el0 = document.create_element("div")?;
        el0.set_inner_html("404 - Page not found");
        body.append_child(&el0)?;

        Ok(Self { contents: (el0,) })
    }
}

struct UserPage {
    // Element array:
    contents: (Element,),

    // Params:
    user_id: String,
    // Prop state: none

    // Reactive state: none

    // Derived state: none

    // Child component local state: none

    // Fragment state: none
}

impl UserPage {
    fn new(parent_path: Vec<u32>, body: Element, params: String) -> Result<Self, JsValue> {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window");

        // If multiple params, would look like:
        // let (a, b) = params; // destructure params tuple into individual variables
        let user_id = params;

        let el0 = document.create_element("div")?;
        el0.set_inner_html(&format!("User ID: {}", user_id));
        body.append_child(&el0)?;

        Ok(Self {
            contents: (el0,),
            user_id,
        })
    }
}*/
