/*
<div>
  <h1>Scoped Snippets</h1>
  {#snippet non_scoped(i)}
    <p>Non-scoped {i}</p>
  {/snippet}

  {#each (0..5) as i}
    {#snippet first_scope()}
      {@render non_scoped(i)}
      <p>First scope</p>

      {#snippet second_scope()}
        <p>Second scope {i}</p>
      {/snippet}

      {@render second_scope()}
    {/snippet}

    {@render first_scope()}
  {/each}
</div>


*/

use crate::*;

pub struct State {
    //
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
        Rc::new_cyclic(|weak_state| RefCell::new(State {}))
    }

    fn update_derived(&mut self) {
        // Update any derived state if needed (none in this example)
    }
}

pub struct RootFrag {
    a: SnippetScope<NonScopedSnippet, GenericFrag1>,
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

        Ok(Self { a })
    }

    fn mount(&mut self, parent: &Element, add_method: &dyn AddMethod) -> Result<(), JsValue> {
        self.a.mount(parent, add_method)?;

        Ok(())
    }

    fn proc(
        &mut self,
        state_rc: &Rc<RefCell<Self::State>>,
        e: web_sys::Event,
        mut target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        self.a.proc(state_rc, e, target_path)?;

        Ok(())
    }

    fn update(
        &mut self,
        parent: &Element,
        state: &Self::State,
        flags: u64,
    ) -> Result<(), JsValue> {
        self.a.update(parent, state, flags)?;
        Ok(())
    }

    fn unmount(&self) {
        self.a.unmount();
    }
}

struct NonScopedSnippet {
    a: Element,
    b: DynamicText,
}

impl SnippetContentTrait for NonScopedSnippet {
    type State = State;
    type Scope = ();
    type Args = (DynamicArg<i32>,);

    fn new(
        state_rc: &Rc<RefCell<Self::State>>,
        scope: &(Self::Scope, Rc<Self::Args>),
        current_path: &Vec<u32>,
    ) -> Result<Self, JsValue>
    where
        Self: Sized,
    {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window");

        let a = document.create_element("p")?;
        let b = DynamicText::new(
            Rc::new(|| {
                return format!("Scoped Snippets");
            }),
            1 << 1,
        );

        Ok(Self { a, b })
    }

    fn mount(&mut self, parent: &Element, add_method: &dyn AddMethod) -> Result<(), JsValue> {
        add_method(&self.a)?;
        self.b.mount(&child_append_closure(&self.a))?;

        Ok(())
    }

    fn proc(
        &mut self,
        state_rc: &Rc<RefCell<Self::State>>,
        e: web_sys::Event,
        target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        // Nothing to proc here
        Ok(())
    }

    fn update(
        &mut self,
        parent: &Element,
        state_rc: &Self::State,
        flags: u64,
    ) -> Result<(), JsValue> {
        self.b.update(flags)?;
        Ok(())
    }

    fn unmount(&self) {
        self.a.remove();
    }
}

struct GenericFrag1 {
    a: Element,
    b: Element,
    c: DynamicText,
    d: EachElement<EachFrag1>,
}

impl GenericFragment for GenericFrag1 {
    type State = State;
    type Scope = ((), Rc<SnippetFactory<NonScopedSnippet>>);

    fn new(
        state_rc: &Rc<RefCell<Self::State>>,
        scope: &Self::Scope,
        current_path: &Vec<u32>,
    ) -> Result<Self, JsValue>
    where
        Self: Sized,
    {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window");

        let a = document.create_element("div")?;
        let b = document.create_element("h1")?;
        let c = DynamicText::new(
            Rc::new(|| {
                return format!("Scoped Snippets");
            }),
            1 << 1,
        );
        let d = EachElement::new(state_rc, scope, &prepend_path(current_path, 1))?;

        Ok(Self { a, b, c, d })
    }

    fn mount(&mut self, parent: &Element, add_method: &dyn AddMethod) -> Result<(), JsValue> {
        add_method(&self.a)?;
        self.a.append_child(&self.b)?;
        self.c.mount(&child_append_closure(&self.b))?;
        self.d.mount(&self.a, &child_append_closure(&self.a))?;

        Ok(())
    }

    fn proc(
        &mut self,
        state_rc: &Rc<RefCell<Self::State>>,
        e: web_sys::Event,
        mut target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        Ok(())
    }

    fn update(
        &mut self,
        parent: &Element,
        state: &Self::State,
        flags: u64,
    ) -> Result<(), JsValue> {
        Ok(())
    }

    fn unmount(&self) {
        self.a.remove();
        self.b.remove();
        self.c.unmount();
        self.d.unmount();
    }
}

struct EachFrag1 {
    a: SnippetScope<FirstScopeSnippet, FirstScopeRenderFrag>,
}

impl EachContentTrait for EachFrag1 {
    type Item = <std::ops::Range<i32> as IntoIterator>::Item;
    type Scope = ((), Rc<SnippetFactory<NonScopedSnippet>>);
    type State = State;

    fn generate(state: &Self::State, scope: &Self::Scope, flags: u64) -> Option<Vec<Self::Item>> {
        let _ = scope;
        if flags & 1u64 != 0 {
            // TODO: do only first time
            Some((0..5).into_iter().collect::<Vec<_>>())
        } else {
            None
        }
    }

    fn new(
        state_rc: &Rc<RefCell<Self::State>>,
        scope: &(Self::Scope, Rc<Self::Item>),
        current_path: &Vec<u32>,
    ) -> Result<Self, JsValue>
    where
        Self: Sized,
    {
        let state = state_rc.borrow();

        let a = SnippetScope::new(state_rc, scope, current_path)?;

        Ok(EachFrag1 { a })
    }

    fn mount(&mut self, parent: &Element, add_method: &dyn AddMethod) -> Result<(), JsValue> {
        Ok(())
    }

    fn proc(
        &mut self,
        state_rc: &Rc<RefCell<Self::State>>,
        e: web_sys::Event,
        mut target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        let mut state = state_rc.borrow_mut();

        Ok(())
    }

    fn update(
        &mut self,
        parent: &Element,
        state_rc: &Self::State,
        flags: u64,
    ) -> Result<(), JsValue> {
        Ok(())
    }

    fn unmount(&self) {}
}

struct FirstScopeRenderFrag {
    a: Box<dyn SnippetElementTrait<()>>,
}

impl GenericFragment for FirstScopeRenderFrag {
    type State = State;
    type Scope = (
        (((), Rc<SnippetFactory<NonScopedSnippet>>), Rc<i32>),
        Rc<SnippetFactory<FirstScopeSnippet>>,
    );

    fn new(
        state: &Rc<RefCell<Self::State>>,
        scope: &Self::Scope,
        current_path: &Vec<u32>,
    ) -> Result<Self, JsValue>
    where
        Self: Sized,
    {
        let (((_, _), _), factory) = scope;

        let a = factory.init(Rc::new(()), current_path)?;

        Ok(Self { a })
    }

    fn mount(&mut self, parent: &Element, add_method: &dyn AddMethod) -> Result<(), JsValue> {
        self.a.mount(parent, add_method)?;
        Ok(())
    }

    fn proc(
        &mut self,
        state: &Rc<RefCell<Self::State>>,
        e: web_sys::Event,
        target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        self.a.proc(e, target_path)?;
        Ok(())
    }

    fn update(
        &mut self,
        parent: &Element,
        state: &Self::State,
        flags: u64,
    ) -> Result<(), JsValue> {
        self.a.update(parent, (), flags)?;
        Ok(())
    }

    fn unmount(&self) {
        self.a.unmount();
    }
}

struct FirstScopeSnippet {
    a: SnippetScope<SecondScopeSnippet, GenericFrag2>,
}

impl SnippetContentTrait for FirstScopeSnippet {
    type State = State;
    type Scope = (((), Rc<SnippetFactory<NonScopedSnippet>>), Rc<i32>);
    type Args = ();

    fn new(
        state_rc: &Rc<RefCell<Self::State>>,
        scope: &(Self::Scope, Self::Args),
        current_path: &Vec<u32>,
    ) -> Result<Self, JsValue>
    where
        Self: Sized
    {
        let (((_, factory), each_item), arg) = scope.clone();

        let a = factory.init((DynamicArg::new(|| *each_item, 0),), current_path)?;

        Ok(Self {
            a
        })
    }
}

struct SecondScopeSnippet {
    a: Element,
    b: DynamicText,
}

impl SnippetContentTrait for SecondScopeSnippet {
    type State = State;
    type Scope = (
        (((), Rc<SnippetFactory<NonScopedSnippet>>), Rc<i32>),
        Rc<SnippetFactory<FirstScopeSnippet>>,
    );
    type Args = ();
}

struct GenericFrag2 {
    a: Box<dyn SnippetElementTrait<(i32,)>>,
    b: Element,
    c: DynamicText,
    d: Box<dyn SnippetElementTrait<()>>,
}

impl GenericFragment for GenericFrag2 {
    type State = State;
    type Scope = (
        (
            (((), Rc<SnippetFactory<NonScopedSnippet>>), Rc<i32>),
            Rc<SnippetFactory<FirstScopeSnippet>>,
        ),
        Rc<SnippetFactory<SecondScopeSnippet>>,
    );
}
