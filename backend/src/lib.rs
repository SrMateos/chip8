use rand::random;

// Must be public due to frontend access to them
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize   = 4096; // 4K RAM SIZE
const NUM_REGS: usize   = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize   = 16;
const FONTSET_SIZE: usize = 80;

const START_ADDR: u16 = 0x200;

const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Emu{
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_HEIGHT * SCREEN_WIDTH],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    dt: u8,
    st: u8,
    keys: [bool; NUM_KEYS],
}

impl Emu {
    pub fn new() -> Self{
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };
        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        
        new_emu    
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    fn push (&mut self, val: u16){
        if self.sp+1 > STACK_SIZE as u16 { eprintln!("Stack overflow");  std::process::exit(1); };
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        if self.sp-1 < 0 as u16  { eprintln!("Stack underflow");  std::process::exit(1); };
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    fn fetch (&mut self) -> u16{
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc+1) as usize] as u16;
        let operation = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        operation
    }   

    pub fn tick(&mut self){
        // Fetch 
        let operation = self.fetch();

        // Decode and execute
        self.execute(operation);
    }

    pub fn tick_timers(&mut self){
        if self.dt > 0 {
            self.dt -= 1;
        }
        
        if self.st > 0 {
            
            if self.st == 1{
                // BEEP
            }

            self.st -= 1;
        } 
    }

    fn execute(&mut self, operation: u16) {
        let digit1 = (operation & 0xF000) >> 12;
        let digit2 = (operation & 0x0F00) >> 8;
        let digit3 = (operation & 0x00F0) >> 4;
        let digit4 =  operation & 0x000F;

        match(digit1,digit2,digit3,digit4){
            
            // NOP
            (0,0,0,0) => return,
                        
            // CLEAR SCREEN 
            (0,0,0xE,0) => {
                self.screen = [false; SCREEN_HEIGHT * SCREEN_WIDTH];
            },
            
            // RETURN FROM SUBRUTINE
            (0,0,0xE,0xE) => {
                self.pc = self.pop();
            },

            // JUMP
            (1,_,_,_) => {
                let nnn = operation & 0xFFF;
                self.pc = nnn;
            },
            
            // CALL SUBRUTINE
            (2,_,_,_) => {
                let nnn = operation & 0xFFF;
                self.push(self.pc);
                self.pc = nnn;
            },
            
            // 3XNN skip next if vx == nn
            (3,_,_,_) => {
                let vx = digit2 as usize;
                let nn = (operation & 0xFF) as u8;
                if nn == self.v_reg[vx] {
                    self.pc += 2;
                }
            },

            // 4XNN skip next if vx != nn
            (4,_,_,_) => {
                let vx = digit2 as usize;
                let nn = (operation & 0xFF) as u8;
                if nn != self.v_reg[vx] {
                    self.pc += 2;
                }
            },

            // 5XY0 skip next if vx == vy
            (5,_,_,_) => {
                let vx = digit2 as usize;
                let vy = digit3 as usize;
                if self.v_reg[vx] == self.v_reg[vy] {
                    self.pc += 2;
                } 
            },

            // 6XNN vx = nn
            (6,_,_,_) => {
                let vx = digit2 as usize;
                self.v_reg[vx] = (operation & 0xFF) as u8;
            },

            // 7XNN vx += nn
            (7,_,_,_) => {
                let vx = digit2 as usize;
                let nn = (operation & 0xFF) as u8;
                self.v_reg[vx] = self.v_reg[vx].wrapping_add(nn);
            },

            // 8XY1 vx OR vy
            (8,_,_,1) => {
                let vx = digit2 as usize;
                let vy = digit3 as usize;
                self.v_reg[vx] |= self.v_reg[vy];
            },

            // 8XY2 vx AND vy
            (8,_,_,2) => {
                let vx = digit2 as usize;
                let vy = digit3 as usize;
                self.v_reg[vx] &= self.v_reg[vy];
            },

            // 8XY3 vx XOR vy
            (8,_,_,3) => {
                let vx = digit2 as usize;
                let vy = digit3 as usize;
                self.v_reg[vx] ^= self.v_reg[vy];
            },

            // 8XY4 vx += vy with carry
            (8,_,_,4) => {
                let vx = digit2 as usize;
                let vy = digit3 as usize;
                let (new_vx, carry) = self.v_reg[vx].overflowing_add(self.v_reg[vy]);
                let new_vf = if carry {1} else {0};
                self.v_reg[vx] = new_vx;
                self.v_reg[0xF] = new_vf;
            },

            // 8XY45vx -= vy with carry
            (8,_,_,5) => {
                let vx = digit2 as usize;
                let vy = digit3 as usize;
                let (new_vx, carry) = self.v_reg[vx].overflowing_sub(self.v_reg[vy]);
                let new_vf = if carry {0} else {1};
                self.v_reg[vx] = new_vx;
                self.v_reg[0xF] = new_vf;
            },

            // 8XY6 vx >>= 1 with lsb flag
            (8,_,_,6) => {
                let vx = digit2 as usize;
                let lsb = self.v_reg[vx] & 1;
                self.v_reg[vx] >>= 1;
                self.v_reg[0xF] = lsb;
            },

            // 8XY7 vx = vy - vx with carry
            (8,_,_,7) => {
                let vx = digit2 as usize;
                let vy = digit3 as usize;
                let (new_vx, carry) = self.v_reg[vy].overflowing_sub(self.v_reg[vx]);
                let new_vf = if carry {0} else {1};
                self.v_reg[vx] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            
            // 8XYE vx <<= 1 with lsb
            (8,_,_,0xE) => {
                let vx = digit2 as usize;
                let msb = (self.v_reg[vx] << 7) & 1;
                self.v_reg[vx] <<= 1;
                self.v_reg[0xF] = msb;
            },

            // 8XY0 vx = vy
            (8,_,_,_) => {
                let vx = digit2 as usize;
                let vy = digit3 as usize;
                self.v_reg[vx] = self.v_reg[vy]; 
            },

            // 9XY0 skip if vx != vy
            (9,_,_,0) => {
                let vx = digit2 as usize;
                let vy = digit3 as usize;
                if self.v_reg[vx] != self.v_reg[vy] {
                    self.pc += 2;
                }                
            },

            // ANNN I = NNN
            (0xA,_,_,_) => {
                let nnn = operation & 0xFFF;
                self.i_reg = nnn;
            },

            // Jump to V0 + NNN
            (0xB,_,_,_) => {
                let nnn = operation & 0xFFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            },

            (0xC,_,_,_) => {
                let vx = digit2 as usize;
                let nn = (operation & 0xFF) as u8;
                let rng: u8 = random();
                self.v_reg[vx] = rng & nn;
            },

            // UNINMPLEMENTED OPERATION
            (_,_,_,_) => unimplemented!("Unimplemented op code {}", operation),

        }
    }
}