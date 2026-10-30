mod engine;
pub mod model;
pub mod path;
use model::Game;
use serde_json::{json, Value};
use std::cell::RefCell;
thread_local! {static GAME:RefCell<Option<Game>>=const{RefCell::new(None)};static OUTPUT:RefCell<Vec<u8>>=const{RefCell::new(Vec::new())};}
pub fn request(bytes: &[u8]) -> Vec<u8> {
    let result = (|| -> Result<Value, String> {
        let v: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        GAME.with(|cell| {
            let mut state = cell.borrow_mut();
            match v["op"].as_str().unwrap_or("") {
                "init" => {
                    let g = Game::init(&v)?;
                    let out = g.snapshot();
                    *state = Some(g);
                    Ok(out)
                }
                "tick" | "command" | "advance" => {
                    let g = state.as_mut().ok_or("Game not initialized")?;
                    g.ev.clear();
                    g.dirty.clear();
                    g.sync_controls(&v["controls"]);
                    if v["op"] == "command" {
                        g.command(&v)
                    } else {
                        let steps = if v["op"] == "advance" {
                            v["steps"].as_u64().unwrap_or(1).min(100000)
                        } else {
                            1
                        };
                        let dt = v["dt"].as_f64().unwrap_or(0.04);
                        if !dt.is_finite() || dt <= 0. {
                            return Err("Invalid timestep".into());
                        }
                        for _ in 0..steps {
                            g.step(dt);
                            if g.over.is_some() {
                                break;
                            }
                            if steps > 1 {
                                g.ev.clear();
                                g.dirty.clear();
                            }
                        }
                    }
                    Ok(g.snapshot())
                }
                "snapshot" => Ok(state.as_ref().ok_or("Game not initialized")?.snapshot()),
                _ => Err("Unknown operation".into()),
            }
        })
    })();
    serde_json::to_vec(&result.unwrap_or_else(|error| json!({"error":error}))).unwrap()
}
#[no_mangle]
pub extern "C" fn z_alloc(len: usize) -> *mut u8 {
    let mut v = vec![0u8; len].into_boxed_slice();
    let p = v.as_mut_ptr();
    std::mem::forget(v);
    p
}
/// # Safety: pointer and length must originate from z_alloc, and be freed only once.
#[no_mangle]
pub unsafe extern "C" fn z_free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() {
        drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len)));
    }
}
/// # Safety: ptr must reference len initialized bytes in wasm memory.
#[no_mangle]
pub unsafe extern "C" fn z_request(ptr: *const u8, len: usize) -> usize {
    let out = request(std::slice::from_raw_parts(ptr, len));
    OUTPUT.with(|o| {
        *o.borrow_mut() = out;
        o.borrow().len()
    })
}
#[no_mangle]
pub extern "C" fn z_output() -> *const u8 {
    OUTPUT.with(|o| o.borrow().as_ptr())
}
#[no_mangle]
pub extern "C" fn z_engine_version() -> u32 {
    3
}
