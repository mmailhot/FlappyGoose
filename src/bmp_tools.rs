pub mod m3 {
    use gba;

    const M3_WIDTH : u32 = 240;
    const M3_HEIGHT : u32 = 160;
    const M3_SIZE : u32 = M3_WIDTH * M3_HEIGHT;
 
    pub fn fill(color: u16) {
        gba::hw::fill_vram16(0x0000, color, M3_SIZE);
    }
}

pub mod m4 {
    use gba;

    const M4_WIDTH : u32 = 240;
    const M4_HEIGHT : u32 = 160;
    const M4_SIZE : u32 = M4_WIDTH * M4_HEIGHT;
    
    pub fn fill(color: u8) {
        gba::hw::fill_vram16(0x0000, ((color as u16) << 8) | (color as u16), M4_SIZE / 2);
    }
}
