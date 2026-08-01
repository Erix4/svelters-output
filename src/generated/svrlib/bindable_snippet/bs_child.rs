/*
<script>
  use $components::bindable_snippet::BsChild;

  struct $state {
    snip: $snippet(String) = $bindable(None), // becomes MutateTracker<Box<dyn SnippetFactoryTrait<(DynamicArg<String>,)>>>
  }
</script>

<#snippet my_snip(text: String)>
  <p>The message: {text}</p>
</snippet>
*/

use crate::*;
pub struct State {
    // Props:
    pub snip: MutateTracker<Box<dyn SnippetFactoryTrait<(DynamicArg<String>,)>>>,
}

impl ComponentState for State {
    type Props = ();

    fn init(&mut self) {
        // Initialize any state if needed
    }

    fn new(props: Self::Props) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(State {
            snip: MutateTracker::new(create_dummy_factory(), 0),
        }))
    }

    fn update_derived(&mut self) {
        // Update any derived state if needed (none in this example)
    }
}

pub struct RootFrag {
    a: SnippetScope<MySnip, EmptyFrag>,
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
        let a = SnippetScope::new(state, scope, current_path)?;

        Ok(RootFrag { a })
    }

    fn mount(&mut self, parent: &Element, add_method: &dyn AddMethod) -> Result<(), JsValue> {
        self.a.mount(parent, add_method)?;

        Ok(())
    }

    fn update(&mut self, parent: &Element, state: &Self::State, flags: u64) -> Result<(), JsValue> {
        self.a.update(parent, state, flags)?;

        Ok(())
    }

    fn proc(
        &mut self,
        state: &Rc<RefCell<Self::State>>,
        e: web_sys::Event,
        target_path: Vec<u32>,
    ) -> Result<(), JsValue>
    {
        self.a.proc(state, e, target_path)?;

        Ok(())
    }

    fn unmount(&self) {
        self.a.unmount();
    }
}

struct EmptyFrag {}

impl GenericFragment for EmptyFrag {
    type State = State;
    type Scope = ((), SnippetFactory<MySnip>);

    fn new(
        state: &Rc<RefCell<Self::State>>,
        scope: &Self::Scope,
        current_path: &Vec<u32>,
    ) -> Result<Self, JsValue>
    where
        Self: Sized,
    {
        let mut state = state.borrow_mut();
        let (_, snip_factory) = scope.clone();
        state.snip = MutateTracker::new(Box::new(snip_factory.clone()), 0);

        Ok(EmptyFrag {})
    }

    fn mount(&mut self, parent: &Element, add_method: &dyn AddMethod) -> Result<(), JsValue> {
        Ok(())
    }

    fn update(&mut self, parent: &Element, state: &Self::State, flags: u64) -> Result<(), JsValue> {
        Ok(())
    }

    fn proc(
        &mut self,
        state: &Rc<RefCell<Self::State>>,
        e: web_sys::Event,
        target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        Ok(())
    }

    fn unmount(&self) {}
}

struct MySnip {
    a: Element,
    b: DynamicText,
}

impl SnippetContentTrait for MySnip {
    type State = State;
    type Scope = ();
    type Args = (DynamicArg<String>,);

    fn update_args(&mut self, args: &mut Self::Args, flags: u64) {
        let (n,) = args;
        n.update(flags);
    }

    fn new(
        state_rc: &Rc<RefCell<Self::State>>,
        scope: &(Self::Scope, Self::Args),
        current_path: &Vec<u32>,
    ) -> Result<Self, JsValue>
    where
        Self: Sized,
    {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window");

        let a = document.create_element("p")?;

        let scope_clone = scope.clone();
        let scope_clone2 = scope.clone();
        let mut versions = (0,);
        let b = DynamicText::new(
            move || {
                let ((), (n,)) = &scope_clone;
                return format!("The message: {}", n.get());
            },
            move |_flags| {
                let ((), (n,)) = &scope_clone2;
                let (n_ver,) = &mut versions;
                if n.version() > *n_ver {
                    *n_ver += 1;
                    return true;
                }
                false
            },
        );

        Ok(MySnip { a, b })
    }

    fn mount(&mut self, parent: &Element, add_method: &dyn AddMethod) -> Result<(), JsValue> {
        add_method(&self.a)?;
        self.b.mount(&child_append_closure(&self.a))?;

        Ok(())
    }

    fn update(&mut self, parent: &Element, state: &Self::State, flags: u64) -> Result<(), JsValue> {
        self.b.update(flags)?;

        Ok(())
    }

    fn proc(
        &mut self,
        state_rc: &Rc<RefCell<Self::State>>,
        e: web_sys::Event,
        target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        Ok(())
    }

    fn unmount(&self) {
        self.b.unmount();
    }
}
