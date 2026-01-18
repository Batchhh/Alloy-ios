mod config;
mod memory;
mod ui;
mod utils;
use memory::{breakpoint::Breakpoint, hook::Hook, patch::Patch};
use parking_lot::Mutex;
use std::ffi::c_void;
use utils::logger;

static PATCH: Mutex<Option<Patch>> = Mutex::new(None);
static HOOK: Mutex<Option<Hook>> = Mutex::new(None);
static BRK_HOOK: Mutex<Option<Breakpoint>> = Mutex::new(None);

fn init_ui() {
    let mtm = objc2_foundation::MainThreadMarker::new().unwrap();
    // Page 0: Testing label
    ui::add_label(0, "Testing", 12.0, true, Some("#888888"));

    // Page 0: Patch test toggle
    ui::add_toggle(
        0,
        "Patch test",
        "patch",
        false,
        Some(|on| {
            let mut lock = PATCH.lock();
            if on {
                // memory::patch::apply(0x292d94c, "COO35FD6");
                if let Ok(p) = memory::patch::apply_asm(0x292d94c, |b| b.ret()) {
                    *lock = Some(p);
                    logger::info("Patch test ON");
                }
            } else if let Some(p) = lock.take() {
                p.revert();
                logger::info("Patch test OFF");
            }
        }),
    );

    // Page 0: Hook test toggle
    ui::add_toggle(
        0,
        "Hook test",
        "hook",
        false,
        Some(|on| {
            let mut lock = HOOK.lock();
            if on {
                fn hook_fn(this: *mut c_void, _dt: f32) {
                    unsafe {
                        let _ = memory::rw::write(this as usize + 0xa8, 999.0);
                    }
                    if let Some(h) = HOOK.lock().as_ref() {
                        let orig: fn(*mut c_void, f32) = unsafe { h.trampoline_as() };
                        orig(this, _dt);
                    }
                }
                if let Ok(h) = unsafe { memory::hook::install(0x292d94c, hook_fn as usize) } {
                    *lock = Some(h);
                    logger::info("Hook test ON");
                }
            } else if let Some(h) = lock.take() {
                h.remove();
                logger::info("Hook test OFF");
            }
        }),
    );

    // Page 0: Scan test action button
    ui::add_action_button(
        0,
        "Scan test",
        Some(|| {
            // if let Ok(results) = memory::scan::scan_image(config::TARGET_IMAGE_NAME, "1F 20 03 D5") {
            //     logger::info(&format!("Found {} NOP instructions", results.len()));
            // }
            if let Ok(results) =
                memory::scan::scan_image_asm(config::TARGET_IMAGE_NAME, |b| b.ret())
            {
                logger::info(&format!("Found {} ret instructions", results.len()));
            }
        }),
    );

    /* Example Usage for other UI features:
    ui::add_label(0, "SETTINGS", 12.0, true, Some("#888888"));
    ui::add_button_with_nav(0, "More Settings", 1, None::<fn()>);
    ui::add_slider(page_id, name, key, min, max, default, callback);
    ui::add_input(page_id, name, key, placeholder, default, callback);
    */

    // Page 0: Breakpoint hook test toggle

    ui::add_toggle(
        0,
        "Brk Hook test",
        "brk_hook",
        false,
        Some(|on| {
            let mut lock = BRK_HOOK.lock();
            if on {
                // Breakpoint hook - no code modification, uses hardware debug registers
                // Note: replacement function receives control directly (no trampoline)
                extern "C" fn brk_replacement(this: *mut c_void, dt: f32) {
                    logger::info(&format!("Breakpoint hook triggered! this: {:?}", this));

                    // Call original function
                    // 1. Suspend breakpoints for this thread (to avoid infinite recursion)
                    // 2. Call original function at target address
                    // 3. Resume breakpoints for this thread
                    unsafe {
                        let _ = memory::rw::write(this as usize + 0xa8, 999.0);
                    }

                    if let Some(h) = BRK_HOOK.lock().as_ref() {
                        let target = h.target();
                        unsafe {
                            let _ = memory::breakpoint::suspend_self();
                            let orig: extern "C" fn(*mut c_void, f32) = std::mem::transmute(target);
                            orig(this, dt);
                            let _ = memory::breakpoint::resume_self();
                        }
                    }
                }
                match unsafe { memory::breakpoint::install(0x292d94c, brk_replacement as usize) } {
                    Ok(h) => {
                        *lock = Some(h);
                        logger::info("Brk Hook ON");
                    }
                    Err(e) => {
                        logger::error(&format!("Brk Hook install failed: {:?}", e));
                    }
                }
            } else if let Some(h) = lock.take() {
                let _ = h.remove();
                logger::info("Brk Hook OFF");
            }
        }),
    );

    ui::native::init_overlay(mtm);
}

#[ctor::ctor]
fn init() {
    dispatch::Queue::main().exec_async(|| {
        init_ui();
        if let Ok(a) = memory::symbol::resolve_symbol("il2cpp_init") {
            logger::info(&format!("il2cpp_init: {:#x}", a));
        }
    });
}
