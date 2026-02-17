use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU64, Ordering::SeqCst},
};

use serde::Serialize;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::state::Page;

mod state;

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

pub static DIRTY_FLAGS: AtomicU64 = AtomicU64::new(0);

pub struct MutateTracker<T> {
    value: T,
    id: u32,
}

impl<T> MutateTracker<T> {
    pub fn new(value: T, id: u32) -> Self {
        Self { value, id }
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
        web_sys::console::log_1(&format!("DerefMut called for id {}", self.id).into());
        DIRTY_FLAGS.fetch_or(1 << self.id, std::sync::atomic::Ordering::SeqCst);
        &mut self.value
    }
}

pub trait Init {
    fn init(&mut self);
}

pub fn prepend_path(base: &Vec<u32>, addition: u32) -> Vec<u32> {
    let mut out = vec![addition];
    out.extend_from_slice(base);
    out
}

thread_local! {
    pub static PAGE: RefCell<Page> = RefCell::new(Page::new());
}

#[wasm_bindgen]
pub fn mount() -> JsValue {
    DIRTY_FLAGS.store(u64::MAX, SeqCst);
    serde_wasm_bindgen::to_value(&PAGE.with(|page| {
        let page = &mut *page.borrow_mut();
        page.mount(vec![])
    }))
    .unwrap()
}

#[wasm_bindgen]
pub fn handle_event(e: web_sys::Event, target: Vec<u32>) -> JsValue {
    DIRTY_FLAGS.store(0, SeqCst);
    serde_wasm_bindgen::to_value(&PAGE.with(|page| {
        let page = &mut *page.borrow_mut();
        page.proc(e, target, ())
    }))
    .unwrap()
}
