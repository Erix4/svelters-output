use crate::{prepend_path, Init, MutateTracker, PatchOp, PatchTreeNode, DIRTY_FLAGS};
use std::sync::atomic::Ordering::SeqCst;

struct MyStruct {
    a: i32,
    b: String,
}

/*struct $state {
    counter = 0,
    my_struct = MyStruct {
        a: 10,
        b: "hello".to_string(),
    },
    counter_plus_one = $derived(counter + 1),
}*/

/// Global state for the page component
///
/// Each component has the following sections recursively:
/// - Prop state: state variables passed from parent component
/// - Reactive state: state variables that trigger re-renders when changed
/// - Derived state: state variables that are computed from reactive state
/// - Child component local state: recurse
/// - Fragment state: state variables for managing fragments (e.g., #if, #each)
///
/// All values except derived are user defined. Derived state is auto-generated.
pub struct Page {
    // Prop state: none

    // Reactive state:
    counter: MutateTracker<i32>,        // id: 0
    my_struct: MutateTracker<MyStruct>, // id: 1

    // Derived state:
    counter_plus_one: i32,

    // Child component local state:
    button_1: Button,

    // Fragment state:
    fragment1: bool,  // whether #if block is currently rendered
    fragment2: usize, // number of items currently rendered in #each block
}

impl Init for Page {
    fn init(&mut self) {
        //let mut other_var = 42;
        //other_var += 1;

        *self.counter += 1;
    }
}

impl Page {
    pub fn new() -> Self {
        let mut new_page = Self {
            counter: MutateTracker::new(0, 0),
            my_struct: MutateTracker::new(
                MyStruct {
                    a: 10,
                    b: "hello".to_string(),
                },
                1,
            ),
            counter_plus_one: 1, // needs to be initialized to something, will be updated in init
            button_1: Button {
                text: "".to_string(),
                func_call: MutateTracker::new(0, 1),
                button_counter: MutateTracker::new(0, 0),
            },
            fragment1: false,
            fragment2: 0,
        };

        new_page.init();

        new_page
    }

    pub fn mount(&mut self, parent_path: Vec<u32>) -> Vec<PatchTreeNode> {
        // generate patches for mounting
        let mut patches_out = vec![
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
        ];

        DIRTY_FLAGS.store(u64::MAX, SeqCst); // mark all as dirty for initial render
        patches_out.extend(self.apply(0)); // prevent propagation down (done manually)
        patches_out
    }

    /// Process an event and return patches to apply to the DOM
    ///
    /// The target_path describes where the event took place.
    /// This function will check if the current component is the target,
    /// and if so run the corresponding user code for that event.
    /// Otherwise, it will propagate the event to the correct child component to process.
    ///
    /// If handled by a child, the function checks for changes in bindable props and updates
    /// the state of the current component accordingly, and marking them to be excluded from
    /// propagation back down to the child to avoid feedback loops.
    ///
    /// Finally, it runs the apply function, to derived, generate patches,
    /// and propagate any changes to children as needed.
    pub fn proc(
        &mut self,
        e: web_sys::Event,
        mut target_path: Vec<u32>,
        _: (),
    ) -> Vec<PatchTreeNode> {
        let mut patches_out = vec![];
        let mut bind_exlude_flags = 0;

        // Event handling
        let target = target_path.pop().unwrap();
        match e.type_().as_str() {
            "click" if target == 1 => {
                self.increment();
            }
            "click" if target == 2 => {
                (|state: &mut Page| state.update_struct(true))(self);
            }
            _ if target == 3 => {
                patches_out.push(PatchTreeNode::Node(3, self.button_1.proc(e, target_path)));
                let child_bindable_flags = DIRTY_FLAGS.load(SeqCst);
                DIRTY_FLAGS.store(0, SeqCst); // reset for parent processing

                // Update bindable props based on child changes
                if child_bindable_flags & 1 << 1 != 0 {
                    // Run user defined closure with bound function call
                    (|state: &mut Page, new_val| {
                        *state.counter = new_val;
                    })(self, *self.button_1.func_call);

                    bind_exlude_flags |= 1 << 0; // don't propagate this change back to button
                }
            }
            _ => {}
        }

        patches_out.extend(self.apply(bind_exlude_flags));

        patches_out
    }

    /// Generate patches based on changes to reactive state and derived state
    ///
    /// The flag_exclude parameter is a bitmask of any bindable props that should
    /// not be included in the propagation to children to avoid feedback loops.
    pub fn apply(&mut self, flag_exclude: u64) -> Vec<PatchTreeNode> {
        // update derived
        if DIRTY_FLAGS.load(SeqCst) & 1 << 0 != 0 {
            self.counter_plus_one = *self.counter + 1;
            DIRTY_FLAGS.fetch_or(1 << 3, SeqCst);
        }

        // generate patches based on dirty flags
        let mut patches_out = vec![];

        // counter changed
        if DIRTY_FLAGS.load(SeqCst) & 1 << 0 != 0 {
            patches_out.push(PatchTreeNode::Leaf(
                1,
                PatchOp::SetContent {
                    value: format!("Count: {}", *self.counter),
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

        // my_struct changed
        if DIRTY_FLAGS.load(SeqCst) & 1 << 1 != 0 {
            patches_out.push(PatchTreeNode::Leaf(
                2,
                PatchOp::SetContent {
                    value: format!("Struct A: {} (Click to update)", self.my_struct.a),
                },
            ));
            patches_out.push(PatchTreeNode::Leaf(
                5,
                PatchOp::SetContent {
                    value: format!("Struct B: {}", self.my_struct.b),
                },
            ));
        }

        // counter_plus_one changed (derived)
        if DIRTY_FLAGS.load(SeqCst) & 1 << 3 != 0 {
            patches_out.push(PatchTreeNode::Leaf(
                11,
                PatchOp::SetContent {
                    value: format!("Counter plus one: {}", self.counter_plus_one),
                },
            ));
        }

        let flag_snapshot = DIRTY_FLAGS.load(SeqCst);
        let mut prop_flags = flag_snapshot;

        // clear any bindable flags for children that we just updated to avoid feedback loops
        prop_flags &= !flag_exclude;

        // Propagate to children

        // Button 1
        if prop_flags & 1 << 1 != 0 {
            DIRTY_FLAGS.store(0, SeqCst); // reset before child processing
                                          // Update props for button
            if prop_flags & 1 << 1 != 0 {
                self.button_1.text = self.my_struct.b.clone(); // my_struct.b is the given expression
                DIRTY_FLAGS.fetch_or(1 << 0, SeqCst); // manually mark prop as dirty
            }

            patches_out.push(PatchTreeNode::Node(3, self.button_1.apply(0)));
        }

        // Restore snapshot
        DIRTY_FLAGS.store(flag_snapshot, SeqCst);

        patches_out
    }

    // User defined functions for Page
    fn increment(&mut self) {
        *self.counter += 1;
        add(*self.counter, 5);
    }

    fn update_struct(&mut self, inc: bool) {
        if inc {
            self.increment();
        }
        self.my_struct.a += *self.counter; // NOTE: no need to deref when a field in a state variable is accessed (it already does it implicitly)
        self.my_struct.b = format!("Count is {}", *self.counter);
    }
}

fn add<T: std::ops::Add<Output = T>>(a: T, b: T) -> T {
    a + b
}

struct Button {
    // Props:
    text: String,
    // Bindable props (any func prop affecting state is automatically bindable):
    func_call: MutateTracker<i32>,

    // Reactive state:
    button_counter: MutateTracker<i32>,
}

impl Button {
    fn proc(&mut self, e: web_sys::Event, mut target_path: Vec<u32>) -> Vec<PatchTreeNode> {
        let mut patches_out = vec![];
        let bind_exlude_flags = 0;

        // Event handling
        let target = target_path.pop().unwrap();
        match e.type_().as_str() {
            "click" if target == 1 => {
                *self.button_counter += 1;
            }
            "click" if target == 2 => {
                *self.func_call = *self.button_counter;
            }
            _ => {}
        }

        patches_out.extend(self.apply(bind_exlude_flags));

        patches_out
    }

    fn apply(&mut self, flag_exclude: u64) -> Vec<PatchTreeNode> {
        // Update derived (none in this example)

        // Generate patches for any state changes in this component
        let mut patches_out = vec![];
        if DIRTY_FLAGS.load(SeqCst) & (1 << 0 | 1 << 1) != 0 {
            patches_out.push(PatchTreeNode::Leaf(
                1,
                PatchOp::SetContent {
                    value: format!("{}: {}", self.text, *self.button_counter),
                },
            ));
        }

        let flag_snapshot = DIRTY_FLAGS.load(SeqCst);

        // clear any bindable flags for children that we just updated to avoid feedback loops
        DIRTY_FLAGS.fetch_and(!flag_exclude, SeqCst);

        // Propagate to children (none in this example)

        // Restore snapshot
        DIRTY_FLAGS.store(flag_snapshot, SeqCst);

        // Return patches to parent for processing
        patches_out
    }

    fn mount(&mut self, parent_path: Vec<u32>) -> Vec<PatchTreeNode> {
        let mut patches_out = vec![
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
        ];

        DIRTY_FLAGS.store(u64::MAX, SeqCst); // mark all as dirty for initial render
        patches_out.extend(self.apply(u64::MAX)); // prevent propagation down (done manually)
        patches_out
    }
}
