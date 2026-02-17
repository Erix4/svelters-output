use serde::Serialize;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::Ordering::SeqCst;
use std::u64;
use wasm_bindgen::prelude::*;

mod state2;

use crate::state2::{DIRTY_FLAGS, STATE};

#[derive(Serialize)]
pub enum PatchOp {
    MountPage {
        tag: String,
    },
    MountFragment {
        tag: String,
        parent_id: u32,
    },
    MountEachFragment,
    MountEachItem {
        tag: String,
        parent_id: u32,
    },
    MountTag {
        tag: String,
        parent_id: u32,
    },
    MountComment {
        parent_id: u32,
    },
    AddToMap {
        // does not insert into DOM
        tag: String,
    },
    InsertBefore {
        // target is the node in map to insert
        reference_id: u32,
        parent_id: u32,
    },
    MountEventListener {
        event_type: String,
        target_id_path: Vec<u32>,
    },
    SetContent {
        value: String,
    },
    SetAttribute {
        name: String,
        value: String,
    },
    UnmountTag {
        parent_id: u32,
    },
    UnmountEachItem {
        each_id: u32,
        parent_id: u32,
    },
}

#[derive(Serialize)]
pub enum PatchTreeNode {
    Leaf(u32, PatchOp),
    Node(u32, Vec<PatchTreeNode>),
}

pub struct MutateTracker<T> {
    value: T,
    id: u32,
    component_id: usize, // in Wasm, usize is also 32 bits
}

impl<T> MutateTracker<T> {
    pub fn new(value: T, id: u32, component_id: usize) -> Self {
        Self {
            value,
            id,
            component_id,
        }
    }
}

impl<T> Deref for MutateTracker<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for MutateTracker<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        web_sys::console::log_1(
            &format!(
                "DerefMut called for id {}, comp {}",
                self.id, self.component_id
            )
            .into(),
        );
        DIRTY_FLAGS[self.component_id].fetch_or(1 << self.id, std::sync::atomic::Ordering::SeqCst);
        &mut self.value
    }
}

pub fn prepend_path(base: &Vec<u32>, addition: u32) -> Vec<u32> {
    let mut out = vec![addition];
    out.extend_from_slice(base);
    out
}

#[wasm_bindgen]
pub fn mount() -> JsValue {
    for flag in DIRTY_FLAGS.iter() {
        flag.store(u64::MAX, SeqCst);
    }
    serde_wasm_bindgen::to_value(&STATE.with(|state| {
        let state = &mut *state.borrow_mut();
        let mut patch_tree = state.mount(vec![]);
        patch_tree.append(&mut state.apply());
        patch_tree
    }))
    .unwrap()
}

#[wasm_bindgen]
pub fn handle_event(e: web_sys::Event, target: Vec<u32>) -> JsValue {
    for flag in DIRTY_FLAGS.iter() {
        flag.store(0, SeqCst);
    }
    serde_wasm_bindgen::to_value(&STATE.with(|state| {
        let state = &mut *state.borrow_mut();
        state.affect(e, target, ());
        state.apply()
    }))
    .unwrap()
}
