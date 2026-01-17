# Rust ios tweak

A long-term project to build an iOS game mod menu framework entirely in Rust.

## Why Rust?

- **Memory Safety**
- **Performance**
- **Developer Experience**
- **Maintainability**

## Status

**Currently Jailbreak Only** - This project currently requires a jailbroken iOS device to function. Support for non-jailbroken devices is planned for future development.

## Features

- **Zero External Dependencies**: All hooking and patching logic is implemented from scratch in pure Rust—no Substrate, Fishhook, or Dobby used.
- **ARM64 Inline Hooking**: Custom trampoline generation with PC-relative instruction support.
- **Memory Patching**: Safe W^X compliant memory writing.
- **Thread Safety**: Automatic thread suspension/resumption during patches.
- **System Logging**: Integration with Apple Unified Logging System.
- **Configurable**: Easily toggle debug logging and target image in `src/config.rs`.

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

## Contributing

Contributions are welcome!
Follow Rust conventions (`cargo fmt`, `cargo clippy`) and document your changes.

### Commit Guidelines

Use clear, descriptive commit messages:
Format: `type: brief description`
Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`

## Legal

Educational purposes only. Modifying games may violate Terms of Service. Use at your own risk.

## License

MIT License - See LICENSE file for details.