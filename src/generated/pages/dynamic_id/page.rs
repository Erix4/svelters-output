use crate::*;

pub struct State {}

impl ComponentState for State {
    type Props = ();

    fn init(&mut self) {}

    fn new(props: Self::Props) -> Rc<RefCell<Self>> {
        Rc::new_cyclic(|weak_state| {
            RefCell::new(State {})
        })
    }

    fn update_derived(&mut self) {}
}
