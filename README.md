# Rust iOS Tweak

A high-performance iOS modding framework built entirely from scratch in Rust—no Substrate, no Dobby, no external libraries.

## Status

**Currently Jailbreak Only** - This project requires a jailbroken iOS device. Support for non-jailbroken devices is planned for future development.

## Features

- **Custom ARM64 Inline Hooking**: Hand-crafted trampoline generation with PC-relative instruction relocation (supports conditional branches, BL, ADR/ADRP, and LDR literals).
- **W^X Compliant Memory Patching**: Safe memory writes with automatic thread suspension/resumption and instruction cache invalidation.
- **Type-Safe Memory Utilities**: Read/write operations with RVA-to-absolute address conversion and pointer chain traversal.
- **Apple Unified Logging**: Direct `oslog` integration for debugging via Console.app.
- **Zero External Binary Dependencies**: Built on direct `mach2` syscalls and idiomatic Rust with exhaustive error handling via `thiserror`.

## Configuration

You can customize the tweak behavior in `src/config.rs`:

```rust
pub const TARGET_IMAGE_NAME: &str = "UnityFramework"; // Binary to hook
pub const DEBUG: bool = true;                         // Toggle detailed logging
```

## Building & Deploying

1. **Prerequisites**:
   ```bash
   rustup target add aarch64-apple-ios
   brew install sshpass
   https://theos.dev/docs/installation-ios
   ```

2. **Deploy to Device**:
   ```bash
   # Edit Makefile to set your DEVICE_IP
   make deploy
   ```

## Viewing Logs

Logs are sent to the Apple Unified Logging System. You can view them using **Console.app** on macOS:
- filter for: `RGG` or `subsystem:com.rust_tweak`.

## Roadmap

Planned features and improvements for future releases:

- [ ] **In-Game UI Menu**: SwiftUI or Metal-based overlay for runtime mod control
- [ ] **Memory Scanning**: Pattern scanning and signature-based function finding
- [ ] **Breakpoint Hooks**: Hardware breakpoint support for non-jailbroken devices
- [ ] **Symbol Resolution**: Automatic symbol lookup and caching
- [ ] **Configuration File**: TOML/JSON-based mod configurations
- [ ] **Hot Reloading**: Dynamic mod loading without reinjection

## Contributing

Contributions are welcome! If you find any issues or have suggestions, please [open an issue](https://github.com/Batchhh/Rust-ios-tweak/issues).

For collaboration, feel free to submit [pull requests](https://github.com/Batchhh/Rust-ios-tweak/pulls).

Follow Rust conventions (`cargo fmt`, `cargo clippy`) and document your changes.

### Commit Guidelines

Use clear, descriptive commit messages:
Format: `type: brief description`
Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`

## Legal

Educational purposes only. Modifying games may violate Terms of Service. Use at your own risk.

## License

[MIT License](https://github.com/Batchhh/Rust-ios-tweak/blob/main/LICENSE) - See LICENSE file for details.