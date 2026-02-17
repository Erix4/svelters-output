//! This file will be auto-generated

use crate::{prepend_path, MutateTracker, PatchOp, PatchTreeNode};
use std::{
    cell::RefCell,
    sync::atomic::{AtomicU64, Ordering::SeqCst},
    vec,
};
use wasm_bindgen::JsCast;

pub static DIRTY_FLAGS: [AtomicU64; 2] = [AtomicU64::new(0), AtomicU64::new(0)]; // Only 64 state variables supported per component

struct MyStruct {
    a: i32,
    b: String,
}

thread_local! {
    /// Global state for the page component
    ///
    /// Each component has the following sections recursively:
    /// - Prop state: state variables passed from parent component
    /// - Reactive state: state variables that trigger re-renders when changed
    /// - Derived state: state variables that are computed from reactive state
    /// - Non-reactive state: state variables that do not trigger re-renders
    /// - Child component local state: recurse
    /// - Fragment state: state variables for managing fragments (e.g., #if, #each)
    ///
    /// All values except derived are user defined. Derived state is auto-generated.
    pub static STATE: RefCell<Page> = RefCell::new(Page {
        // Reactive state
        counter: MutateTracker::new(0, 0, 0),
        my_struct: MutateTracker::new(MyStruct {
            a: 10,
            b: "hello".to_string(),
        }, 1, 0),
        // Derived state
        counter_plus_one: 1,
        // Non-reactive state
        other_var: 42,
        // Child component local state
        button_1: ButtonState {
            // Prop state
            text: "Click me".to_string(),
            // Reactive state
            button_counter: MutateTracker::new(0, 1, 1),
        },
        // Fragment state
        fragment1: false,
        fragment2: 0,
    });
}

pub struct Page {
    counter: MutateTracker<i32>,
    my_struct: MutateTracker<MyStruct>,
    counter_plus_one: i32,
    other_var: i32,
    button_1: ButtonState,
    fragment1: bool,
    fragment2: usize,
}

impl Page {
    pub fn mount(&mut self, parent_path: Vec<u32>) -> Vec<PatchTreeNode> {
        // run user init code
        *self.counter += 1;
        self.other_var += 1;

        // do first derive
        self.counter_plus_one = *self.counter + 1;

        // generate patches for mounting
        vec![
            PatchTreeNode::Leaf(
                0,
                PatchOp::MountPage {
                    tag: "div".to_string(),
                },
            ),
            PatchTreeNode::Leaf(
                1,
                PatchOp::MountTag {
                    tag: "button".to_string(),
                    parent_id: 0,
                },
            ),
            PatchTreeNode::Leaf(
                2,
                PatchOp::MountTag {
                    tag: "button".to_string(),
                    parent_id: 0,
                },
            ),
            PatchTreeNode::Leaf(
                3,
                PatchOp::MountFragment {
                    tag: "div".to_string(),
                    parent_id: 0,
                },
            ),
            PatchTreeNode::Node(3, self.button_1.mount(prepend_path(&parent_path, 3))),
            PatchTreeNode::Leaf(
                4,
                PatchOp::MountTag {
                    tag: "p".to_string(),
                    parent_id: 0,
                },
            ),
            PatchTreeNode::Leaf(
                5,
                PatchOp::MountTag {
                    tag: "p".to_string(),
                    parent_id: 0,
                },
            ),
            PatchTreeNode::Leaf(6, PatchOp::MountComment { parent_id: 0 }),
            PatchTreeNode::Leaf(
                7,
                PatchOp::AddToMap {
                    tag: "p".to_string(),
                },
            ),
            PatchTreeNode::Leaf(
                7,
                PatchOp::SetContent {
                    value: "Counter is greater than 5!".to_string(),
                },
            ),
            PatchTreeNode::Leaf(
                8,
                PatchOp::AddToMap {
                    tag: "p".to_string(),
                },
            ),
            PatchTreeNode::Leaf(
                8,
                PatchOp::SetContent {
                    value: "Counter is 5 or less.".to_string(),
                },
            ),
            PatchTreeNode::Leaf(
                8,
                PatchOp::InsertBefore {
                    reference_id: 6,
                    parent_id: 0,
                },
            ),
            PatchTreeNode::Leaf(9, PatchOp::MountEachFragment),
            PatchTreeNode::Leaf(
                10,
                PatchOp::MountTag {
                    tag: "div".to_string(),
                    parent_id: 0,
                },
            ),
            PatchTreeNode::Leaf(
                11,
                PatchOp::MountTag {
                    tag: "p".to_string(),
                    parent_id: 10,
                },
            ),
            PatchTreeNode::Leaf(
                1,
                PatchOp::MountEventListener {
                    event_type: "click".to_string(),
                    target_id_path: prepend_path(&parent_path, 1),
                },
            ),
            PatchTreeNode::Leaf(
                2,
                PatchOp::MountEventListener {
                    event_type: "click".to_string(),
                    target_id_path: prepend_path(&parent_path, 2),
                },
            ),
        ]
    }

    pub fn affect(&mut self, e: web_sys::Event, mut target_path: Vec<u32>, _: ()) {
        let target = target_path.pop().unwrap();
        match e.type_().as_str() {
            "click" if target == 1 => {
                let e = e.dyn_into::<web_sys::MouseEvent>().unwrap();
                (|_, counter| {
                    increment(counter);
                })(e, &mut self.counter);
            }
            "click" if target == 2 => {
                (|struct_state, counter| {
                    update_struct(struct_state, counter, true);
                })(&mut self.my_struct, &mut self.counter);
            }
            _ if target == 3 => {
                let counter = &mut self.counter;
                let button_props = ButtonProps {
                    func: Box::new(move |new_val| {
                        **counter = new_val;
                    }),
                };
                self.button_1.affect(e, target_path, button_props);
            }
            _ => {}
        }

        // update derived
        if DIRTY_FLAGS[0].load(SeqCst) & 1 << 0 != 0 {
            self.counter_plus_one = *self.counter + 1;
            DIRTY_FLAGS[0].fetch_or(1 << 3, SeqCst);
        }

        // propagate to children
        if DIRTY_FLAGS[0].load(SeqCst) & 1 << 1 != 0 {
            // Update props for button
            if DIRTY_FLAGS[0].load(SeqCst) & 1 << 1 != 0 {
                self.button_1.text = self.my_struct.b.clone(); // my_struct.b is the given expression
                DIRTY_FLAGS[1].fetch_or(1 << 0, SeqCst); // manually mark prop as dirty
            }

            self.button_1.apply();
        }
    }

    pub fn apply(&mut self) -> Vec<PatchTreeNode> {
        // generate patches based on dirty flags
        let mut patches_out = vec![];
        if DIRTY_FLAGS[0].load(SeqCst) & 1 << 0 != 0 {
            patches_out.push(PatchTreeNode::Leaf(
                1,
                PatchOp::SetContent {
                    value: format!("Counter: {}", *self.counter),
                },
            ));
            patches_out.push(PatchTreeNode::Leaf(
                4,
                PatchOp::SetContent {
                    value: format!("Count: {}", *self.counter),
                },
            ));

            // #if block
            patches_out.extend(if self.fragment1 {
                vec![PatchTreeNode::Leaf(7, PatchOp::UnmountTag { parent_id: 0 })]
            } else {
                vec![PatchTreeNode::Leaf(8, PatchOp::UnmountTag { parent_id: 0 })]
            });
            self.fragment1 = *self.counter > 5;
            patches_out.extend(if self.fragment1 {
                vec![PatchTreeNode::Leaf(
                    7,
                    PatchOp::InsertBefore {
                        reference_id: 6,
                        parent_id: 0,
                    },
                )]
            } else {
                vec![PatchTreeNode::Leaf(
                    8,
                    PatchOp::InsertBefore {
                        reference_id: 6,
                        parent_id: 0,
                    },
                )]
            });

            // #each block
            let new_fragment2_list = (0..*self.counter).collect::<Vec<i32>>();
            if new_fragment2_list.len() != self.fragment2 {
                // unmount old
                patches_out.extend((0..self.fragment2 as u32).map(|_| {
                    PatchTreeNode::Leaf(
                        0,
                        PatchOp::UnmountEachItem {
                            each_id: 9,
                            parent_id: 0,
                        },
                    )
                }));

                // mount new
                let mut set_content_patches = vec![];
                for (i, num) in new_fragment2_list.iter().enumerate() {
                    patches_out.push(PatchTreeNode::Leaf(
                        9,
                        PatchOp::MountEachItem {
                            tag: "p".to_string(),
                            parent_id: 0,
                        },
                    ));
                    set_content_patches.push(PatchTreeNode::Leaf(
                        i as u32,
                        PatchOp::SetContent {
                            value: format!("Number: {}", num),
                        },
                    ));
                }
                self.fragment2 = new_fragment2_list.len();
                patches_out.push(PatchTreeNode::Node(9, set_content_patches));
            }
        }
        if DIRTY_FLAGS[0].load(SeqCst) & 1 << 1 != 0 {
            patches_out.push(PatchTreeNode::Leaf(
                2,
                PatchOp::SetContent {
                    value: format!("Struct A: {}", self.my_struct.a),
                },
            ));
            patches_out.push(PatchTreeNode::Leaf(
                5,
                PatchOp::SetContent {
                    value: format!("Struct B: {}", self.my_struct.b),
                },
            ));
        }
        if DIRTY_FLAGS[0].load(SeqCst) & 1 << 3 != 0 {
            patches_out.push(PatchTreeNode::Leaf(
                11,
                PatchOp::SetContent {
                    value: format!("Counter plus one: {}", self.counter_plus_one),
                },
            ));
        }

        patches_out.push(PatchTreeNode::Node(3, self.button_1.apply()));

        patches_out
    }
}

struct ButtonState {
    text: String,
    button_counter: MutateTracker<i32>,
}

struct ButtonProps<'a> {
    func: Box<dyn FnMut(i32) + 'a>,
}

impl ButtonState {
    fn mount(&mut self, parent_path: Vec<u32>) -> Vec<PatchTreeNode> {
        vec![
            PatchTreeNode::Leaf(
                1,
                PatchOp::MountTag {
                    tag: "button".to_string(),
                    parent_id: 0,
                },
            ),
            PatchTreeNode::Leaf(
                2,
                PatchOp::MountTag {
                    tag: "button".to_string(),
                    parent_id: 0,
                },
            ),
            PatchTreeNode::Leaf(
                2,
                PatchOp::SetContent {
                    value: "Set parent".to_string(),
                },
            ),
            PatchTreeNode::Leaf(
                1,
                PatchOp::MountEventListener {
                    event_type: "click".to_string(),
                    target_id_path: prepend_path(&parent_path, 1),
                },
            ),
            PatchTreeNode::Leaf(
                2,
                PatchOp::MountEventListener {
                    event_type: "click".to_string(),
                    target_id_path: prepend_path(&parent_path, 2),
                },
            ),
        ]
    }

    fn affect(
        &mut self,
        e: web_sys::Event,
        mut target_path: Vec<u32>,
        mut func_props: ButtonProps,
    ) {
        let target = target_path.pop().unwrap();
        match e.type_().as_str() {
            "click" if target == 1 => {
                (|button_counter: &mut i32| {
                    *button_counter += 1;
                })(&mut self.button_counter);
            }
            "click" if target == 2 => {
                (|button_counter: &mut i32| {
                    (func_props.func)(*button_counter);
                })(&mut self.button_counter);
            }
            _ => {}
        }
    }

    fn apply(&mut self) -> Vec<PatchTreeNode> {
        let mut patches_out = vec![];
        if DIRTY_FLAGS[1].load(SeqCst) & (1 << 0 | 1 << 1) != 0 {
            patches_out.push(PatchTreeNode::Leaf(
                1,
                PatchOp::SetContent {
                    value: format!("{}: {}", self.text, *self.button_counter),
                },
            ));
        }
        patches_out
    }

    fn unmount(&mut self) -> Vec<PatchTreeNode> {
        vec![]
    }
}

fn increment(counter: &mut i32) {
    *counter += 1;
    add(*counter, 5);
}

fn add<T: std::ops::Add<Output = T>>(a: T, b: T) -> T {
    a + b
}

fn update_struct(
    struct_state: &mut MutateTracker<MyStruct>,
    counter: &mut MutateTracker<i32>,
    inc: bool,
) {
    if inc {
        increment(counter);
    }
    struct_state.a += **counter;
    struct_state.b = format!("Count is {}", **counter);
}
