use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, Text};

use crate::{
    AddMethod, DIRTY_FLAGS, EachElement, IfElement, MutateTracker, add_listener, child_append_closure, diff_each_content, hash_item, prepend_path, state
};
use core::panic;
use std::{sync::atomic::Ordering::SeqCst, vec};

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
    contents: (Element),     // element array for router component
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
            contents: (el0),
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
                &self.contents.2,
            )?;
        }

        Ok(())
    }
}*/

enum IfContent {
    If((Element, Text, Text)),
    Else(Element),
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
    // Element array:
    contents: (
        Element,
        Element,
        Element,
        Element,
        Element,                   // 4
        IfElement<IfContent>,      // 5 - #if block
        EachElement<Element, i32>, // 6 - #each
        Element,
        Element, //
        Button,
    ),

    state: PageState,
}

impl Page {
    pub fn new() -> Result<Self, JsValue> {
        web_sys::console::log_1(&"Initializing Page component".into());

        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window");

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

        let el0 = document.create_element("div")?;
        let el1 = document.create_element("button")?;
        let el2 = document.create_element("button")?;
        let el3 = document.create_element("p")?;
        let el4 = document.create_element("p")?;
        let el5 = if_frag_1_create(&state)?;
        let el6 = each_frag_1_create(&state)?;
        let el7 = document.create_element("div")?;
        let el8 = document.create_element("p")?;
        let el9 = Button::new()?;

        let contents = (el0, el1, el2, el3, el4, el5, el6, el7, el8, el9);

        let mut new_page = Self { contents, state };

        DIRTY_FLAGS.store(u64::MAX, SeqCst); // mark all as dirty for initial render
        new_page.apply()?;

        Ok(new_page)
    }

    pub fn mount(&self, parent_path: Vec<u32>, parent: &Element) -> Result<(), JsValue> {
        let contents = &self.contents;
        parent.append_child(&contents.0)?;
        contents.0.append_child(&contents.1)?;
        contents.0.append_child(&contents.2)?;
        contents.9.mount(prepend_path(&parent_path, 9), child_append_closure(&contents.0))?;
        contents.0.append_child(&contents.3)?;
        contents.0.append_child(&contents.4)?;
        if_frag_1_mount(&contents.0, &contents.5)?;
        each_frag_1_mount(&contents.0, &contents.6)?;
        contents.0.append_child(&contents.7)?;
        contents.7.append_child(&contents.8)?;
        add_listener(&contents.1, "click", prepend_path(&parent_path, 1))?;
        add_listener(&contents.2, "click", prepend_path(&parent_path, 2))?;
        Ok(())
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
                self.state.increment();
            }
            "click" if target == 2 => {
                (|state: &mut PageState| state.update_struct(true))(&mut self.state);
            }
            // target is in button component
            _ if target == 3 => {
                self.contents.9.proc(e, target_path)?;
                let child_bindable_flags = DIRTY_FLAGS.load(SeqCst);
                DIRTY_FLAGS.store(0, SeqCst); // reset for parent processing

                // Update bindable props based on child changes
                if child_bindable_flags & 1 << 1 != 0 {
                    // Ex.
                    // state.bindable_prop = self.button_1.some_prop;
                    // DIRTY_FLAGS.fetch_or(1 << 0, SeqCst); // manually mark parent prop as dirty if it was changed by child

                    // Run user defined closure with bound function call
                    (|state: &mut Page, new_val| {
                        *state.state.counter = new_val;
                    })(self, *self.contents.9.func_call);
                }
            }
            _ if target == 6 => {
                // Example of an #each block nested handler

                // Get item specific scoped variable
                let target = target_path.pop().unwrap();
                let item = self.contents.6.content[target as usize].2; // get the item from the #each content array based on index

                // Find target element in #each content
                let target = target_path.pop().unwrap();
                match e.type_().as_str() {
                    "click" if target == 1 => {
                        // User closure:
                        (|state: &mut Page| {
                            *state.state.counter = item;
                        })(self);
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

        self.apply()?;

        Ok(())
    }

    /// Generate patches based on changes to reactive state and derived state
    ///
    /// The flag_exclude parameter is a bitmask of any bindable props that should
    /// not be included in the propagation to children to avoid feedback loops.
    pub fn apply(&mut self) -> Result<(), JsValue> {
        let contents = &mut self.contents;
        let state = &mut self.state;

        // update derived
        if DIRTY_FLAGS.load(SeqCst) & 1 << 0 != 0 {
            state.counter_plus_one = *state.counter + 1;
            DIRTY_FLAGS.fetch_or(1 << 3, SeqCst);
        }

        // generate patches based on dirty flags
        let flag_snapshot = DIRTY_FLAGS.load(SeqCst);

        // counter changed
        if flag_snapshot & 1 << 0 != 0 {
            contents
                .1
                .set_inner_html(&format!("Counter: {}", *state.counter));

            contents
                .3
                .set_inner_html(&format!("Count: {}", *state.counter));
        }

        if_frag_1_update(&contents.0, &state, &mut contents.5, flag_snapshot)?;

        each_frag_1_update(&contents.0, &state, &mut contents.6, flag_snapshot)?;

        // my_struct changed
        if flag_snapshot & 1 << 1 != 0 {
            contents.2.set_inner_html(&format!(
                "Struct A: {} (Click to update)",
                state.my_struct.a
            ));
            contents
                .4
                .set_inner_html(&format!("Struct B: {}", state.my_struct.b));
        }

        // counter_plus_one changed (derived)
        if flag_snapshot & 1 << 3 != 0 {
            contents
                .8
                .set_inner_html(&format!("Counter plus one: {}", state.counter_plus_one));
        }

        // Propagate to children

        // Button 1
        if flag_snapshot & 1 << 1 != 0 {
            DIRTY_FLAGS.store(0, SeqCst); // reset before child processing

            // Update props for button
            if flag_snapshot & 1 << 1 != 0 {
                contents.9.text = state.my_struct.b.clone(); // my_struct.b is the given expression
                DIRTY_FLAGS.fetch_or(1 << 0, SeqCst); // manually mark prop as dirty
            }

            contents.9.apply()?;
        }

        // Restore snapshot
        DIRTY_FLAGS.store(flag_snapshot, SeqCst);

        Ok(())
    }
}

fn if_frag_1_branch_1_create(state: &PageState) -> Result<(Element, Text, Text), JsValue> {
    let window = web_sys::window().expect("no global window exists");
    let document = window.document().expect("no document on window exists");

    let el5_if = document.create_element("p")?;
    let el5_text_1 = document.create_text_node("Counter is greater than 5! ");
    let el5_text_2 = document.create_text_node(&format!("{}", state.my_struct.a));

    Ok((el5_if, el5_text_1, el5_text_2))
}

fn if_frag_1_branch_1_mount(
    parent: &web_sys::Node,
    comment: &web_sys::Comment,
    contents: &(Element, Text, Text),
) -> Result<(), JsValue> {
    parent.insert_before(&contents.0, Some(&comment))?;
    contents.0.append_child(&contents.1)?;
    contents.0.append_child(&contents.2)?;
    Ok(())
}

fn if_frag_1_branch_1_update(
    state: &PageState,
    contents: &(Element, Text, Text),
    flags: u64,
) -> Result<(), JsValue> {
    if flags & 1 << 1 != 0 {
        contents
            .2
            .set_text_content(Some(&format!("{}", state.my_struct.a)));
    }
    Ok(())
}

fn if_frag_1_branch_1_unmount(contents: &(Element, Text, Text)) {
    let (el5_if, el5_text_1, el5_text_2) = contents;
    el5_text_1.remove();
    el5_text_2.remove();
    el5_if.remove();
}

fn if_frag_1_branch_2_create(state: &PageState) -> Result<Element, JsValue> {
    let window = web_sys::window().expect("no global window exists");
    let document = window.document().expect("no document on window exists");

    let el5_else = document.create_element("p")?;
    el5_else.set_inner_html("Counter is 5 or less.");

    Ok(el5_else)
}

fn if_frag_1_branch_2_mount(
    parent: &web_sys::Node,
    comment: &web_sys::Comment,
    contents: &Element,
) -> Result<(), JsValue> {
    parent.insert_before(contents, Some(comment))?;
    Ok(())
}

fn if_frag_1_branch_2_unmount(contents: &Element) {
    contents.remove();
}

fn if_frag_1_create(state: &PageState) -> Result<IfElement<IfContent>, JsValue> {
    let window = web_sys::window().expect("no global window exists");
    let document = window.document().expect("no document on window exists");

    let (content_enum, active_branch) = if *state.counter > 5 {
        (IfContent::If(if_frag_1_branch_1_create(state)?), 0)
    } else {
        (IfContent::Else(if_frag_1_branch_2_create(state)?), 1)
    };

    Ok(IfElement {
        comment: document.create_comment(""),
        active_branch,
        content_enum,
    })
}

fn if_frag_1_mount(parent: &web_sys::Node, frag: &IfElement<IfContent>) -> Result<(), JsValue> {
    parent.append_child(&frag.comment)?;
    match frag.content_enum {
        IfContent::If(ref contents) => {
            if_frag_1_branch_1_mount(parent, &frag.comment, contents)?;
        }
        IfContent::Else(ref el5_else) => {
            if_frag_1_branch_2_mount(parent, &frag.comment, el5_else)?;
        }
    }

    Ok(())
}

fn if_frag_1_update(
    parent: &web_sys::Node,
    state: &PageState,
    frag: &mut IfElement<IfContent>,
    flags: u64,
) -> Result<(), JsValue> {
    // Check for branch changes
    if flags & 1 << 0 != 0 {
        let active_branch = if *state.counter > 5 { 0 } else { 1 };
        if active_branch != frag.active_branch {
            // Unmount old content
            match frag.content_enum {
                IfContent::If(ref old_if) => {
                    if_frag_1_branch_1_unmount(old_if);
                }
                IfContent::Else(ref old_else) => {
                    if_frag_1_branch_2_unmount(old_else);
                }
            }

            // Mount new content
            (frag.content_enum, frag.active_branch) = if *state.counter > 5 {
                let new_if = if_frag_1_branch_1_create(state)?;
                if_frag_1_branch_1_mount(parent, &frag.comment, &new_if)?;
                (IfContent::If(new_if), 0)
            } else {
                let new_else = if_frag_1_branch_2_create(state)?;
                if_frag_1_branch_2_mount(parent, &frag.comment, &new_else)?;
                (IfContent::Else(new_else), 1)
            };
        }
    }

    // Check for changes in content of active branch
    match &frag.content_enum {
        IfContent::If(contents) => {
            if_frag_1_branch_1_update(state, contents, flags)?;
        }
        IfContent::Else(_) => {}
    }

    Ok(())
}

fn if_frag_1_unmount(frag: &IfElement<IfContent>) {
    match frag.content_enum {
        IfContent::If(ref contents) => {
            if_frag_1_branch_1_unmount(contents);
        }
        IfContent::Else(ref el5_else) => {
            if_frag_1_branch_2_unmount(el5_else);
        }
    }
    frag.comment.remove();
}

fn each_frag_1_create(state: &PageState) -> Result<EachElement<Element, i32>, JsValue> {
    let window = web_sys::window().expect("no global window exists");
    let document = window.document().expect("no document on window exists");

    let content = (0..*state.counter)
        .map(|item| {
            let node_1 = document.create_element("p")?;
            node_1.set_inner_html(&format!("Number: {}", item));
            Ok((hash_item(&item), node_1, item))
        })
        .collect::<Result<Vec<(u64, Element, i32)>, JsValue>>()?;

    Ok(EachElement {
        comment: document.create_comment(""),
        content,
    })
}

fn each_frag_1_mount(
    parent: &web_sys::Node,
    frag: &EachElement<Element, i32>,
) -> Result<(), JsValue> {
    parent.append_child(&frag.comment)?;
    for (_, node_1, _) in frag.content.iter() {
        parent.insert_before(node_1, Some(&frag.comment))?;
    }
    Ok(())
}

fn each_frag_1_update(
    parent: &web_sys::Node,
    state: &PageState,
    frag: &mut EachElement<Element, i32>,
    flags: u64,
) -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global window exists");
    let document = window.document().expect("no document on window exists");

    if flags & 1 << 0 != 0 {
        frag.content = diff_each_content(
            &frag.content,
            (0..*state.counter).collect::<Vec<i32>>(),
            parent,
            frag.comment.clone(),
            each_frag_1_content_unmount,
            |item| {
                let node_1 = document.create_element("p")?;
                node_1.set_inner_html(&format!("Number: {}", item));
                Ok(node_1)
            },
            each_frag_1_content_mount,
        )?;
    }

    Ok(())
}

fn each_frag_1_content_unmount(content: &Element) {
    content.remove();
}

fn each_frag_1_content_mount(
    parent: &web_sys::Node,
    anchor: &web_sys::Node,
    content: &Element,
) -> Result<(), JsValue> {
    parent.insert_before(content, Some(anchor))?;
    Ok(())
}

fn each_frag_1_unmount(frag: EachElement<Element, i32>) {
    for (_, content, _) in frag.content.iter() {
        each_frag_1_content_unmount(content);
    }
    frag.comment.remove();
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
