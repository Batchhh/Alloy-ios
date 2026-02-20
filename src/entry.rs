use crate::ui;
use dispatch::Queue;

fn init_ui() {
    let page_id = 1;

    ui::add_tab("Main Settings", page_id);
    ui::add_section_header(page_id, "General Configurations");

    ui::add_toggle(
        page_id,
        "Enable feature",
        "feature_enabled",
        false,
        Some(|state| crate::utils::logger::info(format!("Feature toggled: {}", state).as_str())),
    );

    ui::add_slider(
        page_id,
        "Intensity",
        "intensity_level",
        0.0,
        100.0,
        50.0,
        Some(|val| crate::utils::logger::info(format!("Intensity changed: {}", val).as_str())),
    );

    ui::add_input(
        page_id,
        "Username",
        "username",
        "Enter your username",
        "",
        Some(|text| crate::utils::logger::info(format!("Username changed: {}", text).as_str())),
    );

    ui::add_dropdown(
        page_id,
        "Theme",
        "theme_selection",
        vec!["Light".to_string(), "Dark".to_string(), "Auto".to_string()],
        1,
        Some(|idx| crate::utils::logger::info(format!("Theme selected index: {}", idx).as_str())),
    );

    ui::add_button_with_nav(
        page_id,
        "Advanced Settings",
        2,
        Some(|| crate::utils::logger::info("Navigating to advanced settings")),
    );

    ui::add_action_button(
        page_id,
        "Reset Defaults",
        Some(|| crate::utils::logger::info("Action button clicked")),
    );

    ui::add_button(
        page_id,
        "Apply Changes",
        Some(|| crate::utils::logger::info("Normal button clicked")),
    );

    ui::native::init_overlay();
}

fn memory_examples() {
    // Dummy hook replacement function
    #[no_mangle]
    extern "C" fn dummy_hook_replacement() {
        crate::utils::logger::info("Hook replacement hit!");
    }

    // Example: Inline hooking
    // Replace the function at `target` with the function at `replacement`.
    let target = 0x100000;
    let replacement = dummy_hook_replacement as usize;
    unsafe {
        if let Ok(hook) = crate::memory::hook::install(target, replacement) {
            crate::utils::logger::info(format!("Hook installed! Trampoline: {:#x}", hook.trampoline()).as_str());
        }
    }

    // Dummy breakpoint hook replacement function
    #[no_mangle]
    extern "C" fn dummy_breakpoint_replacement() {
        crate::utils::logger::info("Hardware breakpoint replacement hit!");
    }

    // Example: Hardware breakpoint hooking
    // Stealthier hook using ARM64 debug registers.
    let brk_target = 0x100004;
    let brk_replacement = dummy_breakpoint_replacement as usize;
    unsafe {
        if let Ok(_brk) = crate::memory::breakpoint::install(brk_target, brk_replacement) {
            crate::utils::logger::info("Hardware breakpoint hook installed!");
        }
    }

    // Example: Memory patching (Hex)
    // Write a hex patch at `target_rva` (using stealth mach_vm_remap if possible).
    let target_rva = 0x100008;
    let patch_hex = "C0 03 5F D6"; // ret
    if let Ok(_patch) = crate::memory::patch::apply(target_rva, patch_hex) {
        crate::utils::logger::info("Hex memory patch applied!");
    }

    // Example: Memory patching (ASM)
    // Write an assembly patch using jit_assembler
    let target_asm_rva = 0x10000C;
    if let Ok(_patch) = crate::memory::patch::apply_asm(target_asm_rva, |b| {
        b.movz(jit_assembler::aarch64::Register(0), 1, 0) // MOV X0, #1
         .ret()         // RET
    }) {
        crate::utils::logger::info("ASM memory patch applied!");
    }

    // Example: Code Cave Allocation
    // Find and allocate an unused memory region (cave) of 32 bytes.
    if let Ok(cave) = crate::memory::code_cave::allocate_cave(32) {
        crate::utils::logger::info(format!("Allocated 32-byte code cave at {:#x}", cave.address).as_str());
        // Clean up the allocated cave manually
        let _ = crate::memory::code_cave::free_cave(cave.address);
    }

    // Example: Shellcode Loading & Execution
    // Load custom ARM64 shellcode safely into an executable memory region and run it.
    let shellcode_instructions: &[u32] = &[
        0xD2800540, // MOV X0, #42 (Return value)
        0xD65F03C0, // RET
    ];
    
    match crate::memory::shellcode::ShellcodeBuilder::from_instructions(shellcode_instructions).load() {
        Ok(loaded_shellcode) => unsafe {
            // Execute the shellcode which returns 42
            let result = loaded_shellcode.execute();
            crate::utils::logger::info(format!("Shellcode executed successfully! Result = {}", result).as_str());
        },
        Err(err) => {
            crate::utils::logger::info(format!("Failed to load shellcode: {:?}", err).as_str());
        }
    }
}

pub fn init() {
    Queue::main().exec_async(|| {
        init_ui();
        memory_examples();
    });
}
