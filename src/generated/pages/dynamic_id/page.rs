use crate::*;

pub struct PageState {}

impl ComponentState for PageState {
    type Props = ();

    fn init(&mut self) {}

    fn new(props: Self::Props) -> Self {
        PageState {}
    }

    fn update_derived(&mut self) {}
}
