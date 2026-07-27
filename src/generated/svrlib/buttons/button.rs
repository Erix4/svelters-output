/*
rsvelte file:
<script>
    struct $state {
        text: String = $prop(),
        func: fn(i32) = $prop(),

        button_counter = $state(0),
    }
</script>

<div>
    <button onclick={|$state| {state.button_counter += 1} }>
        {text}: {button_counter}
    </button>
    <button onclick={|$state| {func(state.button_counter)} }>
        Set parent
    </button>
</div>

<style>
    button {
        padding: 0.5em 1em;
        font-size: 1em;
        cursor: pointer;
    }
</style>
*/

use crate::*;

pub struct State {
    // Props:
    pub text: String,
    // Bindable props (any func prop affecting state is automatically bindable):
    pub func_call: i32,

    // Reactive state:
    pub button_counter: MutateTracker<i32>,
}

impl ComponentState for State {
    type Props = (String,); // text prop

    fn init(&mut self) {
        // Initialize any state if needed
    }

    fn new(props: Self::Props) -> Rc<RefCell<Self>> {
        Rc::new_cyclic(|weak_state| {
            RefCell::new(State {
                text: props.0,
                func_call: 0,
                button_counter: MutateTracker::new(0, 0),
            })
        })
    }

    fn update_derived(&mut self) {
        // Update any derived state if needed (none in this example)
    }
}

pub struct RootFrag {
    a: Element,
    b: Element,
    c: Element,
}

impl GenericFragment for RootFrag {
    type State = State;
    type Scope = ();

    fn new(state: &Rc<RefCell<Self::State>>, scope: &Self::Scope, current_path: &Vec<u32>) -> Result<Self, JsValue>
    where
        Self: Sized,
    {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window");
        let state = state.borrow();

        let a = document.create_element("div")?;
        let b = document.create_element("button")?;
        let c = document.create_element("button")?;
        c.set_inner_html("Set parent");
        b.set_inner_html(&format!("{}: {}", state.text, *state.button_counter));

        add_listener(&b, "click", prepend_path(current_path, 1))?; // target path is in reverse order!!
        add_listener(&c, "click", prepend_path(current_path, 2))?;

        Ok(Self { a, b, c })
    }

    fn mount(&mut self, parent: &Element, add_method: &dyn AddMethod) -> Result<(), JsValue> {
        add_method(&self.a)?;
        self.a.append_child(&self.b)?;
        self.a.append_child(&self.c)?;

        Ok(())
    }

    fn proc(
        &mut self,
        state: &Rc<RefCell<Self::State>>,
        e: web_sys::Event,
        mut target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        let target = target_path.pop().unwrap();
        match e.type_().as_str() {
            "click" if target == 1 => {
                let mut state = state.borrow_mut();
                *state.button_counter += 1;
            }
            "click" if target == 2 => {
                let mut state = state.borrow_mut();
                state.func_call = *state.button_counter;
                DIRTY_FLAGS.fetch_or(1 << 1, SeqCst); // mark func_call prop as dirty to propagate to parent
            }
            _ => {}
        }

        Ok(())
    }

    fn update(
        &mut self,
        parent: &Element,
        state: &Self::State,
        flags: u64,
    ) -> Result<(), JsValue> {
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
