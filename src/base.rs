use gba;
use core::fmt::{Write, Error, write};

#[lang = "eh_personality"]
pub extern "C" fn rust_eh_personality() {}

struct BgWriter(u32);

impl Write for BgWriter {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for c in s.chars() {
            let c = c as u32;
            let clamped = if c < 32 || c > 126 { 32 } else { c };
            gba::hw::write_vram16(self.0, (clamped - 32) as u16);
            if c == 10 || ((self.0 & 31) >= 29) {
                self.0 = (self.0 + 32) & !31;
            } else {
                self.0 += 1;
            }
        }
        Ok(())
    }
}

#[allow(private_no_mangle_fns)]
#[no_mangle]
#[lang = "panic_fmt"]
pub extern "C" fn panic_fmt(_msg: ::core::fmt::Arguments,
                            _file: &'static str,
                            _line: u32)
                            -> ! {
    load_font(0);
    gba::hw::write_pal(0, 0);
    gba::hw::write_pal(15, 0x7fff);
    gba::hw::dispcnt::write(1 << 8);
    gba::hw::bg0cnt::write(2 << 8);
    for i in 0..(32 * 20) {
        gba::hw::write_vram16(0x800 + i, 0);
    }
    let mut writer = BgWriter(0x800);
    write(&mut writer,
          format_args!("Panic in line {} of\n{}\n\n{}", _line, _file, _msg))
        .unwrap();
    loop {}
}

pub fn load_font(offset: u32) {
    let font = include_bytes!("font.bin");
    for (index, byte) in font.iter().enumerate() {
        let mut line = 0u32;
        for bit in 0..7 {
            if (byte & (1 << bit)) != 0 {
                line |= 15 << (bit * 4);
            }
        }
        gba::hw::write_vram16(offset + index as u32 * 2, (line & 0xffff) as u16);
        gba::hw::write_vram16(offset + index as u32 * 2 + 1, (line >> 16) as u16);
    }
}

#[allow(dead_code)]
pub mod rand {
    pub struct Rand {
        state: u32,
    }

    impl Rand {
        pub fn new(seed: u32) -> Rand {
            Rand { state: seed }
        }
        pub fn next_bool(&mut self) -> bool {
            self.state = self.state.wrapping_mul(1664525u32).wrapping_add(1013904223u32);
            self.state & 0x80000000u32 != 0
        }
        pub fn next_u8(&mut self) -> u8 {
            let mut result = 0u8;
            for i in 0..8 {
                result |= (self.next_bool() as u8) << i;
            }
            result
        }
    }
}
