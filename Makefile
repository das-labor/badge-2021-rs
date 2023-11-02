ELF := target/xtensa-esp32-none-elf/debug/badge_2021_rs

build:
	. $(HOME)/export-esp.sh && \
		cargo build --release

flash:
	. $(HOME)/export-esp.sh && \
		cargo espflash flash --release --monitor

objdump:
	. $(HOME)/export-esp.sh && \
	xtensa-esp32-elf-objdump -D $(ELF)
