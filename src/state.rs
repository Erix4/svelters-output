use wasm_bindgen::JsValue;
use web_sys::{Element, Text};

use crate::*;
use std::{rc::Rc, sync::atomic::Ordering::SeqCst, vec};

pub struct TopRouterState {
    // top level layout state would go here (none in this example)
}

pub mod page {
    use crate::state::button::ButtonRootFrag;

    use super::*;

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
        type Props = (); // no props in page root!

        fn init(&mut self) {
            //let mut other_var = 42;
            //other_var += 1;

            *self.counter += 1;
        }

        fn new(props: Self::Props) -> Self {
            PageState {
                counter: MutateTracker::new(0, 0),
                my_struct: MutateTracker::new(
                    MyStruct {
                        a: 10,
                        b: "hello".to_string(),
                    },
                    1,
                ),
                counter_plus_one: 0,
            }
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

    pub struct CPageRootFrag {
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

    impl GenericFragment for CPageRootFrag {
        type State = PageState;
        type Scope<'a> = ();

        fn new(state: &Self::State, scope: (), current_path: &Vec<u32>) -> Result<Self, JsValue> {
            let window = web_sys::window().expect("no global window exists");
            let document = window.document().expect("no document on window");

            let el0 = document.create_element("div")?;
            let el1 = document.create_element("button")?;
            el1.set_inner_html(&format!("Counter: {}", *state.counter)); // technically this would be in its own textnode
            let el2 = document.create_element("button")?;
            let el3 = document.create_element("p")?;
            el3.set_inner_html(&format!("Count: {}", *state.counter));
            let el4 = document.create_element("p")?;
            let el5 = IfElement::new(state, scope, &prepend_path(current_path, 5))?;
            let el6 = EachElement::new(state, scope, &prepend_path(current_path, 6))?;
            let el7 = document.create_element("div")?;
            let el8 = document.create_element("p")?;
            let mut el9_state =
                <ButtonRootFrag as GenericFragment>::State::startup((state.my_struct.b.clone(),)); // create state with props
            DIRTY_FLAGS.fetch_or(1 << 0, SeqCst);
            let el9 = Component::<ButtonRootFrag>::new(el9_state, &prepend_path(current_path, 3))?;
            el4.set_inner_html(&format!("Struct B: {}", state.my_struct.b));
            el8.set_inner_html(&format!("Counter plus one: {}", state.counter_plus_one));
            el2.set_inner_html(&format!(
                "Struct A: {} (Click to update)",
                state.my_struct.a
            ));
            el4.set_inner_html(&format!("Struct B: {}", state.my_struct.b));

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

        fn mount(&mut self, parent: &Element, add_method: impl AddMethod) -> Result<(), JsValue> {
            add_method(&self.a)?;
            self.a.append_child(&self.b)?;
            self.a.append_child(&self.c)?;
            self.j.mount(&self.a, child_append_closure(&self.a))?; // TODO: remove parent path & update button
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
                    self.j.proc(scope, e, target_path)?;
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

        fn update(
            &mut self,
            parent: &Element,
            state: &Self::State,
            scope: (),
            flags: u64,
        ) -> Result<(), JsValue> {
            web_sys::console::log_1(
                &format!("Updating PageRootFrag with flags: {:b}", flags).into(),
            );
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

        fn new(
            state: &Self::State,
            scope: Self::Scope<'_>,
            current_path: &Vec<u32>,
        ) -> Result<Self, JsValue> {
            Ok(if *state.counter > 5 {
                Self::If(IfBranch1::new(state, scope, current_path)?)
            } else {
                Self::Else(IfBranch2::new(state, scope, current_path)?)
            })
        }

        fn mount(&mut self, parent: &Element, add_method: impl AddMethod) -> Result<(), JsValue> {
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

        fn new(
            state: &Self::State,
            scope: Self::Scope<'_>,
            current_path: &Vec<u32>,
        ) -> Result<Self, JsValue> {
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

        fn mount(&mut self, _parent: &Element, add_method: impl AddMethod) -> Result<(), JsValue> {
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
            let mut b_state =
                <ButtonRootFrag as GenericFragment>::State::startup((state.my_struct.b.clone(),));
            DIRTY_FLAGS.fetch_or(1 << 0, SeqCst);
            let b = Component::<ButtonRootFrag>::new(b_state, &prepend_path(current_path, 1))?; // target path is in reverse order!!

            Ok(Self { a: el5_else, b })
        }

        fn mount(&mut self, parent: &Element, add_method: impl AddMethod) -> Result<(), JsValue> {
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

        fn new(
            state: &Self::State,
            scope: (Self::Scope<'_>, &Self::Item),
            current_path: &Vec<u32>,
        ) -> Result<Self, JsValue> {
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

        fn mount(&mut self, parent: &Element, add_method: impl AddMethod) -> Result<(), JsValue> {
            add_method(&self.a)?;
            Ok(())
        }

        fn proc(
            &mut self,
            _state: &mut Self::State,
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
    struct If2Content {
        a: Element,
    }

    impl IfContentTrait for If2Content {
        type Scope<'a> = ((), &'a i32);
        type State = PageState;

        fn branch_changed(
            &self,
            _state: &Self::State,
            _scope: Self::Scope<'_>,
            _flags: u64,
        ) -> bool {
            false // no dynamic content in this example, so branch never changes after initial render
        }

        fn new(
            _state: &Self::State,
            scope: Self::Scope<'_>,
            current_path: &Vec<u32>,
        ) -> Result<Self, JsValue> {
            let (_, a_scope) = scope;
            let window = web_sys::window().expect("no global window exists");
            let document = window.document().expect("no document on window exists");

            let a = document.create_element("p")?;
            a.set_inner_html(&format!("{}", a_scope));

            Ok(Self { a })
        }

        fn mount(&mut self, parent: &Element, add_method: impl AddMethod) -> Result<(), JsValue> {
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
}

mod button {
    use super::*;

    pub struct ButtonState {
        // Props:
        pub text: String,
        // Bindable props (any func prop affecting state is automatically bindable):
        pub func_call: i32,

        // Reactive state:
        pub button_counter: MutateTracker<i32>,
    }

    impl ComponentState for ButtonState {
        type Props = (String,); // text prop

        fn init(&mut self) {
            // Initialize any state if needed
        }

        fn new(props: Self::Props) -> Self {
            ButtonState {
                text: props.0,
                func_call: 0,
                button_counter: MutateTracker::new(0, 0),
            }
        }

        fn update_derived(&mut self) {
            // Update any derived state if needed (none in this example)
        }
    }

    pub struct ButtonRootFrag {
        a: Element,
        b: Element,
        c: Element,
        d: Snippet<Snippet1<()>, Snippet1Peel>,
        //e: Box<dyn Snippet2>,
    }

    impl GenericFragment for ButtonRootFrag {
        type State = ButtonState;
        type Scope<'a> = ();

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
            b.set_inner_html(&format!("{}: {}", state.text, *state.button_counter));
            let d = Snippet::<Snippet1<()>, Snippet1Peel>::new(
                state,
                scope,
                &prepend_path(current_path, 3),
                (
                    Box::new(|_, state| *state.button_counter),
                    Box::new(|_, state| state.text.clone()),
                ),
                (1 << 0, 1 << 0),
                Snippet1Peel,
            )?;

            add_listener(&b, "click", prepend_path(current_path, 1))?; // target path is in reverse order!!
            add_listener(&c, "click", prepend_path(current_path, 2))?;

            Ok(Self { a, b, c, d })
        }

        fn mount(&mut self, parent: &Element, add_method: impl AddMethod) -> Result<(), JsValue> {
            add_method(&self.a)?;
            self.a.append_child(&self.b)?;
            self.a.append_child(&self.c)?;
            self.d.mount(&self.a, child_append_closure(&self.a))?;

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
                _ if target == 3 => {
                    self.d.proc(state, scope, e, target_path)?;
                }
                _ => {}
            }

            Ok(())
        }

        fn update(
            &mut self,
            parent: &Element,
            state: &Self::State,
            scope: (),
            flags: u64,
        ) -> Result<(), JsValue> {
            if flags & 1 << 0 != 0 {
                self.b
                    .set_inner_html(&format!("{}: {}", state.text, *state.button_counter));
            }

            self.d.update(parent, state, scope, flags)?;

            Ok(())
        }

        fn unmount(&self) {
            self.a.remove();
        }
    }

    struct Snippet1Peel;

    impl PeelFn<Snippet1<()>> for Snippet1Peel {
        fn call<'a>(
            &self,
            scope: <Snippet1<()> as SnippetContentTrait>::CallScope<'a>,
        ) -> <Snippet1<()> as SnippetContentTrait>::Scope<'a> {
            // peel CallScope into Scope — both are () here, so identity
            scope
        }
    }

    struct Snippet1<CallScope> {
        a: Element,
        b: Text,
        phantom: std::marker::PhantomData<CallScope>, // to hold the generic type without actually using it
    }

    impl<CallScope: Copy> SnippetContentTrait for Snippet1<CallScope> {
        type State = ButtonState;
        type Scope<'a> = ();
        type CallScope<'a> = CallScope;
        type Props = (i32, String); // single item tuple: (i32,)
        type PropClosures = (
            Box<dyn Fn(Self::Scope<'_>, &Self::State) -> i32>,
            Box<dyn Fn(Self::Scope<'_>, &Self::State) -> String>,
        );
        type PropMasks = (u64, u64); // bitmask for each prop to indicate which ones are affected by an update

        fn init_props(
            state: &Self::State,
            scope: Self::Scope<'_>,
            closures: &Self::PropClosures,
        ) -> Self::Props {
            let prop1 = closures.0(scope, state);
            let prop2 = closures.1(scope, state);
            (prop1, prop2)
        }
        fn update_props(
            props: &mut Self::Props,
            state: &Self::State,
            scope: Self::Scope<'_>,
            closures: &Self::PropClosures,
            masks: Self::PropMasks,
            flags: &mut u64,
        ) {
            if *flags & masks.0 != 0 {
                props.0 = closures.0(scope, state);
                *flags |= 1 << 3; // if the new value is different from the old value, mark the prop as dirty for child propagation
            }
            if *flags & masks.1 != 0 {
                props.1 = closures.1(scope, state);
                *flags |= 1 << 4;
            }
        }

        fn new(
            state: &Self::State,
            scope: (Self::Scope<'_>, &Self::Props),
            current_path: &Vec<u32>,
        ) -> Result<Self, JsValue> {
            let (_, (n_prop, s_prop)) = scope;

            let window = web_sys::window().expect("no global window exists");
            let document = window.document().expect("no document on window exists");

            let a = document.create_element("p")?;
            let b = document.create_text_node(&format!("Hello, {}!", s_prop));

            Ok(Self {
                a,
                b,
                phantom: std::marker::PhantomData,
            })
        }

        fn mount(&mut self, parent: &Element, add_method: impl AddMethod) -> Result<(), JsValue> {
            add_method(&self.a)?;
            self.a.append_child(&self.b)?;
            Ok(())
        }

        fn proc(
            &mut self,
            _state: &mut Self::State,
            _scope: Self::Scope<'_>,
            _e: web_sys::Event,
            _target_path: Vec<u32>,
        ) -> Result<(), JsValue> {
            // No events in this example
            Ok(())
        }

        fn update(
            &mut self,
            _parent: &Element,
            state: &Self::State,
            scope: Self::Scope<'_>,
            flags: u64,
        ) -> Result<(), JsValue> {
            if flags & 1 << 3 != 0 {
                self.b
                    .set_text_content(Some(&format!("Hello, {}!", state.text)));
            }
            Ok(())
        }

        fn unmount(&self) {
            self.a.remove();
        }
    }

    /*
    Need: instance at mounted location
         - closure to acquire state & scope & pass in props
    Handle creators in definition
    */

    struct SnippetHandle2 {
        state: Rc<RefCell<ButtonState>>,
        scope: (), //items are rc'd
        contents: SnippetInstance2A,
    }

    impl SnippetHandle2 {
        fn create(state: Rc<RefCell<ButtonState>>, scope: ()) -> Self {
            todo!()
        }
    }

    trait Snippet2 {}

    struct SnippetInstance2A {
        a: Element,
        b: Text,
    }

    struct SnippetFactory2 {
        create_fn: Box<dyn Fn(Rc<RefCell<ButtonState>>, ()) -> dyn Snippet2>,
    }

    struct StateTest {
        factory: FactoryTest,
    }

    struct FactoryTest {
        state_rc: Rc<RefCell<StateTest>>,
    }
}
