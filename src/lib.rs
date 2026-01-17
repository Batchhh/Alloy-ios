//! Main entry point

mod config;
mod memory;
mod ui;
mod utils;

use std::{ffi::c_void};
use utils::logger;

use dispatch::Queue;

fn _example_patch() -> bool {
    // Example 1: Hex-based patch with revert
    // if let Ok(patch) = memory::patch::apply(0x292d94c, "C0035FD6") {
    //     logger::info("Applied hex patch!");
    //     patch.revert();
    //     logger::info("Hex patch reverted");
    //     return true;
    // }

    // Example 2: ASM-based patch with revert
    if let Ok(_patch) = memory::patch::apply_asm(0x292d94c, |b| b.ret()) {
        logger::info("Applied ASM patch!");
        // patch.revert();
        // logger::info("ASM patch reverted");
        return true;
    }

    false
}

static HOOK: std::sync::OnceLock<memory::hook::Hook> = std::sync::OnceLock::new();

fn _example_hook() -> bool {
    type UpdateFn = fn(*mut c_void, f32);

    fn update_hook(this: *mut c_void, delta_time: f32) {
        if let Some(hook) = HOOK.get() {
            let original: UpdateFn = unsafe { hook.trampoline_as() };
            
            // Read and log values at offsets 0xa8 and 0xb0
            unsafe {
                let coins_addr = this as usize + 0xa8;
                
                if let Ok(coins_value) = memory::rw::read::<f64>(coins_addr) {
                    logger::info(&format!("this+0xa8 (double): {}", coins_value));
                    let _ = memory::rw::write::<f64>(coins_addr, 99999.0);
                    logger::info("Modified this+0xa8 to 99999.0");
                }
            }
            
            original(this, delta_time);
        }
    }

    unsafe {
        if let Ok(hook) = memory::hook::install(0x292d94c, update_hook as usize) {
            logger::info("Hook installed!");
            let _ = HOOK.set(hook);
            // To remove later: HOOK.get().unwrap().remove();
            return true;
        }
    }

    false
}

fn init_ui() {
    use objc2_foundation::MainThreadMarker;
    
    logger::info("Initializing native iOS UI overlay..");
    
    // === HOME PAGE (Page 0) - Navigation ===
    // Add a title label
    ui::add_label(0, "CATEGORIES", 12.0, true, Some("#888888"));
    
    // Add navigation buttons to reach different pages
    ui::add_button_with_nav(0, "Cheats", 1, None::<fn()>);
    ui::add_button_with_nav(0, "Settings", 2, None::<fn()>);
    
    // === REGISTER MENU ITEMS ===
    // Page 1: Cheats
    ui::add_toggle(1, "God Mode", "god_mode", false, Some(|enabled| {
        logger::info(&format!("God Mode: {}", enabled));
    }));
    
    ui::add_toggle(1, "Infinite Ammo", "infinite_ammo", false, Some(|enabled| {
        logger::info(&format!("Infinite Ammo: {}", enabled));
    }));
    
    ui::add_toggle(1, "No Clip", "no_clip", false, Some(|enabled| {
        logger::info(&format!("No Clip: {}", enabled));
    }));
    
    ui::add_slider(1, "Speed Multiplier", "speed", 0.1, 5.0, 1.0, Some(|value| {
        logger::info(&format!("Speed: {}x", value));
    }));
    
    // Page 2: Settings
    ui::add_input(2, "Username", "username", "Enter username", "", Some(|text| {
        logger::info(&format!("Username changed: {}", text));
    }));
    
    // Action button example (button with arrow and callback)
    ui::add_action_button(2, "Clear Data", Some(|| {
        logger::info("Clear Data button tapped!");
        // Add your action here
    }));
    
    // === EXAMPLE: Create a custom page ===
    // let custom_page = ui::add_page("My Features");
    // ui::add_toggle(custom_page, "Custom Feature", "custom_feat", false, Some(|enabled| {
    //     logger::info(&format!("Custom Feature: {}", enabled));
    // }));
    // ui::add_slider(custom_page, "Custom Value", "custom_val", 0.0, 100.0, 50.0, None::<fn(f32)>);
    
    // Get main thread marker
    if let Some(mtm) = MainThreadMarker::new() {
        ui::native::init_overlay(mtm);
        logger::info("Mod menu overlay created");
    } else {
        logger::error("Failed to get MainThreadMarker - not on main thread!");
    }
}

#[ctor::ctor]
fn init() {
    logger::info("rust_igmm initializing...");
    
    Queue::main().exec_async(|| {
        // Initialize UI menu
        init_ui();
        
        // Run example hook
        if _example_hook() {
            logger::info("Example hook executed");
        }
        

        // if _example_patch() {
        //     logger::info("Example patch executed");
        // }
    });
}
