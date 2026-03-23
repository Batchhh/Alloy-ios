# Rust library
RUST_PROFILE ?= release

# For Linux builds (xcrun is not available on Linux)
# export SDKROOT := $(THEOS)/sdks/iPhoneOS15.5.sdk/

# Enable with dev-release profile logging
ifeq ($(RUST_PROFILE),dev-release)
    export RUSTFLAGS += --cfg dev_release
endif

# Auto-detect sccache for faster rebuilds
SCCACHE := $(shell command -v sccache 2> /dev/null)
ifdef SCCACHE
    export RUSTC_WRAPPER := sccache
endif

before-all::
	@printf "\033[1;36m==>\033[0m Building Rust library...\n"
	cargo build --profile $(RUST_PROFILE) --target aarch64-apple-ios

ARCHS = arm64
FINALPACKAGE = 1
THEOS_DYLIB := .theos/obj/arm64/alloy.dylib 

# TARGET ?= iphone:clang:latest:15.5  # Uncomment to specify a specific SDK version

# Uncomment the following for roothide builds
# THEOS_PACKAGE_SCHEME = roothide

include $(THEOS)/makefiles/common.mk

TWEAK_NAME = alloy
RUST_LIB := target/aarch64-apple-ios/$(RUST_PROFILE)/liballoy.a

${TWEAK_NAME}_CFLAGS = -fobjc-arc
${TWEAK_NAME}_LDFLAGS = -force_load $(RUST_LIB)
${TWEAK_NAME}_FRAMEWORKS = UIKit Foundation CoreGraphics QuartzCore
${TWEAK_NAME}_LIBRARIES = objc

include $(THEOS_MAKE_PATH)/tweak.mk

clean::
	rm -rf .theos packages
	cargo clean

fmt:
	@printf "\033[1;36m==>\033[0m Formatting Rust code...\n"
	cargo fmt

clippy:
	@printf "\033[1;36m==>\033[0m Running Clippy...\n"
	cargo clippy --all-targets --all-features -- -D warnings

deploy:
	@$(MAKE) fmt
	@$(MAKE) rust-build
	@if [ ! -f "$(THEOS_DYLIB)" ] || [ "$(RUST_LIB)" -nt "$(THEOS_DYLIB)" ]; then \
		printf "\033[1;36m==>\033[0m Rust lib changed, re-linking...\n"; \
		$(MAKE) theos-link; \
	else \
		printf "\033[1;36m==>\033[0m Dylib up-to-date, skipping link\n"; \
	fi
	@$(MAKE) package
	@printf "\033[1;36m==>\033[0m Deploy complete!\n"

rust-build:
	@printf "\033[1;36m==>\033[0m Building Rust library...\n"
	@cargo build --profile $(RUST_PROFILE) --target aarch64-apple-ios

theos-link:
	@find .theos/obj -name "*.dylib" -delete 2>/dev/null || true
	@$(MAKE) all

