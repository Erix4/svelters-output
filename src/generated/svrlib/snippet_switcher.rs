/*
rsvelte file:
<script>
    struct $state {
        current_snippet: $bindable(test1),
    }

    impl $state {
        fn switch_snippet(&mut self) {
            if self.current_snippet == test1 {
                self.current_snippet = test2;
            } else {
                self.current_snippet = test1;
            }
        }
    }
</script>

<button onclick={switch_snippet}>
    Switch Snippet
</button>
<#snippet test1(n: String)>
    <p>{n}</p>
</snippet>
<#snippet test2(n: String)>
    <h1>{n}</h1>
</snippet>

{#each (0..5) as i}
    <#snippet test_each(n: String)> <!-- this is a separate snippet for each i -->
        <p>{n} from snippet {i}</p>
    </snippet>
    <button onclick={|| current_snippet = test_each;}>Snippet {i}</button>
{/each}

{@render current_snippet("Hello, world!") }
*/

use crate::*;

pub struct State {
    // Props:
    pub snippet: MutateTracker<Box<dyn SnippetFactoryTrait<(String,)>>>,
}

// User defined functions for State
impl State {
    fn switch_snippet(&mut self) {
        //
    }
}

impl ComponentState for State {
    type Props = ();

    fn init(&mut self) {
        // Initialize any state if needed
    }

    fn new(props: Self::Props) -> Rc<RefCell<Self>> {
        Rc::new_cyclic(|weak_state| {
            RefCell::new(State {
                snippet: MutateTracker::new(
                    Box::new(SnippetFactory::<Test1Snippet>::new(
                        weak_state.clone(),
                        (),
                        &vec![],
                    )),
                    0,
                ),
            })
        })
    }

    fn update_derived(&mut self) {
        // Update any derived state if needed (none in this example)
    }
}

pub struct RootFrag {
    a: Element,
    b: EachElement<EachFrag1>,
    c: Text,
}

impl GenericFragment for RootFrag {
    type State = State;
    type Scope = ();

    fn new(
        state: Rc<RefCell<Self::State>>,
        scope: (),
        current_path: &Vec<u32>,
    ) -> Result<Self, JsValue>
    where
        Self: Sized,
    {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window");

        //let snippet = SnippetElement::new(state, Test2Snippet::new(state, scope)?);

        let a = document.create_element("button")?;
        let b = EachElement::new(state.clone(), scope, &prepend_path(current_path, 1))?;
        let c = document.create_text_node(&format!("Switch snippet!"));

        add_listener(&a, "click", prepend_path(current_path, 0))?;

        Ok(Self { a, b, c })
    }

    fn mount(&mut self, parent: &Element, add_method: &dyn AddMethod) -> Result<(), JsValue> {
        add_method(&self.a)?;
        self.b.mount(parent, &add_method)?;
        self.a.append_child(&self.c)?;

        Ok(())
    }

    fn proc(
        &mut self,
        state_rc: Rc<RefCell<Self::State>>,
        scope: (),
        e: web_sys::Event,
        mut target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        let state_rc_clone = state_rc.clone();
        let mut state = state_rc.borrow_mut();
        let state_rc = state_rc_clone;

        let target = target_path.pop().unwrap();
        match e.type_().as_str() {
            "click" if target == 0 => {
                state.switch_snippet();
                *state.snippet = Box::new(SnippetFactory::<Test2Snippet>::new(
                    Rc::downgrade(&state_rc),
                    (),
                    &vec![],
                ));
            }
            _ => {}
        }

        Ok(())
    }

    fn update(
        &mut self,
        parent: &Element,
        state: Rc<RefCell<Self::State>>,
        scope: (),
        flags: u64,
    ) -> Result<(), JsValue> {
        if flags & 1 << 0 != 0 {
            self.b.update(parent, state, scope, flags)?;
        }

        Ok(())
    }

    fn unmount(&self) {
        self.a.remove();
        self.b.unmount();
        //self.c.unmount();
    }
}

struct Test1Snippet {
    a: Element,
    b: Text,
}

impl SnippetContentTrait for Test1Snippet {
    type State = State;
    type Scope = ();
    type Args = (String,);

    fn new(
        state: Rc<RefCell<Self::State>>,
        scope: (Self::Scope, Rc<Self::Args>),
        current_path: &Vec<u32>,
    ) -> Result<Self, JsValue>
    where
        Self: Sized,
    {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window");

        let ((), args) = scope;
        let (n,) = &*args;

        let a = document.create_element("div")?;
        let b = document.create_text_node(&format!("{}", n));

        Ok(Self { a, b })
    }

    fn mount(&mut self, parent: &Element, add_method: &dyn AddMethod) -> Result<(), JsValue> {
        add_method(&self.a)?;
        self.a.append_child(&self.b)?;

        Ok(())
    }

    fn proc(
        &mut self,
        state: Rc<RefCell<Self::State>>,
        scope: (Self::Scope, Rc<Self::Args>),
        e: web_sys::Event,
        target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        Ok(())
    }

    fn update(
        &mut self,
        parent: &Element,
        state: Rc<RefCell<Self::State>>,
        scope: (Self::Scope, Rc<Self::Args>),
        flags: u64,
    ) -> Result<(), JsValue> {
        Ok(())
    }

    fn unmount(&self) {
        self.a.remove();
    }
}

struct Test2Snippet {
    a: Element,
    b: Text,
}

/// TODO: can Args just be Scope?
impl SnippetContentTrait for Test2Snippet {
    type State = State;
    type Scope = ();
    type Args = (String,);

    fn new(
        state: Rc<RefCell<Self::State>>,
        scope: (Self::Scope, Rc<Self::Args>),
        current_path: &Vec<u32>,
    ) -> Result<Self, JsValue> {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window");

        let ((), args) = scope;
        let (n,) = &*args;

        let a = document.create_element("div")?;
        let b = document.create_text_node(&format!("{} Wow!", n));

        Ok(Test2Snippet { a, b })
    }

    fn mount(&mut self, parent: &Element, add_method: &dyn AddMethod) -> Result<(), JsValue> {
        add_method(&self.a)?;
        self.a.append_child(&self.b)?;

        Ok(())
    }

    fn proc(
        &mut self,
        state: Rc<RefCell<Self::State>>,
        scope: (Self::Scope, Rc<Self::Args>),
        e: web_sys::Event,
        target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        Ok(())
    }

    fn update(
        &mut self,
        parent: &Element,
        state: Rc<RefCell<Self::State>>,
        scope: (Self::Scope, Rc<Self::Args>),
        flags: u64,
    ) -> Result<(), JsValue> {
        Ok(())
    }

    fn unmount(&self) {
        self.a.remove();
    }
}

struct EachFrag1 {
    a: Box<dyn SnippetElementTrait<(String,)>>,
}

impl EachContentTrait for EachFrag1 {
    type Item = <std::ops::Range<i32> as IntoIterator>::Item;
    type Scope = ();
    type State = State;

    fn generate(
        state: Rc<RefCell<Self::State>>,
        scope: Self::Scope,
        flags: u64,
    ) -> Option<Vec<Self::Item>> {
        let _ = scope;
        if flags & 1u64 != 0 {
            // TODO: do only first time
            Some((0..5).into_iter().collect::<Vec<_>>())
        } else {
            None
        }
    }

    fn new(
        state_rc: Rc<RefCell<Self::State>>,
        scope: (Self::Scope, Rc<Self::Item>),
        current_path: &Vec<u32>,
    ) -> Result<Self, JsValue>
    where
        Self: Sized,
    {
        let state = state_rc.borrow();

        let a = state.snippet.init(
            Rc::new(("Hello world!".to_string(),)),
            &prepend_path(current_path, 1),
        )?;

        Ok(EachFrag1 { a })
    }

    fn mount(&mut self, parent: &Element, add_method: &dyn AddMethod) -> Result<(), JsValue> {
        self.a.mount(parent, add_method)?;

        Ok(())
    }

    fn proc(
        &mut self,
        state_rc: Rc<RefCell<Self::State>>,
        scope: (Self::Scope, &Self::Item),
        e: web_sys::Event,
        mut target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        let mut state = state_rc.borrow_mut();

        let target = target_path.pop().unwrap();
        match e.type_().as_str() {
            _ if target == 1 => {
                self.a.proc(e, target_path)?;
            }
            _ => {}
        }

        Ok(())
    }

    fn update(
        &mut self,
        parent: &Element,
        state_rc: Rc<RefCell<Self::State>>,
        scope: (Self::Scope, Rc<Self::Item>),
        flags: u64,
    ) -> Result<(), JsValue> {
        let state = state_rc.borrow();

        if flags & 1 << 0 != 0 {
            self.a = state.snippet.init_swap(parent, &self.a)?;
        }

        Ok(())
    }

    fn unmount(&self) {
        self.a.unmount();
    }
}
