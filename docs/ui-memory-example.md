# Alloy-ios: Ultimate UI & Memory Integration Guide

This document is the definitive reference for integrating Alloy's Native UI with the `specter-mem` memory subsystem. Each example is **self-contained** and provides the full code required for implementation.

---

## 1. Hex Patching (No Recoil)
**Concept:** Directly overwriting memory bytes at an RVA. Best for simple instruction changes.

```rust
// 1. Define the patching logic
fn toggle_recoil_patch(enabled: bool) {
    let target_rva = 0x100AABBCC;
    let patch_hex = if enabled { "1F 20 03 D5" } else { "08 01 40 B9" }; // NOP vs Original

    if let Ok(_) = specter::memory::manipulation::patch::apply(target_rva, patch_hex) {
        crate::utils::logger::info(&format!("Recoil Patch: {}", enabled));
    }
}

// 2. Register UI Toggle
ui::add_toggle(
    page_id,
    "No Recoil",
    "recoil_toggle",
    false,
    Some(|state| toggle_recoil_patch(state))
);
```

---

## 2. Assembly Patching (Instant Kill)
**Concept:** Using `jit-assembler` for type-safe ARM64 instruction patching.

```rust
use jit_assembler::aarch64::{builder::Aarch64InstructionBuilder, Register};

fn toggle_instakill(enabled: bool) {
    let target_rva = 0x100112233;

    if enabled {
        // Apply type-safe ASM: MOV X0, #9999; RET
        let _ = specter::memory::manipulation::patch::apply_asm(
            target_rva,
            |b: &mut Aarch64InstructionBuilder| {
                b.movz(Register(0), 9999, 0).ret()
            },
        );
    } else {
        // Restore original bytes (Example original: LDR X0, [X1, #0])
        let _ = specter::memory::manipulation::patch::apply(target_rva, "20 00 40 F9");
    }
}

// 3. Register UI Toggle
ui::add_toggle(page_id, "One Hit Kill", "instakill_key", false, Some(|s| toggle_instakill(s)));
```

---

## 3. Float Hook (Damage Multiplier)
**Concept:** Intercepting a function that returns a float and scaling it.

```rust
type GetDamageFn = extern "C" fn(usize) -> f32;
static mut ORIG_GET_DAMAGE: Option<GetDamageFn> = None;

#[no_mangle]
extern "C" fn hooked_get_damage(instance: usize) -> f32 {
    let original = unsafe { ORIG_GET_DAMAGE.unwrap()(instance) };
    let multiplier = ui::get_slider_value("dmg_multi_key");
    original * multiplier
}

// UI: Basic Slider
ui::add_slider(page_id, "Damage Multiplier", "dmg_multi_key", 1.0, 10.0, 1.0, None);

// Install:
unsafe {
    if let Ok(hook) = specter::memory::manipulation::hook::install(0x100223344, hooked_get_damage as usize) {
        ORIG_GET_DAMAGE = Some(std::mem::transmute(hook.trampoline()));
    }
}
```

---

## 4. Slider with Toggle + Float Hook (Walk Speed)
**Concept:** Master switch combined with a variable intensity value.

```rust
type GetSpeedFn = extern "C" fn(usize) -> f32;
static mut ORIG_GET_SPEED: Option<GetSpeedFn> = None;

#[no_mangle]
extern "C" fn hooked_get_speed(instance: usize) -> f32 {
    let original = unsafe { ORIG_GET_SPEED.unwrap()(instance) };

    if ui::get_toggle_value("speed_enabled_key") {
        return ui::get_slider_value("speed_val_key");
    }
    original
}

// UI: Slider with Toggle Options
ui::add_slider_with_options(
    page_id,
    "Walk Speed",
    "speed_val_key",
    1.0, 50.0, 5.0,
    ui::SliderOptions::new().with_toggle(ui::ToggleOptions::new("speed_enabled_key", false)),
    None
);

// Install:
unsafe {
    if let Ok(hook) = specter::memory::manipulation::hook::install(0x100556677, hooked_get_speed as usize) {
        ORIG_GET_SPEED = Some(std::mem::transmute(hook.trampoline()));
    }
}
```

---

## 5. Int Hook (Infinite Ammo)
**Concept:** Forcing a return value for integer-based game stats.

```rust
type GetAmmoFn = extern "C" fn(usize) -> i32;
static mut ORIG_GET_AMMO: Option<GetAmmoFn> = None;

#[no_mangle]
extern "C" fn hooked_get_ammo(instance: usize) -> i32 {
    if ui::get_toggle_value("inf_ammo_key") { return 999; }
    unsafe { ORIG_GET_AMMO.unwrap()(instance) }
}

// UI:
ui::add_toggle(page_id, "Infinite Ammo", "inf_ammo_key", false, None);

// Install:
unsafe {
    if let Ok(hook) = specter::memory::manipulation::hook::install(0x100889900, hooked_get_ammo as usize) {
        ORIG_GET_AMMO = Some(std::mem::transmute(hook.trampoline()));
    }
}
```

---

## 6. Bool Hook (Premium Status)
**Concept:** Forcing a flag to always return true.

```rust
type IsPremiumFn = extern "C" fn(usize) -> bool;
static mut ORIG_IS_PREMIUM: Option<IsPremiumFn> = None;

#[no_mangle]
extern "C" fn hooked_is_premium(instance: usize) -> bool {
    if ui::get_toggle_value("unlock_premium_key") { return true; }
    unsafe { ORIG_IS_PREMIUM.unwrap()(instance) }
}

// UI:
ui::add_toggle(page_id, "Unlock Premium", "unlock_premium_key", false, None);

// Install:
unsafe {
    if let Ok(hook) = specter::memory::manipulation::hook::install(0x100AABBCC, hooked_is_premium as usize) {
        ORIG_IS_PREMIUM = Some(std::mem::transmute(hook.trampoline()));
    }
}
```

---

## 7. Void Hook (God Mode)
**Concept:** Skipping a function's execution entirely.

```rust
type TakeDamageFn = extern "C" fn(usize, f32);
static mut ORIG_TAKE_DAMAGE: Option<TakeDamageFn> = None;

#[no_mangle]
extern "C" fn hooked_take_damage(instance: usize, amount: f32) {
    if ui::get_toggle_value("god_mode_key") { return; } // Skip damage
    unsafe { ORIG_TAKE_DAMAGE.unwrap()(instance, amount) };
}

// UI:
ui::add_toggle(page_id, "God Mode", "god_mode_key", false, None);

// Install:
unsafe {
    if let Ok(hook) = specter::memory::manipulation::hook::install(0x100DDEEFF, hooked_take_damage as usize) {
        ORIG_TAKE_DAMAGE = Some(std::mem::transmute(hook.trampoline()));
    }
}
```

---

## 8. Input with Toggle + String Hook (Custom Name)
**Concept:** Overriding a character string with user input.

```rust
use std::ffi::CString;

type GetNameFn = extern "C" fn(usize) -> *const i8;
static mut ORIG_GET_NAME: Option<GetNameFn> = None;

#[no_mangle]
extern "C" fn hooked_get_name(instance: usize) -> *const i8 {
    if ui::get_toggle_value("name_enabled_key") {
        let custom_name = ui::get_input_value("name_val_key");
        return CString::new(custom_name).unwrap().into_raw();
    }
    unsafe { ORIG_GET_NAME.unwrap()(instance) }
}

// UI:
ui::add_input_with_options(
    page_id,
    "Fake Nickname",
    "name_val_key",
    "Name...", "Alloy",
    ui::InputOptions::new().with_toggle(ui::ToggleOptions::new("name_enabled_key", false)),
    None
);

// Install:
unsafe {
    if let Ok(hook) = specter::memory::manipulation::hook::install(0x100114477, hooked_get_name as usize) {
        ORIG_GET_NAME = Some(std::mem::transmute(hook.trampoline()));
    }
}
```

---

## 9. Dropdown + Integer Hook (Team Selection)
**Concept:** Using a selection index to influence game logic.

```rust
type GetTeamFn = extern "C" fn(usize) -> i32;
static mut ORIG_GET_TEAM: Option<GetTeamFn> = None;

#[no_mangle]
extern "C" fn hooked_get_team(instance: usize) -> i32 {
    let selection = ui::get_dropdown_value("team_select_key");
    if selection != 0 {
        return selection; // 1 = Red, 2 = Blue, etc.
    }
    unsafe { ORIG_GET_TEAM.unwrap()(instance) }
}

// UI:
ui::add_dropdown(
    page_id,
    "Force Team",
    "team_select_key",
    vec!["Default".into(), "Red".into(), "Blue".into()],
    0,
    None
);

// Install:
unsafe {
    if let Ok(hook) = specter::memory::manipulation::hook::install(0x100558822, hooked_get_team as usize) {
        ORIG_GET_TEAM = Some(std::mem::transmute(hook.trampoline()));
    }
}
```

---

## 10. Hardware Breakpoint Hook (Stealth Hook)
**Concept:** Hooking without modifying code bytes (stealthier).

```rust
#[no_mangle]
extern "C" fn hooked_stealth_action() {
    crate::utils::logger::info("Stealth Hook Triggered!");
    ui::show_toast("Stealth Hook Hit!", ui::ToastStatus::Success);
}

// UI:
ui::add_action_button(page_id, "Install Stealth Hook", Some(|| {
    let target = 0x100AABBCC;
    unsafe {
        if let Ok(_) = specter::memory::platform::breakpoint::install(target, hooked_stealth_action as usize) {
            ui::alert("Success", "Hardware breakpoint installed!");
        }
    }
}));
```

---

## 11. Direct Function Calls (Add Gold / Trigger Action)
**Concept:** Manually executing a game's internal function when a button is pressed.

```rust
// 1. Define the function signature
// Example: void AddGold(void* player_instance, int amount)
type AddGoldFn = extern "C" fn(usize, i32);

// 2. Helper to resolve and call the function
fn trigger_add_gold(amount: i32) {
    let rva = 0x100556677; // Address from IDA
    
    // Resolve RVA to absolute memory address
    if let Ok(abs_addr) = specter::memory::info::address::resolve_rva(rva) {
        let add_gold: AddGoldFn = unsafe { std::mem::transmute(abs_addr) };
        
        // Note: 'player_instance' must be captured from a hook or static pointer
        let player_instance = 0x12345678; 
        
        unsafe { add_gold(player_instance, amount); }
        ui::show_toast("Gold Added!", ui::ToastStatus::Success);
    }
}

// 3. UI: Basic Button with Fixed Amount
ui::add_action_button(page_id, "Add 10k Gold", Some(|| {
    trigger_add_gold(10000);
}));

// 4. UI: Button using Slider Value
ui::add_slider(page_id, "Custom Gold", "custom_gold_val", 100.0, 50000.0, 1000.0, None);
ui::add_button(page_id, "Apply Custom Gold", Some(|| {
    let val = ui::get_slider_value("custom_gold_val") as i32;
    trigger_add_gold(val);
}));
```

## 12. Buttons & Feedback (Reset/Alerts)
**Concept:** Triggering UI feedback and one-time actions.

```rust
ui::add_action_button(page_id, "Purge Cache", Some(|| {
    ui::show_loading("Purging...");
    // perform_purge();
    ui::show_toast("Cache Cleared!", ui::ToastStatus::Success);
}));

ui::add_button(page_id, "Show Instructions", Some(|| {
    ui::alert("Instructions", "1. Open Game\n2. Toggle Features\n3. Enjoy!");
}));
```

---

## 13. Case Study: Il2Cpp Method (AddDiamond)
**Target:** `void AddDiamond(void* this, int32_t count, void* method)` at `0x32C23DC`.

### Option A: The Multiplier Hook (Recommended)
This intercepts every diamond the game gives you (from quests, rewards, etc.) and multiplies it by your Slider value.

```rust
// 1. Setup the Hook
type AddDiamondFn = extern "C" fn(usize, i32, usize);
static mut ORIG_ADD_DIAMOND: Option<AddDiamondFn> = None;

#[no_mangle]
extern "C" fn hooked_add_diamond(instance: usize, count: i32, method: usize) {
    // Read multiplier from UI (e.g., x10)
    let multiplier = ui::get_slider_value("diamond_multi_key") as i32;
    let boosted_count = count * multiplier;
    
    // Call original with the boosted amount
    unsafe { ORIG_ADD_DIAMOND.unwrap()(instance, boosted_count, method) };
}

// 2. UI: Slider
ui::add_slider(page_id, "Diamond Multiplier", "diamond_multi_key", 1.0, 100.0, 1.0, None);

// 3. Install:
// specter::memory::manipulation::hook::install(0x32C23DC, hooked_add_diamond as usize)...
```

### Option B: The "Give Diamonds" Button (Direct Call)
This allows you to press a button in the menu to instantly add diamonds.

```rust
static mut SAVED_MODEL_INSTANCE: usize = 0;

// 1. We must first "Capture" the 'this' pointer (instance) via a hook
#[no_mangle]
extern "C" fn hooked_capture_instance(instance: usize, count: i32, method: usize) {
    unsafe { SAVED_MODEL_INSTANCE = instance; } // Save the 'this' pointer for later
    unsafe { ORIG_ADD_DIAMOND.unwrap()(instance, count, method) };
}

// 2. Function to trigger the call from a button
fn trigger_give_diamonds(amount: i32) {
    let instance = unsafe { SAVED_MODEL_INSTANCE };
    if instance == 0 {
        ui::show_toast("Error: Play a game first to capture instance!", ui::ToastStatus::Error);
        return;
    }

    let rva = 0x32C23DC;
    if let Ok(abs_addr) = specter::memory::info::address::resolve_rva(rva) {
        let add_diamond: AddDiamondFn = unsafe { std::mem::transmute(abs_addr) };
        // Call the game's function directly (MethodInfo can usually be null)
        unsafe { add_diamond(instance, amount, std::ptr::null_mut()); }
        ui::show_toast("Diamonds Added!", ui::ToastStatus::Success);
    }
}

// 3. UI: Button
ui::add_action_button(page_id, "Give 10,000 Diamonds", Some(|| {
    trigger_give_diamonds(10000);
}));
```

## 🛠️ Organizing Your File

For a clean project, structure your `src/lib.rs` as follows:

1.  **Imports & Types**: `use` statements and `type FnName = ...`
2.  **Statics**: `static mut ORIG_...`
3.  **Hooks**: All `#[no_mangle] extern "C" fn hooked_...` functions.
4.  **UI Setup**: A `fn setup_menu()` containing all `ui::add_...` calls.
5.  **Installation**: A `fn install_hooks()` containing all `hook::install` calls.
6.  **Entry Point**: `#[ctor] fn init()` that dispatches to `setup_menu()` and calls `install_hooks()`.
