#![allow(dead_code)]

pub mod sprites {
    pub enum SpriteShape {
        Square = 0b00,
        Wide = 0b01,
        Tall = 0b10
    }
    
    pub enum SpriteSize {
        Small = 0b00,
        Medium = 0b01,
        Large = 0b10,
        XLarge = 0b11
    }

    #[derive(Clone, Copy)]
    pub struct TagObjAttr {
        attr0: u16,
        attr1: u16,
        attr2: u16,
        _fill: u16
    }
    impl TagObjAttr {
        pub fn set_y(&mut self, val: i16){
            self.attr0 = (self.attr0 & 0xFF00) | ((val as u16) & 0xFF);
        }
        pub fn set_x(&mut self, val: i16){
            self.attr1 = (self.attr1 & 0xFF00) | ((val as u16) & 0xFF);
        }
        pub fn set_hidden(&mut self, is_hidden: bool){
            if is_hidden {
                self.attr0 = self.attr0 | 0x200;
            } else {
                self.attr0 = self.attr0 & 0xFDFF;
            }
        }
        pub fn set_sprite_shape(&mut self, val: SpriteShape){
            self.attr0 = (self.attr0 & 0x3FFF) | ((val as u16) << 14)
        }
        pub fn set_sprite_size(&mut self, val: SpriteSize){
            self.attr1 = (self.attr1 & 0x3FFF) | ((val as u16) << 14)
        }
        pub fn set_tile_index(&mut self, val: u16){
            self.attr2 = (self.attr2 & 0xFE00) | (val & 0x1FF);
        }
        pub fn set_palette_bank(&mut self, val: u16){
            self.attr2 = (self.attr2 & 0x0FFF) | (val << 12);
        }
    }

    static mut MAX_ALLOCATED_SPRITE : usize = 0;

    static mut OBJ_INFO: *mut [TagObjAttr; 128] = (0x7000000 as *mut [TagObjAttr; 128]);
    static mut OBJ_INFO_SHADOW: &'static mut [TagObjAttr; 128] = &mut[TagObjAttr{attr0:0, attr1:0, attr2:0, _fill:0}; 128];

    pub fn copy_shadow() {
        use core::ptr::{write_volatile};

        unsafe{
            write_volatile(OBJ_INFO, *OBJ_INFO_SHADOW);
        }
    }

    pub fn hide_all() {
        unsafe {
            for tile in OBJ_INFO_SHADOW.iter_mut(){
                tile.set_hidden(true);
            }
        }
    }

    pub fn reset_allocator() {
        unsafe {
            MAX_ALLOCATED_SPRITE = 0;
        }
    }
    
    pub fn get_sprite<'a>() -> &'a mut TagObjAttr {
        unsafe {
            MAX_ALLOCATED_SPRITE += 1;
            &mut OBJ_INFO_SHADOW[MAX_ALLOCATED_SPRITE - 1]
        }
    }
}

pub mod tiles {
    type Tile = [u32; 8];

    static mut TILE_MEM_4BPP: *mut [[Tile; 512]; 6] = (0x6000000 as *mut [[Tile; 512]; 6]);

    pub fn load_tile_set(tset_start: *const u32, n_tiles: usize, bank: usize){
        use core::mem::{transmute};
        use core::ptr::{write_volatile, read_volatile};
        unsafe {
            let tbank_start : *mut u32 = transmute(&mut((*TILE_MEM_4BPP)[bank][0]));
            for i in 0..(n_tiles * 8) {
                write_volatile(tbank_start.offset(i as isize), *tset_start.offset(i as isize));
                //panic!("{:p} {:p} \n {} {}", tbank_start.offset(i as isize), tset_start.offset(i as isize), read_volatile(tbank_start.offset(i as isize)), read_volatile(tset_start.offset(i as isize)));
            }
        }
    }
}

pub mod screenblock {
    
}

pub mod palettes {
    pub type Palette = [u32; 128];

    static mut BACKGROUND_PALLETE: *mut Palette = (0x5000000 as *mut Palette);
    static mut SPRITE_PALLETE: *mut Palette = (0x5000200 as *mut Palette);

    pub fn load_sprite_palette(pal: *const u32){
        use core::ptr::write_volatile;
        use core::mem::transmute;
        unsafe {
            write_volatile(SPRITE_PALLETE, *(pal as *const Palette));
        }
    }

    pub fn load_bg_palette(pal: *const u32) {
        use core::ptr::write_volatile;
        use core::mem::transmute;
        unsafe {
            write_volatile(BACKGROUND_PALLETE, *(pal as *const Palette));
        }
    }
}

pub mod hw {
    use assets::BGTileType;
    use core::ptr::{read_volatile, write_volatile};

    unsafe fn read16(addr: u32) -> u16 {
        read_volatile(addr as *const u16)
    }

    unsafe fn write16(addr: u32, value: u16) {
        write_volatile(addr as *mut u16, value);
    }

    unsafe fn write32(addr: u32, value: u32) {
        write_volatile(addr as *mut u32, value);
    }

    macro_rules! hw_reg {
        (rw $addr: expr, $read:ident, $write: ident) => {
            #[allow(dead_code)]
            pub fn $read() -> u16 {
                unsafe { read16($addr) }
            }

            #[allow(dead_code)]
            pub fn $write(value: u16) {
                unsafe { write16($addr, value) }
            }
        };
        (r $addr: expr, $read: ident) => {
            #[allow(dead_code)]
            pub fn $read() -> u16 {
                unsafe { read16($addr) }
            }
        };
        (w $addr: expr, $write: ident) => {
            #[allow(dead_code)]
            pub fn $write(value: u16) {
                unsafe { write16($addr, value) }
            }
        };
    }

    macro_rules! io_sub_reg {
        (rw $addr:expr, $sub_name:ident, $sub_off_start:expr, $sub_off_end:expr) => {
            #[allow(dead_code)]
            pub mod $sub_name {
                use super::{read16, write16};
                pub fn write(value:u16) {
                    let old_val = unsafe { read16($addr) };
                    let old_val_cleared = old_val & !(((1 << ($sub_off_end - $sub_off_start)) - 1) << $sub_off_start);
                    unsafe { write16($addr, old_val_cleared | (value << $sub_off_start)) }
                }
                pub fn read() -> u16 {
                    let whole_reg = unsafe { read16($addr) };
                    (whole_reg >> $sub_off_start) & ((1 << ($sub_off_end - $sub_off_start)) - 1)
                }
            }
        };
        (r $addr:expr, $sub_name:ident, $sub_off_start:expr, $sub_off_end:expr) => {
            #[allow(dead_code)]
            pub mod $sub_name {
                use super::{read16};
                pub fn read() -> u16 {
                    let whole_reg = unsafe { read16($addr) };
                    (whole_reg >> $sub_off_start) & ((1 << ($sub_off_end - $sub_off_start)) - 1)
                }
            }
        };
    }
    
    macro_rules! io_reg {
        (rw $addr:expr, $name:ident, $($type:ident $sub_name:ident => $sub_off_start:tt..$sub_off_end:tt),*) => {
            #[allow(dead_code)]
            pub mod $name {
                use super::{read16, write16};
                $(
                    io_sub_reg!($type $addr, $sub_name, $sub_off_start, $sub_off_end);
                )*

                pub fn write(value:u16) {
                    unsafe { write16($addr, value) }
                }

                pub fn read() -> u16 {
                    unsafe { read16($addr) }
                }
            }
        };
        (r $addr:expr, $name:ident, $($type:ident $sub_name:ident => $sub_off_start:tt..$sub_off_end:tt),*) => {
            #[allow(dead_code, unused_imports)]
            pub mod $name {
                use super::{read16, write16};
                $(
                    io_sub_reg!($type $addr, $sub_name, $sub_off_start, $sub_off_end);
                )*

                pub fn read() -> u16 {
                    unsafe { read16($addr) }
                }
            }
        };
    }

    io_reg!(rw 0x4000000, dispcnt,
            rw mode => 0..3,
            r is_gbc => 3..4,
            rw page_select => 4..5,
            rw allow_oam_in_hblank => 5..6,
            rw obj_is_1d => 6..7, // 0 = 2D, 1 = 1D
            rw blank => 7..8,
            rw bg0_enable => 8..9,
            rw bg1_enable => 9..10,
            rw bg2_enable => 10..11,
            rw bg3_enable => 11..12,
            rw obj_enable => 12..13,
            rw w0_enable => 13..14,
            rw w1_enable => 14..15,
            rw wobj_enable => 15..16
    );
    io_reg!(rw 0x4000004, dispstat,
            r in_vbl => 0..1,
            r in_hbl => 1..2,
            r in_vct => 2..3,
            rw vbl_irq => 3..4,
            rw hbl_irg => 4..5,
            rw vct_irg => 5..6,
            rw vct_trigger_value => 8..16
    );
    io_reg!(r 0x4000008, vcount, );
    io_reg!(rw 0x4000008, bg0cnt,
            rw priority => 0..2,
            rw charblock => 2..4,
            rw mosaic => 6..7,
            rw color_mode => 7..8,
            rw screenblock_base => 8..13,
            rw affine_wrap => 13..14,
            rw size => 14..16
    );
    hw_reg!(rw 0x400000a, read_bg1cnt, write_bg1cnt);
    hw_reg!(rw 0x400000c, read_bg2cnt, write_bg2cnt);
    hw_reg!(rw 0x400000e, read_bg3cnt, write_bg3cnt);
    hw_reg!(w 0x4000010, write_bg0hofs);
    hw_reg!(w 0x4000012, write_bg0vofs);
    hw_reg!(w 0x4000014, write_bg1hofs);
    hw_reg!(w 0x4000016, write_bg1vofs);
    hw_reg!(w 0x4000018, write_bg2hofs);
    hw_reg!(w 0x400001a, write_bg2vofs);
    hw_reg!(w 0x400001c, write_bg3hofs);
    hw_reg!(w 0x400001e, write_bg3vofs);
    io_reg!(r 0x4000130, keyinput,
            r a => 0..1,
            r b => 1..2,
            r select => 2..3,
            r start => 3..4,
            r right => 4..5,
            r left => 5..6,
            r up => 6..7,
            r down => 7..8,
            r r => 8..9,
            r l => 9..10
    );

    pub fn write_pal(index: u32, col: u16) {
        if index < 512 {
            unsafe { write16(0x5000000u32 + (index * 2) as u32, col) }
        }
    }

    pub fn write_vram16(offset: u32, data: u16) {
        if offset < 0xc000 {
            unsafe { write16(0x6000000u32 + offset * 2, data) }
        }
    }
    
    pub fn set_screenblock_entry(base_block: u32, x: u32, y: u32, tile: BGTileType) {
        let block_offset = (32 * (y % 32)) + (x % 32);
        let x_block = if x >= 32 {1} else {0};
        let y_block = if y >= 32 {2} else {0};
        write_vram16(0x400 * (base_block + x_block + y_block) + block_offset, (tile as u16))
    }
    
    pub fn fill_screenblock(baseBlock: u32, tile: BGTileType) {
        fill_vram16(0x400 * baseBlock, (tile as u16), 0x800)
    }

    pub fn fill_vram16(offset: u32, data: u16, size: u32) {
        if size % 2 == 0 && offset % 2 == 0 {
            let big_data : u32 = ((data as u32) << 16) | (data as u32);
            for i in (0..(size * 2)).step_by(4) {
                unsafe { write32(0x6000000u32 + offset * 2 + i, big_data); }
            }
        }else{
            if offset < 0xc000 {
                for i in (0..(size * 2)).step_by(2) {
                    unsafe { write16(0x6000000u32 + offset * 2 + i, data); }
                }
            }
        }
    }
    
    pub fn copy_vram32(data: *const u32, count: usize) {
        use core::ptr::{write_volatile, read_volatile};
        unsafe{
            for i in 0..count{
                write_volatile((0x6000000u32 as *mut u32).offset(i as isize), *(data.offset(i as isize)));
                //panic!("{:p} {:p} \n {} {}", (0x6000000u32 as *mut u32).offset(i as isize),  (data.offset(i as isize)),
                       //read_volatile((0x6000000u32 as *mut u32).offset(i as isize)), read_volatile(data.offset(i as isize)) );
                
            }
        }
    }
}

pub struct KeyState {
    state: u32,
}
pub enum Key {
    A = 1,
    B = 2,
    Select = 4,
    Start = 8,
    Right = 16,
    Left = 32,
    Up = 64,
    Down = 128,
    R = 256,
    L = 512,
}

impl KeyState {
    pub fn new() -> KeyState {
        KeyState { state: 0 }
    }
    pub fn update(&mut self) {
        let pressed = hw::keyinput::read() ^ 0xffffu16;
        let triggered = pressed & !self.get_pressed();
        self.state = (pressed as u32) | ((triggered as u32) << 16);
    }
    fn get_pressed(&self) -> u16 {
        self.state as u16
    }
    fn get_triggered(&self) -> u16 {
        (self.state >> 16) as u16
    }
    #[allow(dead_code)]
    pub fn is_pressed(&self, key: Key) -> bool {
        self.get_pressed() & (key as u16) != 0
    }
    #[allow(dead_code)]
    pub fn is_triggered(&self, key: Key) -> bool {
        self.get_triggered() & (key as u16) != 0
    }
}

pub fn wait_vblank() {
    while hw::dispstat::in_vbl::read() & 1 != 0 {}
    while hw::dispstat::in_vbl::read() & 1 == 0 {}
}
