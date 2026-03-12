use wasm_bindgen::JsValue;
use web_sys::{Element, Text};

use crate::*;
use std::{sync::atomic::Ordering::SeqCst, vec};

// User generated agnostic code
struct MyStruct {
    a: i32,
    b: String,
}

fn add<T: std::ops::Add<Output = T>>(a: T, b: T) -> T {
    a + b
}

pub struct PageState {
    // Prop state: none

    // Reactive state:
    counter: MutateTracker<i32>,        // id: 0
    my_struct: MutateTracker<MyStruct>, // id: 1

    // Derived state:
    counter_plus_one: i32,
}

impl ComponentState for PageState {
    fn init(&mut self) {
        //let mut other_var = 42;
        //other_var += 1;

        *self.counter += 1;
    }

    fn new() -> Self {
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

    fn update_derived(&mut self) {
        if DIRTY_FLAGS.load(SeqCst) & 1 << 0 != 0 {
            self.counter_plus_one = *self.counter + 1;
            DIRTY_FLAGS.fetch_or(1 << 3, SeqCst);
        }
    }
}

// User defined functions for Page
impl PageState {
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

pub struct PageRootFrag {
    a: Element,
    b: Element,
    c: Element,
    d: Element,
    e: Element,                // 4
    f: IfElement<If1Content>,  // 5 - #if block
    g: EachElement<EachFrag1>, // 6 - #each
    h: Element,
    i: Element,
    j: Component<ButtonRootFrag>,
}

impl RootFragment for PageRootFrag {
    type State = PageState;

    fn new(state: &Self::State, scope: (), current_path: &Vec<u32>) -> Result<Self, JsValue> {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window");

        let el0 = document.create_element("div")?;
        let el1 = document.create_element("button")?;
        let el2 = document.create_element("button")?;
        let el3 = document.create_element("p")?;
        let el4 = document.create_element("p")?;
        let el5 = IfElement::new(state, scope, &prepend_path(current_path, 5))?;
        let el6 = EachElement::new(state, scope, &prepend_path(current_path, 6))?;
        let el7 = document.create_element("div")?;
        let el8 = document.create_element("p")?;
        let el9 = Component::<ButtonRootFrag>::new(&prepend_path(current_path, 3))?;

        // target paths are static and unique to each fragment
        // listeners are in new() to preserve them if moved (unmounted & remounted)
        add_listener(&el1, "click", prepend_path(current_path, 1))?;
        add_listener(&el2, "click", prepend_path(current_path, 2))?;

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

    fn mount(&self, add_method: impl AddMethod) -> Result<(), JsValue> {
        add_method(&self.a)?;
        self.a.append_child(&self.b)?;
        self.a.append_child(&self.c)?;
        self.j.mount(child_append_closure(&self.a))?; // TODO: remove parent path & update button
        self.a.append_child(&self.d)?;
        self.a.append_child(&self.e)?;
        self.f.mount(&self.a, child_append_closure(&self.a))?;
        self.g.mount(&self.a, child_append_closure(&self.a))?;
        self.a.append_child(&self.h)?;
        self.h.append_child(&self.i)?;
        Ok(())
    }

    fn proc(
        &mut self,
        state: &mut Self::State,
        scope: (),
        e: web_sys::Event,
        mut target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        let target = target_path.pop().unwrap();
        web_sys::console::log_1(
            &format!(
                "Target is {}, event type: {}, target path: {:?}",
                target,
                e.type_(),
                target_path
            )
            .into(),
        );
        match e.type_().as_str() {
            "click" if target == 1 => {
                state.increment();
            }
            "click" if target == 2 => {
                (|| state.update_struct(true))();
            }
            // target is in button component
            _ if target == 3 => {
                self.j.proc(e, target_path, scope)?;
                let child_bindable_flags = DIRTY_FLAGS.load(SeqCst);
                DIRTY_FLAGS.store(0, SeqCst); // reset for parent processing

                // Update bindable props based on child changes
                if child_bindable_flags & 1 << 1 != 0 {
                    // Ex.
                    // state.bindable_prop = self.button_1.some_prop;
                    // DIRTY_FLAGS.fetch_or(1 << 0, SeqCst); // manually mark parent prop as dirty if it was changed by child

                    // Run user defined closure with bound function call
                    (|new_val| {
                        *state.counter = new_val;
                    })(self.j.state.func_call);
                }
            }
            _ if target == 6 => {
                // Example of an #each block nested handler
                self.g.proc(state, scope, e, target_path)?;
            }
            _ => {}
        }

        Ok(())
    }

    fn update(&mut self, state: &mut Self::State, scope: (), flags: u64) -> Result<(), JsValue> {
        web_sys::console::log_1(&format!("Updating PageRootFrag with flags: {:b}", flags).into());
        // counter changed
        if flags & 1 << 0 != 0 {
            self.b
                .set_inner_html(&format!("Counter: {}", *state.counter));

            self.d.set_inner_html(&format!("Count: {}", *state.counter));
        }

        self.f.update(&self.a, state, scope, flags)?;
        self.g.update(&self.a, state, scope, flags)?;

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
                self.j.state.text = state.my_struct.b.clone(); // my_struct.b is the given expression
                DIRTY_FLAGS.fetch_or(1 << 0, SeqCst); // manually mark prop as dirty
            }

            self.j.apply()?;
        }

        Ok(())
    }

    fn unmount(&self) {
        self.a.remove();
        //self.j.unmount();
    }
}

enum If1Content {
    If(IfBranch1),
    Else(IfBranch2),
}

impl IfContentTrait for If1Content {
    type Scope<'a> = ();
    type State = PageState;

    fn branch_changed(&self, state: &Self::State, _scope: Self::Scope<'_>, flags: u64) -> bool {
        if flags & 1 << 0 != 0 {
            match self {
                Self::If(_) if *state.counter > 5 => false,
                Self::Else(_) if !(*state.counter > 5) => false,
                _ => true,
            }
        } else {
            false
        }
    }

    fn new(state: &Self::State, scope: Self::Scope<'_>, current_path: &Vec<u32>) -> Result<Self, JsValue> {
        Ok(if *state.counter > 5 {
            Self::If(IfBranch1::new(state, scope, current_path)?)
        } else {
            Self::Else(IfBranch2::new(state, scope, current_path)?)
        })
    }

    fn mount(&self, parent: &Element, add_method: impl AddMethod) -> Result<(), JsValue> {
        match self {
            Self::If(contents) => contents.mount(parent, add_method),
            Self::Else(contents) => contents.mount(parent, add_method),
        }
    }

    fn proc(
        &mut self,
        state: &mut Self::State,
        scope: Self::Scope<'_>,
        e: web_sys::Event,
        target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        // this could call into the branches but in this example it wouldn't do anything anyway
        match self {
            Self::If(fragment) => fragment.proc(state, scope, e, target_path),
            Self::Else(_) => Ok(()),
        }
    }

    fn update(
        &mut self,
        parent: &Element,
        state: &Self::State,
        scope: Self::Scope<'_>,
        flags: u64,
    ) -> Result<(), JsValue> {
        // Check for changes in content of active branch
        match self {
            Self::If(contents) => contents.update(parent, state, scope, flags),
            Self::Else(contents) => contents.update(parent, state, scope, flags),
        }
    }

    fn unmount(&self) {
        match self {
            Self::If(contents) => contents.unmount(),
            Self::Else(contents) => contents.unmount(),
        }
    }
}

struct IfBranch1 {
    a: Element,
    b: Text,
    c: Text,
}

impl GenericFragment for IfBranch1 {
    type State = PageState;
    type Scope<'a> = ();

    fn new(state: &Self::State, scope: Self::Scope<'_>, current_path: &Vec<u32>) -> Result<Self, JsValue> {
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

    fn mount(&self, _parent: &Element, add_method: impl AddMethod) -> Result<(), JsValue> {
        add_method(&self.a)?;
        self.a.append_child(&self.b)?;
        self.a.append_child(&self.c)?;
        Ok(())
    }

    fn proc(
        &mut self,
        state: &mut Self::State,
        scope: Self::Scope<'_>,
        e: web_sys::Event,
        target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        // Handle events for content inside #if block if needed
        Ok(())
    }

    fn update(
        &mut self,
        _parent: &Element,
        state: &Self::State,
        _scope: Self::Scope<'_>,
        flags: u64,
    ) -> Result<(), JsValue> {
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
    b: Component<ButtonRootFrag>,
}

impl GenericFragment for IfBranch2 {
    type State = PageState;
    type Scope<'a> = ();

    fn new(
        state: &Self::State,
        scope: Self::Scope<'_>,
        current_path: &Vec<u32>,
    ) -> Result<Self, JsValue> {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window exists");

        let el5_else = document.create_element("p")?;
        el5_else.set_inner_html("Counter is 5 or less.");
        let b = Component::<ButtonRootFrag>::new(&prepend_path(current_path, 1))?; // target path is in reverse order!!

        Ok(Self { a: el5_else, b })
    }

    fn mount(&self, parent: &Element, add_method: impl AddMethod) -> Result<(), JsValue> {
        add_method(&self.a)?;
        Ok(())
    }

    fn proc(
        &mut self,
        _state: &mut Self::State,
        _scope: Self::Scope<'_>,
        _e: web_sys::Event,
        _target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        // Handle events for content inside #if block if needed
        Ok(())
    }

    fn update(
        &mut self,
        _parent: &Element,
        _state: &Self::State,
        _scope: Self::Scope<'_>,
        _flags: u64,
    ) -> Result<(), JsValue> {
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
    b: IfElement<If2Content>,
}

impl EachContentTrait for EachFrag1 {
    type Item = i32;
    type Scope<'a> = ();
    type State = PageState;

    fn generate(
        state: &Self::State,
        _scope: Self::Scope<'_>,
        flags: u64,
    ) -> Option<Vec<Self::Item>> {
        if flags & 1 << 0 != 0 {
            Some((0..*state.counter).collect())
        } else {
            None
        }
    }

    fn new(state: &Self::State, scope: (Self::Scope<'_>, &Self::Item), current_path: &Vec<u32>) -> Result<Self, JsValue> {
        let (_, item) = scope;

        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window exists");

        let node_1 = document.create_element("p")?;
        node_1.set_inner_html(&format!("Number: {}", item));
        let node_2 = IfElement::new(state, scope, current_path)?;
        Ok(Self {
            a: node_1,
            b: node_2,
        })
    }

    fn mount(&self, parent: &Element, add_method: impl AddMethod) -> Result<(), JsValue> {
        add_method(&self.a)?;
        Ok(())
    }

    fn proc(
        &self,
        _state: &Self::State,
        _scope: (Self::Scope<'_>, &Self::Item),
        _e: web_sys::Event,
        _target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        // Handle events for content inside #each block if needed
        // TODO: unwrap target & go deeper

        Ok(())
    }

    fn update(
        &mut self,
        _parent: &Element,
        _state: &Self::State,
        _scope: (Self::Scope<'_>, &Self::Item),
        _flags: u64,
    ) -> Result<(), JsValue> {
        // No reactive stuff inside, otherwise updates would go here
        Ok(())
    }

    fn unmount(&self) {
        self.a.remove();
        self.b.unmount();
    }
}

// TODO: this should be an enum
#[derive(Clone)]
struct If2Content {
    a: Element,
}

impl IfContentTrait for If2Content {
    type Scope<'a> = ((), &'a i32);
    type State = PageState;

    fn branch_changed(&self, _state: &Self::State, _scope: Self::Scope<'_>, _flags: u64) -> bool {
        false // no dynamic content in this example, so branch never changes after initial render
    }

    fn new(_state: &Self::State, scope: Self::Scope<'_>, current_path: &Vec<u32>) -> Result<Self, JsValue> {
        let (_, a_scope) = scope;
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window exists");

        let a = document.create_element("p")?;
        a.set_inner_html(&format!("{}", a_scope));

        Ok(Self { a })
    }

    fn mount(&self, parent: &Element, add_method: impl AddMethod) -> Result<(), JsValue> {
        add_method(&self.a)?;
        Ok(())
    }

    fn proc(
        &mut self,
        state: &mut Self::State,
        scope: Self::Scope<'_>,
        e: web_sys::Event,
        mut target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        // Handle events for content inside #each block if needed
        let (_, item) = scope;
        let target = target_path.pop().unwrap();
        match e.type_().as_str() {
            "click" if target == 0 => {
                (|| *state.counter += item)();
            }
            _ => {}
        }

        Ok(())
    }

    fn update(
        &mut self,
        _parent: &Element,
        _state: &Self::State,
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

struct ButtonState {
    // Props:
    text: String,
    // Bindable props (any func prop affecting state is automatically bindable):
    func_call: i32,

    // Reactive state:
    button_counter: MutateTracker<i32>,
}

impl ComponentState for ButtonState {
    fn init(&mut self) {
        // Initialize any state if needed
    }

    fn new() -> Self {
        let mut state = ButtonState {
            text: "".to_string(),
            func_call: 0,
            button_counter: MutateTracker::new(0, 0),
        };
        state.init();
        state
    }

    fn update_derived(&mut self) {
        // Update any derived state if needed (none in this example)
    }
}

struct ButtonRootFrag {
    a: Element,
    b: Element,
    c: Element,
}

impl RootFragment for ButtonRootFrag {
    type State = ButtonState;

    fn new(state: &Self::State, scope: (), current_path: &Vec<u32>) -> Result<Self, JsValue>
    where
        Self: Sized,
    {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window");

        let a = document.create_element("div")?;
        let b = document.create_element("button")?;
        let c = document.create_element("button")?;
        c.set_inner_html("Set parent");

        add_listener(&b, "click", prepend_path(current_path, 1))?; // target path is in reverse order!!
        add_listener(&c, "click", prepend_path(current_path, 2))?;

        Ok(Self { a, b, c })
    }

    fn mount(&self, add_method: impl AddMethod) -> Result<(), JsValue> {
        add_method(&self.a)?;
        self.a.append_child(&self.b)?;
        self.a.append_child(&self.c)?;

        Ok(())
    }

    fn proc(
        &mut self,
        state: &mut Self::State,
        scope: (),
        e: web_sys::Event,
        mut target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        let target = target_path.pop().unwrap();
        match e.type_().as_str() {
            "click" if target == 1 => {
                *state.button_counter += 1;
            }
            "click" if target == 2 => {
                state.func_call = *state.button_counter;
                DIRTY_FLAGS.fetch_or(1 << 1, SeqCst); // mark func_call prop as dirty to propagate to parent
            }
            _ => {}
        }

        Ok(())
    }

    fn update(&mut self, state: &mut Self::State, scope: (), flags: u64) -> Result<(), JsValue> {
        if flags & 1 << 0 != 0 {
            self.b
                .set_inner_html(&format!("{}: {}", state.text, *state.button_counter));
        }

        Ok(())
    }

    fn unmount(&self) {
        self.a.remove();
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
