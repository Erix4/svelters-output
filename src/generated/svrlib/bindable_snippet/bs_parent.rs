/*
<script>
  use $components::bindable_snippet::BsChild;

  struct $state {
    snip: $snippet(String)> = $state(), // default to Dummy Factory, becomes MutateTracker<Box<dyn SnippetFactoryTrait<(DynamicArg<String>,)>>>
    message: String = $state(),
  }
</script>

<input bind:value={message} />
<BsChild bind:{snip}>

{@render snip(message)}
*/
use crate::{generated::svrlib::bindable_snippet::bs_child, *};

pub struct State {
    // Props:
    pub snip: MutateTracker<Box<dyn SnippetFactoryTrait<(DynamicArg<String>,)>>>,
    pub message: MutateTracker<String>,
}

impl ComponentState for State {
    type Props = ();

    fn init(&mut self) {
        // Initialize any state if needed
    }

    fn new(props: Self::Props) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(State {
            snip: MutateTracker::new(create_dummy_factory(), 0),
            message: MutateTracker::new(String::new(), 1),
        }))
    }

    fn update_derived(&mut self) {
        // Update any derived state if needed (none in this example)
    }
}

pub struct RootFrag {
    a: HtmlInputElement,
    b: Component<bs_child::RootFrag>,
    c: Box<dyn SnippetElementTrait<(DynamicArg<String>,)>>,
}

impl GenericFragment for RootFrag {
    type State = State;
    type Scope = ();

    fn new(
        state: &Rc<RefCell<Self::State>>,
        scope: &Self::Scope,
        current_path: &Vec<u32>,
    ) -> Result<Self, JsValue>
    where
        Self: Sized,
    {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window");

        let a = document
            .create_element("input")?
            .dyn_into::<HtmlInputElement>()
            .unwrap();

        let b_state = bs_child::State::startup(());
        let b = Component::<bs_child::RootFrag>::new(&b_state, current_path)?;

        // Update bound snippet
        {
            let mut state = state.borrow_mut();
            *state.snip = b_state.borrow().snip.clone_box();
        }

        let state_clone = state.clone();
        let state_clone2 = state.clone();
        let factory = {
            let state = state.borrow();
            state.snip.clone_box()
        };
        let c = factory.init(
            (DynamicArg::new(
                move || {
                    let state = state_clone.borrow();
                    let str = format!("{}", *state.message);
                    str
                },
                move |flags| {
                    let state = state_clone2.borrow();
                    flags & 1 << state.message.id != 0
                },
            ),),
            current_path,
        )?;

        add_listener(&a, "input", prepend_path(current_path, 1))?;

        Ok(RootFrag { a, b, c })
    }

    fn mount(&mut self, parent: &Element, add_method: &dyn AddMethod) -> Result<(), JsValue> {
        add_method(&self.a)?;
        self.b.mount(parent, add_method)?;
        self.c.mount(parent, add_method)?;

        Ok(())
    }

    fn update(&mut self, parent: &Element, state: &Self::State, flags: u64) -> Result<(), JsValue> {
        self.c.update(parent, flags)?;

        // Propogate to children
        self.b.apply()?;

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
            "input" if target == 1 => {
                web_sys::console::log_1(&format!("got change event").into());
                let mut state = state.borrow_mut();
                *state.message = self.a.value();
            }
            _ => {}
        }

        Ok(())
    }

    fn unmount(&self) {
        self.a.remove();
        self.b.unmount();
        self.c.unmount();
    }
}
