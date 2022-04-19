# Note: You need to have the toolchain set up for all of this to work.
#
# TODO: Document how ^^

_ESP_TOOLS := "$(HOME)/.espressif/tools"
_CLANG_VER :="esp-13.0.0-20211203-x86_64-unknown-linux-gnu"
_CLANG_DIR := "$(_ESP_TOOLS)/xtensa-esp32-elf-clang/$(_CLANG_VER)"
_GCC_VER := "xtensa-esp32-elf-gcc8_4_0-esp-2021r2-patch3"
_GCC_DIR := "$(_ESP_TOOLS)/xtensa-esp32-elf-gcc/$(_GCC_VER)"

X_FEATURES = "xtensa-lx-rt/esp32"

all:
	$(MAKE) cargo CMD=build

release:
	$(MAKE) cargo CMD="build --release"

cargo:
	LIBCLANG_PATH="$(_CLANG_DIR)/lib" \
	PATH="$(_GCC_DIR)/bin:$(_CLANG_DIR)/bin:$(PATH)" \
	PIP_USER=no \
		cargo $(CMD) --features="$(X_FEATURES)"

objdump:
	$(_CLANG_DIR)/bin/xtensa-esp32-elf-objdump -D \
	  target/xtensa-esp32-none-elf/debug/badge_2021_rs

flash:
	$(MAKE) cargo CMD="espflash --speed=115200"
