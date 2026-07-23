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
        Rc::new_cyclic(|weak_state| {
            RefCell::new(State {})
        })
    }

    fn update_derived(&mut self) {
        // Update any derived state if needed (none in this example)
    }
}

pub struct RootFrag {
    a: Element,
    b: Element,
    c: Text,
    d: EachElement<EachFrag1>,
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

        let a = document.create_element("div")?;
        let b = document.create_element("h1")?;
        let c = document.create_text_node(&format!("Scoped Snippets"));
        let d = EachElement::new(state.clone(), scope, &prepend_path(current_path, 1))?;

        Ok(Self { a, b, c, d })
    }

    fn mount(&mut self, parent: &Element, add_method: &dyn AddMethod) -> Result<(), JsValue> {
        add_method(&self.a)?;
        self.a.append_child(&self.b)?;
        self.b.append_child(&self.c)?;
        //self.b.mount(parent, &add_method)?;

        Ok(())
    }

    fn proc(
        &mut self,
        state_rc: Rc<RefCell<Self::State>>,
        scope: (),
        e: web_sys::Event,
        mut target_path: Vec<u32>,
    ) -> Result<(), JsValue> {
        Ok(())
    }

    fn update(
        &mut self,
        parent: &Element,
        state: Rc<RefCell<Self::State>>,
        scope: (),
        flags: u64,
    ) -> Result<(), JsValue> {
        Ok(())
    }

    fn unmount(&self) {
        self.a.remove();
        self.b.remove();
        self.c.remove();
        self.d.unmount();
    }
}

struct EachFrag1 {
  
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

        Ok(EachFrag1 {  })
    }

    fn mount(&mut self, parent: &Element, add_method: &dyn AddMethod) -> Result<(), JsValue> {

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

        Ok(())
    }

    fn unmount(&self) {
    }
}
