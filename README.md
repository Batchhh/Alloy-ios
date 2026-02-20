# Alloy ios tweak

High-performance iOS modding framework built in Rust.

## Features
- **Hooking**: ARM64 Inline & Hardware Breakpoints.
- **Memory**: Safe patching, scanning, and symbol resolution.
- **UI**: Native iOS Menu (objc2) & content-rich Mod Menu.
- **Stealth**: Stealthy hooking and patching. (ShellCode Injection + Code Caves)

## Usage

1. **Configure**: Edit `src/config.rs` to set target binary and options.
2. **Deploy**:
   ```bash
   make deploy
   ```

## Documentation

Read the source code to understand it better, since there will be no documentation. It is easy to understand since it is well commented.

## Viewing Logs

Logs are sent to the Apple Unified Logging System. You can view them using **Console.app** on macOS:
- filter for: `Alloy` or `subsystem:com.batch.alloy`.

## Contributing

Contributions are welcome! If you find any issues or have suggestions, please [open an issue](https://github.com/Batchhh/Rust-ios-tweak/issues).

For collaboration, feel free to submit [pull requests](https://github.com/Batchhh/Rust-ios-tweak/pulls).

Follow Rust conventions (`cargo fmt`, `cargo clippy`) and document your changes.

### Commit Guidelines

- Use clear, descriptive messages
- Format: `type: brief description`
- Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`

## Legal

Educational purposes only. Modifying games may violate Terms of Service. Use at your own risk.

## License

[MIT License](https://github.com/Batchhh/Rust-ios-tweak/blob/main/LICENSE) - See LICENSE file for details.

## Credits

- [Batch](https://github.com/Batchhh) - Creator
- [Titanox](https://github.com/Ragekill3377/Titanox) - Inspiration for breakpoint hooks