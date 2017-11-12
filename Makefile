BINUTILS_PREFIX=arm-none-eabi-
RUST_LIBS=libs

default: out out/gbarust.gba

debug: out out/gbarust-debug.gba

out:
	mkdir -p out

cargo-build-release:
	rustup run nightly `which xargo` build --release --target=gba 

cargo-build-debug:
	rustup run nightly `which xargo` build --target=gba --pretty=expanded

out/gbarust.gba: cargo-build-release crt0.s
	$(BINUTILS_PREFIX)as -o out/crt0.o crt0.s
	$(BINUTILS_PREFIX)ld -T linker.ld -o out/gbarust.elf out/assets.o out/crt0.o target/gba/release/libgbarust.a
	$(BINUTILS_PREFIX)objcopy -O binary out/gbarust.elf out/gbarust.gba

out/gbarust-debug.gba: cargo-build-debug crt0.s
	$(BINUTILS_PREFIX)as -o out/crt0.o crt0.s
	$(BINUTILS_PREFIX)ld -T linker.ld -o out/gbarust-debug.elf out/assets.o out/crt0.o target/gba/debug/libgbarust.a
	$(BINUTILS_PREFIX)objcopy -O binary out/gbarust-debug.elf out/gbarust-debug.gba
