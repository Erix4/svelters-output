use wasm_bindgen::JsValue;
use web_sys::{Element, Text};

use crate::*;
use std::{rc::Rc, sync::atomic::Ordering::SeqCst, vec};

pub struct TopRouterState {
    // top level layout state would go here (none in this example)
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

    pub struct RootFrag {
        a: Element,
        b: Element,
        c: Element,
        d: Snippet<Snippet1<()>, Snippet1Peel>,
        //e: Box<dyn Snippet2>,
    }

    impl GenericFragment for RootFrag {
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
