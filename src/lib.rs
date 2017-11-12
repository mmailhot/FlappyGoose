#![no_std]
#![feature(lang_items)]
#![feature(iterator_step_by)]
#![feature(plugin)]

mod base;
mod gba;
mod bmp_tools;
mod assets;
mod title;

use base::rand::Rand;

const GOOSE_MAX_V : i32  = 48;

struct Goose<'a> {
    x: i32, // in 32nds of a pixel
    y: i32, // in 32nds of a pixel
    sprite: &'a mut gba::sprites::TagObjAttr,
    velocity: i32, // in 32nds of pixel per frame
    flap_frames: u32
}
impl<'a> Goose<'a> {
    fn init(&mut self){
        self.sprite.set_x((self.x / 32) as i16);
        self.sprite.set_y((self.y / 32) as i16);
        self.sprite.set_tile_index(assets::SpriteTileType::Goose1 as u16);
        self.sprite.set_hidden(false);
        self.sprite.set_sprite_shape(gba::sprites::SpriteShape::Wide);
        self.sprite.set_sprite_size(gba::sprites::SpriteSize::Large);
    }
    fn update(&mut self) -> bool{
        self.y += self.velocity;
        if self.y < -1024 {
            self.y = -1024;
        }

        if self.velocity != GOOSE_MAX_V {
            self.velocity += 2;
        }

        self.sprite.set_y((self.y / 32) as i16);
        if self.flap_frames == 1 {
            self.sprite.set_tile_index(assets::SpriteTileType::Goose1 as u16);
            self.flap_frames == 0;
        } else if self.flap_frames > 1 {
            self.flap_frames -= 1;
        }

        // Check floor collission
        if ((self.y / 32) > 140){
            return true;
        } else {
            return false;
        }
    }
    fn jump(&mut self){
        self.velocity = -48;
        self.sprite.set_tile_index(assets::SpriteTileType::Goose2 as u16);
        self.flap_frames = 5;
    }
}

struct Wall<'a> {
    x: i32,
    sprites: [&'a mut gba::sprites::TagObjAttr; 19],
    gap_top: i32,
    gap_bottom: i32,
    active: bool
}
impl <'a> Wall<'a>{
    fn new() -> Wall<'a>{
        let wall = Wall{
            x: 0,
            gap_top: 0,
            gap_bottom: 0,
            active: false,
            sprites: [
                gba::sprites::get_sprite(),
                gba::sprites::get_sprite(),
                gba::sprites::get_sprite(),
                gba::sprites::get_sprite(),
                gba::sprites::get_sprite(),
                gba::sprites::get_sprite(),
                gba::sprites::get_sprite(),
                gba::sprites::get_sprite(),
                gba::sprites::get_sprite(),
                gba::sprites::get_sprite(),
                gba::sprites::get_sprite(),
                gba::sprites::get_sprite(),
                gba::sprites::get_sprite(),
                gba::sprites::get_sprite(),
                gba::sprites::get_sprite(),
                gba::sprites::get_sprite(),
                gba::sprites::get_sprite(),
                gba::sprites::get_sprite(),
                gba::sprites::get_sprite(),
            ]
        };

        for i in 0..19 {
            wall.sprites[i].set_x(wall.x as i16);
            wall.sprites[i].set_y((i * 8) as i16);
            wall.sprites[i].set_tile_index(assets::SpriteTileType::Mine as u16);
            wall.sprites[i].set_hidden(true);
            wall.sprites[i].set_sprite_shape(gba::sprites::SpriteShape::Square);
            wall.sprites[i].set_sprite_size(gba::sprites::SpriteSize::Small);
        }

        return wall
    }

    fn update(&mut self){
        if !self.active {
            return;
        }
        let next_x = self.x - 1;
        self.set_x(next_x);

        if self.x < -8{
            self.kill();
        }
    }

    fn set_x(&mut self, x : i32){
        self.x = x;
        for i in 0..19 {
            self.sprites[i].set_x(x as i16);
        }
    }

    fn kill(&mut self){
        self.active = false;
        for i in 0..19{
            self.sprites[i].set_hidden(true);
        }
    }

    fn activate(&mut self, rand: &mut Rand){
        self.set_x(240);
        self.active = true;

        self.gap_top = ((rand.next_u8() as i32) >> 4) + 1;

        self.gap_bottom = ((rand.next_u8() as i32) >> 6) + 4 + self.gap_top;
        if self.gap_bottom > 19 {
            self.gap_bottom = 19;
        }

        for i  in 0..19{
            if i <= self.gap_top {
                self.sprites[i as usize].set_hidden(false);
            } else if i >= self.gap_bottom {
                self.sprites[i as usize].set_hidden(false);
            }
        }
    }

    fn check_collision(& self, goose: & Goose) -> bool {
        if !self.active {
            return false;
        }
        if (self.x < ((goose.x / 32) - 8)) || (self.x > ((goose.x / 32) + 32)){
            return false;
        }

        if (goose.y / 32) + 4 < (self.gap_top * 8) {
            return true;
        }

        if (goose.y / 32) + 12 > ((self.gap_bottom + 1) * 8) {
            return true;
        }

        return false;
    }
}


fn draw_mc(x: u32, left_width: u32, right_width:u32, floors: u32, screen_height:u32){
    use assets::BGTileType;
    // Draw top level]
    let top_y = screen_height - 3 - floors;
    gba::hw::set_screenblock_entry(8, x, top_y, BGTileType::MCTopLeft);
    for i in 0..left_width {
        gba::hw::set_screenblock_entry(8, x + 1 + i, top_y, BGTileType::MCTopMid);
    }
    gba::hw::set_screenblock_entry(8, x + left_width + 1, top_y, BGTileType::MCTowerTL);
    gba::hw::set_screenblock_entry(8, x + left_width + 2, top_y, BGTileType::MCTowerTR);
    for i in 0..right_width {
        gba::hw::set_screenblock_entry(8, x + left_width + 3 + i, top_y, BGTileType::MCTopMid);
    }
    gba::hw::set_screenblock_entry(8, x + left_width + right_width + 3, top_y, BGTileType::MCTopRight);

    for y_offset in 0..floors {
        let y = screen_height - 3 - y_offset;
        gba::hw::set_screenblock_entry(8, x, y, BGTileType::MCMidLeft);
        for i in 0..left_width {
            gba::hw::set_screenblock_entry(8, x + 1 + i, y, BGTileType::MCMidMid);
        }
        gba::hw::set_screenblock_entry(8, x + left_width + 1, y, BGTileType::MCTowerML);
        gba::hw::set_screenblock_entry(8, x + left_width + 2, y, BGTileType::MCTowerMR);
        for i in 0..right_width {
            gba::hw::set_screenblock_entry(8, x + left_width + 3 + i, y, BGTileType::MCMidMid);
        }
        gba::hw::set_screenblock_entry(8, x + left_width + right_width + 3, y, BGTileType::MCMidRight);
    }
    
    let bottom_y = screen_height - 2;
    gba::hw::set_screenblock_entry(8, x, bottom_y, BGTileType::MCLowLeft);
    for i in 0..left_width {
        gba::hw::set_screenblock_entry(8, x + 1 + i, bottom_y, BGTileType::MCLowMid);
    }
    gba::hw::set_screenblock_entry(8, x + left_width + 1, bottom_y, BGTileType::MCTowerLL);
    gba::hw::set_screenblock_entry(8, x + left_width + 2, bottom_y, BGTileType::MCTowerLR);
    for i in 0..right_width {
        gba::hw::set_screenblock_entry(8, x + left_width + 3 + i, bottom_y, BGTileType::MCLowMid);
    }
    gba::hw::set_screenblock_entry(8, x + left_width + right_width + 3, bottom_y, BGTileType::MCLowRight);
}

fn setup_map(){
    const NUM_CLOUDS : u32 = 20;
    const WIDTH : u32 = 64;
    const HEIGHT : u32 = 20;
    use assets::BGTileType;
    let mut rand = Rand::new(1234);
    gba::hw::dispcnt::bg0_enable::write(1);
    gba::palettes::load_bg_palette(&assets::bg_palette[0]);
    gba::tiles::load_tile_set(&assets::bg_tiles[0], 40, 0);
    gba::hw::bg0cnt::screenblock_base::write(8);
    gba::hw::bg0cnt::size::write(0b01);
    gba::hw::fill_screenblock(8, BGTileType::Sky);


    // Make grass
    for x in 0..WIDTH {
        gba::hw::set_screenblock_entry(8, x, HEIGHT-1, if(rand.next_u8() % 2 == 0) {BGTileType::Grass1} else {BGTileType::Grass2});
        gba::hw::set_screenblock_entry(9, x, HEIGHT-1, if(rand.next_u8() % 2 == 0) {BGTileType::Grass1} else {BGTileType::Grass2});
    }

    // Make clouds
    for clound_num in 0..NUM_CLOUDS {
        let x : u32 = (rand.next_u8() as u32) % WIDTH;
        let y : u32 = (rand.next_u8() as u32) % (HEIGHT - 1);
        let is_wide : bool = if x != (WIDTH - 1) {rand.next_bool()} else {false};
        if is_wide {
            gba::hw::set_screenblock_entry(8, x, y, BGTileType::BigCloudL);
            gba::hw::set_screenblock_entry(8, x + 1, y, BGTileType::BigCloudR);
        } else {
            gba::hw::set_screenblock_entry(8, x, y, BGTileType::Cloud);
        }
    }

    draw_mc(10, 6, 2 ,6, HEIGHT);
    draw_mc(28, 3, 3 ,4, HEIGHT);
    draw_mc(40, 3, 4 ,15, HEIGHT);
}

pub fn game_loop() {
    let mut key_state = gba::KeyState::new();
    //gba::hw::dispcnt::write(0);
    gba::hw::dispcnt::blank::write(0);
    gba::hw::dispcnt::obj_enable::write(1);
    gba::hw::dispcnt::mode::write(0);
    gba::hw::dispcnt::bg2_enable::write(0);
    gba::hw::dispcnt::obj_is_1d::write(1);
    setup_map();
    let mut offset : u16 = 0;
    let mut rand : Rand = Rand::new(1234);
    let mut ticks_till_next_wall : u8 = (rand.next_u8() % 64) + 64;
    gba::sprites::reset_allocator();
    gba::sprites::hide_all();
    gba::palettes::load_sprite_palette(&assets::sprite_palette[0]);
    gba::tiles::load_tile_set(&assets::sprite_tiles[0], 32, 4);
    let mut goose = Goose{x: 320, y: 320, sprite: gba::sprites::get_sprite(), velocity: 0, flap_frames: 0};
    goose.init();
    let mut walls = [
        Wall::new(),
        Wall::new(),
        Wall::new(),
        Wall::new(),
        Wall::new()
    ];
    walls[0].activate(&mut rand);
    loop {
        gba::wait_vblank();
        key_state.update();
        if goose.update(){
            return;
        }
        for i in 0..5 {
            walls[i].update();
            if walls[i].check_collision(&goose) {
                return;
            }
        }
        offset += 1;
        if key_state.is_triggered(gba::Key::A) {
            rand.next_u8();
            goose.jump();
        }
        gba::hw::write_bg0hofs(offset);
        gba::sprites::copy_shadow();
        ticks_till_next_wall -= 1;
        if ticks_till_next_wall == 0 {
            'wallloop: for i in 0..5 {
                if !walls[i].active {
                    walls[i].activate(&mut rand);
                    break 'wallloop;
                }
            }
            ticks_till_next_wall = (rand.next_u8() % 64) + 64;
        }
    }
}

#[no_mangle]
pub extern "C" fn main() {
    let mut key_state = gba::KeyState::new();
    gba::sprites::hide_all();
    loop {
        gba::hw::dispcnt::blank::write(0);
        gba::hw::dispcnt::bg0_enable::write(1);
        gba::hw::dispcnt::bg2_enable::write(1);
        gba::hw::dispcnt::obj_enable::write(0);
        gba::hw::dispcnt::mode::write(3);
        gba::hw::dispcnt::obj_is_1d::write(1);
        gba::hw::copy_vram32(&title::title_screen_bmp[0], title::title_screen_bmp.len());
        loop{
            key_state.update();
            if key_state.is_triggered(gba::Key::A) {
                game_loop();
                break;
            }
        }
    }
}
