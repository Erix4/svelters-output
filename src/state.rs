use wasm_bindgen::JsValue;
use web_sys::Element;

use crate::{
    DIRTY_FLAGS, EachElement, IfElement, MutateTracker, add_listener, diff_each_content, prepend_path
};
use std::{
    sync::atomic::Ordering::SeqCst,
    vec,
};

// User generated agnostic code
struct MyStruct {
    a: i32,
    b: String,
}

fn add<T: std::ops::Add<Output = T>>(a: T, b: T) -> T {
    a + b
}

/*const ROUTES: &[Route] = &[
    Route {
        pattern: "/",
        param_names: &[],
    },
    Route {
        pattern: "/users/:id",
        param_names: &["id"],
    },
];

enum ActivePage {
    Page(Page),
    UserPage(UserPage),
    NotFound(NotFoundPage),
}

impl ActivePage {
    fn new(path: &str) -> Result<Self, JsValue> {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window");
        let body = document.body().expect("document should have a body");

        match path {
            "/" => Ok(ActivePage::Page(Page::new(vec![0], body.into())?)),
            _ => Ok(ActivePage::NotFound(NotFoundPage::new(
                vec![0],
                body.into(),
            )?)),
        }
    }
}

struct Router {
    elements: (Element),     // element array for router component
    active_page: ActivePage, // state for router component

    params: Vec<(String, String)>, // extracted route params for active page (name, value)
}

impl Router {
    fn new() -> Result<Self, JsValue> {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window");
        let body = document.body().expect("document should have a body");

        let el0 = document.create_element("p")?;
        el0.set_inner_html("Title");
        body.append_child(&el0)?;

        let path = window.location().pathname().unwrap_or_default();

        Ok(Self {
            elements: (el0),
            active_page: ActivePage::new(&path)?,
            params: vec![],
        })
    }

    fn navigate(&mut self, new_path: &str) -> Result<(), JsValue> {
        let window = web_sys::window().unwrap();
        let history = window.history().unwrap();

        history.push_state_with_url(&JsValue::NULL, "", Some(new_path))?;

        self.current_path = new_path.to_string();

        Ok(())
    }

    fn proc(
        &mut self,
        e: web_sys::Event,
        mut target_path: Vec<u32>,
        _: (),
    ) -> Result<(), JsValue> {
        let target = target_path.pop().unwrap();

        match target {
            1 => {
                // Delegate to active page
                match &mut self.active_page {
                    ActivePage::Home(page) => page.proc(e, target_path, ())?,
                    ActivePage::About(page) => page.proc(e, target_path, ())?,
                    ActivePage::User(page) => page.proc(e, target_path, ())?,
                    ActivePage::NotFound(_) => {}
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn apply(&mut self) -> Result<(), JsValue> {
        // Route changed - swap the page
        if DIRTY_FLAGS.load(SeqCst) & 1 << 0 != 0 {
            let (route_idx, params) = Self::match_route(&self.current_path);
            self.params = params;
            self.active_page = Self::mount_page(
                route_idx,
                &self.params,
                &self.elements.2,
            )?;
        }

        Ok(())
    }
}*/

/// Global state for the page component
///
/// Each component has the following sections recursively:
/// - Element array: array of all elements in the component, used for patching
/// - Prop state: state variables passed from parent component
/// - Reactive state: state variables that trigger re-renders when changed
/// - Derived state: state variables that are computed from reactive state
/// - Child component local state: recurse
/// - Fragment state: state variables for managing fragments (e.g., #if, #each)
///
/// All values except derived are user defined. Derived state is auto-generated.
pub struct Page {
    // Element array:
    elements: (
        Element,
        Element,
        Element,
        Element,
        Element,                     // 4
        IfElement<Element, Element>, // 5 - #if block
        EachElement<Element>,      // 6 - #each
        Element,
        Element, // 8
    ),

    // Prop state: none

    // Reactive state:
    counter: MutateTracker<i32>,        // id: 0
    my_struct: MutateTracker<MyStruct>, // id: 1

    // Derived state:
    counter_plus_one: i32,

    // Child component local state:
    button_1: Button,
}

impl Page {
    fn init(&mut self) {
        //let mut other_var = 42;
        //other_var += 1;

        *self.counter += 1;
    }

    pub fn new(parent_path: Vec<u32>) -> Result<Self, JsValue> {
        web_sys::console::log_1(&"Initializing Page component".into());

        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window");
        let body = document.body().expect("document should have a body");

        let el0 = document.create_element("div")?;
        body.append_child(&el0)?;

        let el1 = document.create_element("button")?;
        el0.append_child(&el1)?;

        let el2 = document.create_element("button")?;
        el0.append_child(&el2)?;

        let c1 = document.create_element("div")?;
        el0.append_child(&c1)?;

        let el3 = document.create_element("p")?;
        el0.append_child(&el3)?;

        let el4 = document.create_element("p")?;
        el0.append_child(&el4)?;

        // #if block
        let el5 = document.create_comment("");
        el0.append_child(&el5)?;

        let el5_if = document.create_element("p")?;
        el5_if.set_inner_html("Counter is greater than 5!");

        let el5_else = document.create_element("p")?;
        el5_else.set_inner_html("Counter is 5 or less.");
        el0.insert_before(&el5_else, Some(&el5))?;

        // 6 - #each block
        let el6 = document.create_comment("");
        el0.append_child(&el6)?;

        let el7 = document.create_element("div")?;
        el0.append_child(&el7)?;

        let el8 = document.create_element("p")?;
        el7.append_child(&el8)?;

        add_listener(&el1, "click", prepend_path(&parent_path, 1))?;

        add_listener(&el2, "click", prepend_path(&parent_path, 2))?;

        let elements = (
            el0,
            el1,
            el2,
            el3,
            el4,
            IfElement {
                comment: el5,
                condition: false, // needs to be initialized to something, will be updated in apply
                if_content: el5_if,
                else_content: el5_else,
            },
            EachElement {
                comment: el6,
                content: vec![],
            },
            el7,
            el8,
        );

        let mut new_page = Self {
            elements,
            counter: MutateTracker::new(0, 0),
            my_struct: MutateTracker::new(
                MyStruct {
                    a: 10,
                    b: "hello".to_string(),
                },
                1,
            ),
            counter_plus_one: 1, // needs to be initialized to something, will be updated in init
            button_1: Button::new(prepend_path(&parent_path, 3), c1)?,
        };

        new_page.init();

        DIRTY_FLAGS.store(u64::MAX, SeqCst); // mark all as dirty for initial render
        new_page.apply()?;

        Ok(new_page)
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
    pub fn proc(
        &mut self,
        e: web_sys::Event,
        mut target_path: Vec<u32>,
        _: (),
    ) -> Result<(), JsValue> {
        // Event handling
        let target = target_path.pop().unwrap();
        match e.type_().as_str() {
            "click" if target == 1 => {
                self.increment();
            }
            "click" if target == 2 => {
                (|state: &mut Page| state.update_struct(true))(self);
            }
            _ if target == 3 => {
                self.button_1.proc(e, target_path)?;
                let child_bindable_flags = DIRTY_FLAGS.load(SeqCst);
                DIRTY_FLAGS.store(0, SeqCst); // reset for parent processing

                // Update bindable props based on child changes
                if child_bindable_flags & 1 << 1 != 0 {
                    // Run user defined closure with bound function call
                    (|state: &mut Page, new_val| {
                        *state.counter = new_val;
                    })(self, *self.button_1.func_call);
                }
            }
            _ => {}
        }

        self.apply()?;

        Ok(())
    }

    /// Generate patches based on changes to reactive state and derived state
    ///
    /// The flag_exclude parameter is a bitmask of any bindable props that should
    /// not be included in the propagation to children to avoid feedback loops.
    pub fn apply(&mut self) -> Result<(), JsValue> {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window");

        // update derived
        if DIRTY_FLAGS.load(SeqCst) & 1 << 0 != 0 {
            self.counter_plus_one = *self.counter + 1;
            DIRTY_FLAGS.fetch_or(1 << 3, SeqCst);
        }

        // generate patches based on dirty flags
        let flag_snapshot = DIRTY_FLAGS.load(SeqCst);

        // counter changed
        if flag_snapshot & 1 << 0 != 0 {
            self.elements
                .1
                .set_inner_html(&format!("Counter: {}", *self.counter));

            self.elements
                .3
                .set_inner_html(&format!("Count: {}", *self.counter));

            // #if block
            let frag1_eval = *self.counter > 5;
            if frag1_eval != self.elements.5.condition {
                if self.elements.5.condition {
                    self.elements.5.if_content.remove();
                    self.elements.0.insert_before(
                        &self.elements.5.else_content,
                        Some(&self.elements.5.comment),
                    )?;
                } else {
                    self.elements
                        .0
                        .remove_child(&self.elements.5.else_content)?;
                    self.elements.0.insert_before(
                        &self.elements.5.if_content,
                        Some(&self.elements.5.comment),
                    )?;
                }
                self.elements.5.condition = frag1_eval;
            }

            // #each block
            // User expression: (0..counter)
            self.elements.6.content = diff_each_content(
                &self.elements.6.content,
                (0..*self.counter).collect::<Vec<i32>>(),
                self.elements.6.comment.clone(),
                |el| {
                    el.remove();
                    el.clone()
                },
                |item| {
                    let new_el = document.create_element("p")?;
                    new_el.set_inner_html(&format!("Number: {}", item));
                    Ok(new_el)
                },
                |item, anchor| {
                    self.elements
                        .0
                        .insert_before(item, Some(anchor))?;
                    Ok(())
                }
            )?;
        }

        // my_struct changed
        if flag_snapshot & 1 << 1 != 0 {
            self.elements
                .2
                .set_inner_html(&format!("Struct A: {} (Click to update)", self.my_struct.a));
            self.elements
                .4
                .set_inner_html(&format!("Struct B: {}", self.my_struct.b));
        }

        // counter_plus_one changed (derived)
        if flag_snapshot & 1 << 3 != 0 {
            self.elements
                .8
                .set_inner_html(&format!("Counter plus one: {}", self.counter_plus_one));
        }

        // Propagate to children

        // Button 1
        if flag_snapshot & 1 << 1 != 0 {
            DIRTY_FLAGS.store(0, SeqCst); // reset before child processing

            // Update props for button
            if flag_snapshot & 1 << 1 != 0 {
                self.button_1.text = self.my_struct.b.clone(); // my_struct.b is the given expression
                DIRTY_FLAGS.fetch_or(1 << 0, SeqCst); // manually mark prop as dirty
            }

            self.button_1.apply()?;
        }

        // Restore snapshot
        DIRTY_FLAGS.store(flag_snapshot, SeqCst);

        Ok(())
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
}

struct Button {
    // Element array:
    elements: (Element, Element, Element),

    // Props:
    text: String,
    // Bindable props (any func prop affecting state is automatically bindable):
    func_call: MutateTracker<i32>,

    // Reactive state:
    button_counter: MutateTracker<i32>,
}

impl Button {
    fn new(parent_path: Vec<u32>, el0: Element) -> Result<Self, JsValue> {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window");

        let el1 = document.create_element("button")?;
        el0.append_child(&el1)?;

        let el2 = document.create_element("button")?;
        el0.append_child(&el2)?;

        el2.set_inner_html("Set parent");

        add_listener(&el1, "click", prepend_path(&parent_path, 1))?;

        add_listener(&el2, "click", prepend_path(&parent_path, 2))?;

        let elements = (el0, el1, el2);

        let mut new_self = Self {
            elements,
            text: "".to_string(),
            func_call: MutateTracker::new(0, 1),
            button_counter: MutateTracker::new(0, 0),
        };

        DIRTY_FLAGS.store(u64::MAX, SeqCst); // mark all as dirty for initial render
        new_self.apply()?;

        Ok(new_self)
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
            self.elements
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
    elements: (Element,),
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

        Ok(Self { elements: (el0,) })
    }
}

struct UserPage {
    // Element array:
    elements: (Element,),

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
            elements: (el0,),
            user_id,
        })
    }
}*/
