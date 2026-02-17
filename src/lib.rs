use std::{
    cell::RefCell, ops::{Deref, DerefMut}, sync::atomic::{AtomicU64, Ordering::SeqCst}, vec
};

use wasm_bindgen::{JsCast, JsValue, prelude::{Closure, wasm_bindgen}};
use web_sys::Comment;

use crate::state::Page;

mod state;

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

pub struct IfElement<T, U> {
    pub comment: Comment,
    pub if_content: T,
    pub else_content: U,
}

pub fn prepend_path(base: &Vec<u32>, addition: u32) -> Vec<u32> {
    let mut out = vec![addition];
    out.extend_from_slice(base);
    out
}

pub fn add_listener(el: &web_sys::Element, event: &str, target_path: Vec<u32>) -> Result<(), JsValue> {
    let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
        handle_event(e, target_path.clone());
    }) as Box<dyn FnMut(_)>);
    el.add_event_listener_with_callback(event, closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

/// Each route maps a path pattern to a component constructor
pub struct Route {
    pattern: &'static str,
    // Params extracted from URL (e.g., /users/:id)
    param_names: &'static [&'static str],
}

/// Simple pattern matching: "/users/:id" matches "/users/42"
pub fn match_pattern(
    pattern: &str,
    path: &str,
    param_names: &[&str],
) -> Option<Vec<(String, String)>> {
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let path_parts: Vec<&str> = path.split('/').collect();

    if pattern_parts.len() != path_parts.len() {
        return None;
    }

    let mut params = Vec::new();
    let mut param_idx = 0;

    for (p, actual) in pattern_parts.iter().zip(path_parts.iter()) {
        if p.starts_with(':') {
            params.push((param_names[param_idx].to_string(), actual.to_string()));
            param_idx += 1;
        } else if p != actual {
            return None;
        }
    }

    Some(params)
}

thread_local! {
    pub static PAGE: RefCell<Option<Page>> = RefCell::new(None);
}

#[wasm_bindgen]
pub fn mount() -> Result<(), JsValue> {
    web_sys::console::log_1(&"Mounting application".into());
    PAGE.with(|page| {
        *page.borrow_mut() = Some(Page::new(vec![])?);
        web_sys::console::log_1(&"Page component mounted".into());
        Ok(())
    })
}

pub fn handle_event(e: web_sys::Event, target: Vec<u32>) {
    DIRTY_FLAGS.store(0, SeqCst);
    let _ = PAGE.with(|page| {
        let page = &mut *page.borrow_mut();
        let page = page.as_mut().expect("Page component should be initialized");
        page.proc(e, target, ()).or_else(|e| {
            web_sys::console::error_1(&format!("Error processing event: {:?}", e).into());
            Err(e)
        }).ok();
    });
}
