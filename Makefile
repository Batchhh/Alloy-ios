# Rust library
RUST_PROFILE ?= dev-release

# Auto-detect sccache for faster rebuilds
SCCACHE := $(shell command -v sccache 2> /dev/null)
ifdef SCCACHE
    export RUSTC_WRAPPER := sccache
endif

before-all::
	@printf "\033[1;36m==>\033[0m Building Rust library...\n"
	cargo build --profile $(RUST_PROFILE) --target aarch64-apple-ios

# Target configuration
ARCHS = arm64
TARGET := iphone:clang:latest:14.0
FINALPACKAGE = 1

# Device config (change IP as needed)
DEVICE_IP ?= 1.1.1
DEVICE_USER ?= mobile
DEVICE_PASS ?= 1

include $(THEOS)/makefiles/common.mk

TWEAK_NAME = rgg

${TWEAK_NAME}_CFLAGS = -fobjc-arc
${TWEAK_NAME}_LDFLAGS = -force_load target/aarch64-apple-ios/$(RUST_PROFILE)/librust_igmm.a
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

# We are using sshpass to avoid typing the password every time
deploy:
	$(MAKE) fmt
	@printf "\033[1;36m==>\033[0m Cleaning...\n"
	find .theos/obj -name "*.dylib" -delete 2>/dev/null || true
	$(MAKE) all
	$(MAKE) package
	@printf "\033[1;36m=>\033[0m Copying package to device...\n"
	@sshpass -p "$(DEVICE_PASS)" scp packages/*.deb $(DEVICE_USER)@$(DEVICE_IP):~
	@printf "\033[1;36m>\033[0m Deploy complete.\n"
