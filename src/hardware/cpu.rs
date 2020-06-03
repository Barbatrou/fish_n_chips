//!
//! CPU emulator
//!

use rand::Rng;
use super::memory::{Memory, Display};
use super::keyboard::Keyboard;

const STACK_SIZE: usize = 16;

#[derive(Debug)]
struct Stack
{
    stack : [u16; STACK_SIZE],
    stack_pointer: usize,
}

impl Stack
{
    pub fn new() -> Stack
    {
        Stack {
            stack: [0; STACK_SIZE],
            stack_pointer: 0,
        }
    }

    pub fn push(&mut self, address: u16)
    {
        if self.stack_pointer >= STACK_SIZE {
            panic!("ERROR: cpu stack overflow, too many nested subroutines: {:#?}", self);
        }
        self.stack[self.stack_pointer] = address;
        self.stack_pointer += 1;
    }

    pub fn top(&self) -> u16 { self.stack[self.stack_pointer] }

    pub fn pop(&mut self) -> u16
    {
        if self.stack_pointer == 0 {
            panic!("ERROR: cpu stack underflow: {:#?}", self);
        }
        self.stack_pointer -= 1;
        let address = self.top();
        address
    }
}

const PROGRAM_START_ADDRESS: usize = 0x200;
const OPCODE_SIZE: usize = 2;

enum ProgramCounter
{
    NEXT,
    SKIP,
    JUMP(u16),
}

impl ProgramCounter
{
    pub fn skip_if(condition: bool) -> ProgramCounter
    {
        match condition
        {
            true => ProgramCounter::SKIP,
            false => ProgramCounter::NEXT,
        }
    }
}

pub struct Cpu
{
    v_registers: [u8; 16],
    i_register: u16,
    delay_timer_register: u8,
    sound_timer_register: u8,

    pc : usize,

    stack : Stack,

    opcode: u16,

    waiting_for_input: bool,
    input_register: usize,

    pub beeping: bool,
}

impl Cpu
{
    pub fn new() -> Cpu
    {
        Cpu {
            v_registers: [0; 16],
            i_register: 0,
            delay_timer_register: 0,
            sound_timer_register: 0,
            pc: PROGRAM_START_ADDRESS,
            stack: Stack::new(),
            opcode: 0,
            waiting_for_input: false,
            input_register: 0,
            beeping: false,
        }
    }

    pub fn fetch_opcode(&mut self, memory: &Memory) -> u16
    {
        self.opcode = (memory[self.pc] as u16) << 8 | memory[self.pc + 1] as u16;
        self.opcode
    }

    pub fn execute_opcode(&mut self, memory: &mut Memory, keyboard: &Keyboard)
    {
        let splitted_opcode = (
            ((self.opcode & 0xF000) >> 12) as u8,
            ((self.opcode & 0x0F00) >> 8) as u8,
            ((self.opcode & 0x00F0) >> 4) as u8,
            (self.opcode & 0x000F) as u8,
        );
        let nnn = (self.opcode & 0x0FFF) as u16;
        let kk = (self.opcode & 0x00FF) as u8;
        let x = splitted_opcode.1 as usize;
        let y = splitted_opcode.2 as usize;
        let n = splitted_opcode.3 as usize;

        let program_counter_next_operation = match splitted_opcode {
            (0x00, 0x00, 0x0e, 0x00) => self.op_00e0(&mut memory.display),
            (0x00, 0x00, 0x0e, 0x0e) => self.op_00ee(),
            (0x01, _, _, _) => self.op_1nnn(nnn),
            (0x02, _, _, _) => self.op_2nnn(nnn),
            (0x03, _, _, _) => self.op_3xkk(x, kk),
            (0x04, _, _, _) => self.op_4xkk(x, kk),
            (0x05, _, _, 0x00) => self.op_5xy0(x, y),
            (0x06, _, _, _) => self.op_6xkk(x, kk),
            (0x07, _, _, _) => self.op_7xkk(x, kk),
            (0x08, _, _, 0x00) => self.op_8xy0(x, y),
            (0x08, _, _, 0x01) => self.op_8xy1(x, y),
            (0x08, _, _, 0x02) => self.op_8xy2(x, y),
            (0x08, _, _, 0x03) => self.op_8xy3(x, y),
            (0x08, _, _, 0x04) => self.op_8xy4(x, y),
            (0x08, _, _, 0x05) => self.op_8xy5(x, y),
            (0x08, _, _, 0x06) => self.op_8xy6(x, y),
            (0x08, _, _, 0x07) => self.op_8xy7(x, y),
            (0x08, _, _, 0x0E) => self.op_8xye(x, y),
            (0x09, _, _, 0x00) => self.op_9xy0(x, y),
            (0x0A, _, _, _) => self.op_annn(nnn),
            (0x0B, _, _, _) => self.op_bnnn(nnn),
            (0x0C, _, _, _) => self.op_cxkk(x, kk),
            (0x0d, _, _, _) => self.op_dxyn(x, y, n, memory),
            (0x0e, _, 0x09, 0x0e) => self.op_ex9e(x, keyboard),
            (0x0e, _, 0x0a, 0x01) => self.op_exa1(x, keyboard),
            (0x0f, _, 0x00, 0x07) => self.op_fx07(x),
            (0x0f, _, 0x00, 0x0a) => self.op_fx0a(x),
            (0x0f, _, 0x01, 0x05) => self.op_fx15(x),
            (0x0f, _, 0x01, 0x08) => self.op_fx18(x),
            (0x0f, _, 0x01, 0x0e) => self.op_fx1e(x),
            (0x0f, _, 0x02, 0x09) => self.op_fx29(x),
            (0x0f, _, 0x03, 0x03) => self.op_fx33(x, memory),
            (0x0f, _, 0x05, 0x05) => self.op_fx55(x, memory),
            (0x0f, _, 0x06, 0x05) => self.op_fx65(x, memory),
            _ => ProgramCounter::NEXT,
        };
        match program_counter_next_operation {
            ProgramCounter::NEXT => self.pc += OPCODE_SIZE,
            ProgramCounter::SKIP => self.pc += OPCODE_SIZE * 2,
            ProgramCounter::JUMP(address) => self.pc = address as usize,
        }
    }

    pub fn update_timers(&mut self) -> Result<(), ()>
    {
        if !self.waiting_for_input {
            if self.delay_timer_register > 0 {
                self.delay_timer_register -= 1;
            }
            if self.sound_timer_register > 0 {
                self.sound_timer_register -= 1;
            }
            return Ok(())
        }
        return Err(());
    }

    pub fn do_cycle(&mut self, memory: &mut Memory, keyboard: &Keyboard)
    {
        if self.waiting_for_input && keyboard.iter().any(|x| *x == 1) {
            self.waiting_for_input = false;
            self.v_registers[self.input_register] = keyboard.iter().position(|x| *x == 1 as u8).unwrap() as u8;
        }
        if !self.waiting_for_input {
            // execute new instruction
            self.fetch_opcode(memory);
            self.execute_opcode(memory, keyboard);

            if self.sound_timer_register > 0 {
                self.beeping = true;
            } else {
                self.beeping = false;
            }
        }
    }

    // opcode instructions:
    //
    // variables meanings
    // nnn or addr - A 12-bit value, the lowest 12 bits of the instruction
    // n or nibble - A 4-bit value, the lowest 4 bits of the instruction
    // x - A 4-bit value, the lower 4 bits of the high byte of the instruction
    // y - A 4-bit value, the upper 4 bits of the low byte of the instruction
    // kk or byte - An 8-bit value, the lowest 8 bits of the instruction
    //
    // (notation come from [Cowgod's Chip-8 technical documentation](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM))
    //

    fn op_00e0(&self, display: &mut Display) -> ProgramCounter // CLS - clear the display
    {
        display.clear();
        ProgramCounter::NEXT
    }

    fn op_00ee(&mut self) -> ProgramCounter // RET - return from a subroutine
    {
        ProgramCounter::JUMP(self.stack.pop())
    }

    fn op_1nnn(&mut self, nnn: u16) -> ProgramCounter // JP addr - Jump at location nnn
    {
        ProgramCounter::JUMP(nnn)
    }

    fn op_2nnn(&mut self, nnn: u16) -> ProgramCounter // CALL addr - Call subroutine at location nnn
    {
        self.stack.push((self.pc + OPCODE_SIZE) as u16);
        ProgramCounter::JUMP(nnn)
    }

    fn op_3xkk(&mut self, x: usize, kk: u8) -> ProgramCounter // SE Vx, byte - Skip next instruction if Vx = kk
    {
        ProgramCounter::skip_if(self.v_registers[x] == kk)
    }

    fn op_4xkk(&mut self, x: usize, kk: u8) -> ProgramCounter // SNE Vx, byte - Skip next instruction if Vx != kk
    {
        ProgramCounter::skip_if(self.v_registers[x] != kk)
    }

    fn op_5xy0(&mut self, x: usize, y: usize) -> ProgramCounter // SE Vx, Vy - Skip next instruction if Vx = Vy
    {
        ProgramCounter::skip_if(self.v_registers[x] == self.v_registers[y])
    }

    fn op_6xkk(&mut self, x: usize, kk: u8) -> ProgramCounter // LD Vx, byte - Set Vx = kk
    {
        self.v_registers[x] = kk;
        ProgramCounter::NEXT
    }

    fn op_7xkk(&mut self, x: usize, kk: u8) -> ProgramCounter // ADD Vx, byte - Set Vx = Vx + kk.
    {
        self.v_registers[x] = self.v_registers[x].wrapping_add(kk);
        ProgramCounter::NEXT
    }

    fn op_8xy0(&mut self, x: usize, y: usize) -> ProgramCounter // LD Vx, Vy- Set Vx = Vy.
    {
        self.v_registers[x] = self.v_registers[y];
        ProgramCounter::NEXT
    }

    fn op_8xy1(&mut self, x: usize, y: usize) -> ProgramCounter // OR Vx, Vy - Set Vx = Vx OR Vy.
    {
        self.v_registers[x] = self.v_registers[x] | self.v_registers[y];
        ProgramCounter::NEXT
    }

    fn op_8xy2(&mut self, x: usize, y: usize) -> ProgramCounter // AND Vx, Vy - Set Vx = Vx AND Vy.
    {
        self.v_registers[x] = self.v_registers[x] & self.v_registers[y];
        ProgramCounter::NEXT
    }

    fn op_8xy3(&mut self, x: usize, y: usize) -> ProgramCounter // XOR Vx, Vy - Set Vx = Vx XOR Vy.
    {
        self.v_registers[x] = self.v_registers[x] ^ self.v_registers[y];
        ProgramCounter::NEXT
    }

    fn op_8xy4(&mut self, x: usize, y: usize) -> ProgramCounter // ADD Vx, Vy - Set Vx = Vx + Vy, set VF = carry.
    {
        let vx = self.v_registers[x] as u16;
        let vy = self.v_registers[y] as u16;
        let vx = vx + vy;
        self.v_registers[0x0F] = if vx > 255 { 1 } else { 0 };
        self.v_registers[x] = vx as u8;
        ProgramCounter::NEXT
    }

    fn op_8xy5(&mut self, x: usize, y: usize) -> ProgramCounter // SUB Vx, Vy - Set Vx = Vx - Vy, set VF = NOT borrow.
    {
        self.v_registers[0x0F] = if self.v_registers[x] > self.v_registers[y] { 1 } else { 0 };
        self.v_registers[x] = self.v_registers[x].wrapping_sub(self.v_registers[y]);
        ProgramCounter::NEXT
    }

    fn op_8xy6(&mut self, x: usize, _y: usize) -> ProgramCounter // SHR Vx {, Vy} - Set Vx = Vx SHR 1.
    {
        self.v_registers[0x0F] = self.v_registers[x] & 1;
        self.v_registers[x] >>= 1;
        ProgramCounter::NEXT
    }

    fn op_8xy7(&mut self, x: usize, y: usize) -> ProgramCounter // SUBN Vx, Vy - Set Vx = Vy - Vx, set VF = NOT borrow.
    {
        self.v_registers[0x0F] = if self.v_registers[x] > self.v_registers[y] { 1 } else { 0 };
        self.v_registers[x] = self.v_registers[y].wrapping_sub(self.v_registers[x]);
        ProgramCounter::NEXT
    }

    fn op_8xye(&mut self, x: usize, _y: usize) -> ProgramCounter // SHL Vx {, Vy} - Set Vx = Vx SHL 1.
    {
        self.v_registers[0x0F] = (self.v_registers[x] & 0b10000000) >> 7;
        self.v_registers[x] <<= 1;
        ProgramCounter::NEXT
    }

    fn op_9xy0(&mut self, x: usize, y: usize) -> ProgramCounter // SNE Vx, Vy - Skip next instruction if Vx != Vy.
    {
        ProgramCounter::skip_if(self.v_registers[x] != self.v_registers[y])
    }

    fn op_annn(&mut self, nnn: u16) -> ProgramCounter // LD I, addr - Set I = nnn.
    {
        self.i_register = nnn;
        ProgramCounter::NEXT
    }

    fn op_bnnn(&mut self, nnn: u16) -> ProgramCounter // JP V0, addr - Jump to location nnn + V0.
    {
        ProgramCounter::JUMP(nnn + self.v_registers[0] as u16)
    }

    fn op_cxkk(&mut self, x: usize, kk: u8) -> ProgramCounter // RND Vx, byte - Set Vx = random byte AND kk.
    {
        let mut rng = rand::thread_rng();
        let random = rng.gen_range(0, 255);
        self.v_registers[x] = random & kk;
        ProgramCounter::NEXT
    }

    fn op_dxyn(&mut self, x: usize, y: usize, n: usize, memory: &mut Memory) -> ProgramCounter // DRW Vx, Vy, nibble - Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    {
        self.v_registers[0x0F] = 0;
        let (width, height) = memory.display.get_sizes();
        for byte in 0..n {
            let y: usize = (self.v_registers[y].wrapping_add(byte as u8)) as usize % height;
            for bit in 0..8 {
                let x: usize = (self.v_registers[x].wrapping_add(bit as u8)) as usize % width;
                let pixel = (memory[self.i_register as usize + byte] >> (7 - bit)) & 1;
                self.v_registers[0x0F] |= pixel & memory.display[[x,y]];
                memory.display[[x,y]] ^= pixel;
            }
        }
        ProgramCounter::NEXT
    }

    fn op_ex9e(&mut self, x: usize, keyboard: &Keyboard) -> ProgramCounter // SKP Vx - Skip next instruction if key with the value of Vx is pressed.
    {
        ProgramCounter::skip_if(keyboard[self.v_registers[x] as usize] == 1)
    }

    fn op_exa1(&mut self, x: usize, keyboard: &Keyboard) -> ProgramCounter // SKNP Vx - Skip next instruction if key with the value of Vx is not pressed.
    {
        ProgramCounter::skip_if(keyboard[self.v_registers[x] as usize] == 0)
    }

    fn op_fx07(&mut self, x: usize) -> ProgramCounter // LD Vx, DT - Set Vx = delay timer value.
    {
        self.v_registers[x] = self.delay_timer_register;
        ProgramCounter::NEXT
    }

    fn op_fx0a(&mut self, x: usize) -> ProgramCounter // LD Vx, K - Wait for a key press, store the value of the key in Vx.
    {
        self.waiting_for_input = true;
        self.input_register = x;
        ProgramCounter::NEXT
    }

    fn op_fx15(&mut self, x: usize) -> ProgramCounter // LD DT, Vx - Set delay timer = Vx.
    {
        self.delay_timer_register = self.v_registers[x];
        ProgramCounter::NEXT
    }

    fn op_fx18(&mut self, x: usize) -> ProgramCounter // LD ST, Vx - Set sound timer = Vx.
    {
        self.sound_timer_register = self.v_registers[x];
        ProgramCounter::NEXT
    }

    fn op_fx1e(&mut self, x: usize) -> ProgramCounter // ADD I, Vx - Set I = I + Vx.
    {
        self.i_register += self.v_registers[x] as u16;
        ProgramCounter::NEXT
    }

    fn op_fx29(&mut self, x: usize) -> ProgramCounter // LD F, Vx - Set I = location of sprite for digit Vx.
    {
        self.i_register = self.v_registers[x] as u16 * 5;
        ProgramCounter::NEXT
    }

    fn op_fx33(&mut self, x: usize, memory: &mut Memory) -> ProgramCounter // LD B, Vx - Store BCD representation of Vx in memory locations I, I+1, and I+2.
     {
        memory[self.i_register as usize] = self.v_registers[x] / 100;
        memory[self.i_register as usize + 1] = self.v_registers[x] % 100 / 10;
        memory[self.i_register as usize + 2] = self.v_registers[x] % 10;
        ProgramCounter::NEXT
    }

    fn op_fx55(&mut self, x: usize, memory: &mut Memory) -> ProgramCounter // LD [I], Vx - Store registers V0 through Vx in memory starting at location I.
    {
        for index in 0..x + 1 {
            memory[self.i_register as usize + index] = self.v_registers[index];
        }
        ProgramCounter::NEXT
    }

    fn op_fx65(&mut self, x: usize, memory: &Memory) -> ProgramCounter // LD Vx, [I] - Read registers V0 through Vx from memory starting at location I.
    // The interpreter reads values from memory starting at location I into registers V0 through Vx.
    {
        for index in 0..x + 1 {
             self.v_registers[index] = memory[self.i_register as usize + index];
        }
        ProgramCounter::NEXT
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    #[test]
    fn cpu_initial_state()
    {
        let cpu = Cpu::new();
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.stack.stack_pointer, 0);
        assert_eq!(cpu.stack.stack, [0; 16]);
    }

    #[test]
    fn test_op00e0()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        cpu.pc = 0x200;
        cpu.opcode = 0x00E0;

        cpu.execute_opcode(&mut mem, &key);

        // TODO check that the display has indeed been cleaned

        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE)
    }

    #[test]
    fn test_op00ee()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        cpu.opcode = 0x00EE;

        cpu.stack.stack_pointer = 5;
        cpu.stack.stack[4] = 0x4444;
        cpu.execute_opcode(&mut mem, &key);

        assert_eq!(cpu.stack.stack_pointer, 4);
        assert_eq!(cpu.pc, 0x4444);
    }

    #[test]
    fn test_op1nnn()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        cpu.opcode = 0x1300;

        cpu.pc = 0x200;
        cpu.execute_opcode(&mut mem, &key);

        assert_eq!(cpu.pc, 0x300);
    }

    #[test]
    fn test_op2nnn()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        cpu.opcode = 0x2300;

        cpu.pc = 0x200;
        cpu.stack.stack_pointer = 2;
        cpu.stack.stack[2] = 0x4444;
        cpu.execute_opcode(&mut mem, &key);

        assert_eq!(cpu.stack.stack_pointer, 3);
        assert_eq!(cpu.stack.stack[2], 0x200 + OPCODE_SIZE as u16);
        assert_eq!(cpu.pc, 0x300);
    }

    #[test]
    fn test_op3xkk()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        // With satisfied predicate
        cpu.opcode = 0x3469;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x69;
        cpu.execute_opcode(&mut mem, &key);

        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE * 2);

        // With unsatisfied predicate
        cpu.opcode = 0x3469;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x49;
        cpu.execute_opcode(&mut mem, &key);

        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_op4xkk()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        // With satisfied predicate
        cpu.opcode = 0x4469;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x49;
        cpu.execute_opcode(&mut mem, &key);

        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE * 2);

        // With unsatisfied predicate
        cpu.opcode = 0x4469;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x69;
        cpu.execute_opcode(&mut mem, &key);

        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_op5xy0()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        // With satisfied predicate
        cpu.opcode = 0x5440;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x04;
        cpu.execute_opcode(&mut mem, &key);

        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE * 2);

        // With unsatisfied predicate
        cpu.opcode = 0x5460;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x04;
        cpu.v_registers[0x06] = 0x06;
        cpu.execute_opcode(&mut mem, &key);

        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_op6xkk()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        cpu.opcode = 0x6440;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x04;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.v_registers[0x04], 0x40);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_op7xkk()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        cpu.opcode = 0x7440;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x04;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.v_registers[0x04], 0x44);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_op8xy0()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        cpu.opcode = 0x8450;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x04;
        cpu.v_registers[0x05] = 0x05;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.v_registers[0x04], 0x05);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_op8xy1()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        cpu.opcode = 0x8451;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x04;
        cpu.v_registers[0x05] = 0x05;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.v_registers[0x04], 0x04 | 0x05);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_op8xy2()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        cpu.opcode = 0x8452;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x04;
        cpu.v_registers[0x05] = 0x05;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.v_registers[0x04], 0x04 & 0x05);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_op8xy3()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        cpu.opcode = 0x8453;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x04;
        cpu.v_registers[0x05] = 0x05;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.v_registers[0x04], 0x04 ^ 0x05);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_op8xy4()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        // ADD does not exceed 8 bit (255)
        cpu.opcode = 0x8454;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x04;
        cpu.v_registers[0x05] = 0x05;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.v_registers[0x04], 0x09);
        assert_eq!(cpu.v_registers[0x0F], 0);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);

        // ADD exceed 8 bit (255)
        cpu.opcode = 0x8454;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 254;
        cpu.v_registers[0x05] = 3;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.v_registers[0x04], 1);
        assert_eq!(cpu.v_registers[0x0F], 1);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_op8xy5()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        // Vx > Vy
        cpu.opcode = 0x8455;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x04;
        cpu.v_registers[0x05] = 0x01;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.v_registers[0x04], 0x03);
        assert_eq!(cpu.v_registers[0x0F], 1);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);

        // Vx < Vy
        cpu.opcode = 0x8455;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x04;
        cpu.v_registers[0x05] = 0x05;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.v_registers[0x04], 0xFF);
        assert_eq!(cpu.v_registers[0x0F], 0);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_op8xy6()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        // Least significant bit = 1
        cpu.opcode = 0x8456;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x05;
        cpu.v_registers[0x05] = 0x05;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.v_registers[0x04], 0x02);
        assert_eq!(cpu.v_registers[0x0F], 1);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);

        // Least significant bit = 0
        cpu.opcode = 0x8456;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x04;
        cpu.v_registers[0x05] = 0x05;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.v_registers[0x04], 0x02);
        assert_eq!(cpu.v_registers[0x0F], 0);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_op8xy7()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        // Vy > Vx
        cpu.opcode = 0x8457;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x04;
        cpu.v_registers[0x05] = 0x05;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.v_registers[0x04], 0x01);
        assert_eq!(cpu.v_registers[0x0F], 0);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);

        // Vy < Vx
        cpu.opcode = 0x8457;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x04;
        cpu.v_registers[0x05] = 0x03;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.v_registers[0x04], 0xFF);
        assert_eq!(cpu.v_registers[0x0F], 1);
    }

    #[test]
    fn test_op8xye()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        // Most significant bit = 1
        cpu.opcode = 0x845E;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x81;
        cpu.v_registers[0x05] = 0x05;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.v_registers[0x04], 0x02);
        assert_eq!(cpu.v_registers[0x0F], 1);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);

        // Most significant bit = 0
        cpu.opcode = 0x845E;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x01;
        cpu.v_registers[0x05] = 0x05;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.v_registers[0x04], 0x02);
        assert_eq!(cpu.v_registers[0x0F], 0);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_op9xy0()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        // Vx == Vy
        cpu.opcode = 0x9450;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x04;
        cpu.v_registers[0x05] = 0x04;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);

        // Vx != Vy
        cpu.opcode = 0x9450;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x04;
        cpu.v_registers[0x05] = 0x05;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE * 2);
    }

    #[test]
    fn test_opannn()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        cpu.opcode = 0xA456;

        cpu.pc = 0x200;
        cpu.i_register = 0;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.i_register, 0x456);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_opbnnn()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        cpu.opcode = 0xB512;

        cpu.pc = 0x200;
        cpu.v_registers[0] = 2;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.pc, 0x514);
    }

    #[test]
    fn test_opcxkk()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        // kk = 0
        cpu.opcode = 0xC400;

        cpu.pc = 0x200;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.v_registers[4], 0);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);

        // kk = 0F
        cpu.opcode = 0xC40F;

        cpu.pc = 0x200;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.v_registers[4] & 0xF0, 0);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);

        // kk = F0
        cpu.opcode = 0xC4F0;

        cpu.pc = 0x200;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.v_registers[4] & 0x0F, 0);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_opdxyn()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        cpu.opcode = 0xD455;

        // print on empty display
        cpu.pc = 0x200;
        cpu.i_register = 0x00;
        cpu.v_registers[4] = 4;
        cpu.v_registers[5] = 5;
        cpu.execute_opcode(&mut mem, &key);
        // first row of 0 sprite
        assert_eq!(mem.display[[4,5]], 1);
        assert_eq!(mem.display[[7,5]], 1);
        assert_eq!(mem.display[[8,5]], 0);
        assert_eq!(mem.display[[11,5]], 0);
        // last row of 0 sprite
        assert_eq!(mem.display[[4,9]], 1);
        assert_eq!(mem.display[[7,9]], 1);
        assert_eq!(mem.display[[8,9]], 0);
        assert_eq!(mem.display[[11,9]], 0);

        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);

        // print on filled display
        cpu.pc = 0x200;
        cpu.i_register = 0x00;
        cpu.v_registers[4] = 6;
        cpu.v_registers[5] = 9;
        cpu.execute_opcode(&mut mem, &key);
        // first row of first 0 sprite
        assert_eq!(mem.display[[4,5]], 1);
        assert_eq!(mem.display[[7,5]], 1);
        assert_eq!(mem.display[[8,5]], 0);
        assert_eq!(mem.display[[11,5]], 0);
        // last row of first 0 sprite
        assert_eq!(mem.display[[4,9]], 1);
        assert_eq!(mem.display[[7,9]], 0);
        assert_eq!(mem.display[[8,9]], 1);
        assert_eq!(mem.display[[11,9]], 0);
        // first row of second 0 sprite
        assert_eq!(mem.display[[6,9]], 0);
        assert_eq!(mem.display[[9,9]], 1);
        assert_eq!(mem.display[[10,9]], 0);
        assert_eq!(mem.display[[13,9]], 0);
        // last row of second 0 sprite
        assert_eq!(mem.display[[6,13]], 1);
        assert_eq!(mem.display[[9,13]], 1);
        assert_eq!(mem.display[[10,13]], 0);
        assert_eq!(mem.display[[13,13]], 0);

        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_opex9e()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let mut key = Keyboard::new();
        cpu.opcode = 0xE49E;

        // key 4 is pressed
        cpu.pc = 0x200;
        cpu.v_registers[4] = 4;
        key[4] = 1;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE * 2);

        // key 4 is still
        cpu.pc = 0x200;
        cpu.v_registers[4] = 4;
        key[4] = 0;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_opexa1()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let mut key = Keyboard::new();
        cpu.opcode = 0xE4A1;

        // key 4 is pressed
        cpu.pc = 0x200;
        cpu.v_registers[4] = 4;
        key[4] = 1;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);

        // key 4 is still
        cpu.pc = 0x200;
        cpu.v_registers[4] = 4;
        key[4] = 0;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE * 2);
    }

    #[test]
    fn test_opfx07()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        cpu.opcode = 0xF407;

        cpu.pc = 0x200;
        cpu.delay_timer_register = 4;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.v_registers[4], 4);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_opfx0a()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        cpu.opcode = 0xF40A;

        cpu.pc = 0x200;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.input_register, 4);
        assert_eq!(cpu.waiting_for_input, true);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_opfx15()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        cpu.opcode = 0xF415;

        cpu.pc = 0x200;
        cpu.v_registers[4] = 4;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.delay_timer_register, 4);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_opfx18()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        cpu.opcode = 0xF418;

        cpu.pc = 0x200;
        cpu.v_registers[4] = 4;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.sound_timer_register, 4);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_opfx1e()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        cpu.opcode = 0xF41E;

        cpu.pc = 0x200;
        cpu.v_registers[4] = 4;
        cpu.i_register = 2;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.i_register, 6);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_opfx29()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        cpu.opcode = 0xF429;

        // Vx = 0
        cpu.pc = 0x200;
        cpu.v_registers[4] = 0;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.i_register, 0);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
        // Vx = 1
        cpu.pc = 0x200;
        cpu.v_registers[4] = 1;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.i_register, 5);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
        // Vx = 4
        cpu.pc = 0x200;
        cpu.v_registers[4] = 4;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.i_register, 20);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_opfx33()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        cpu.opcode = 0xF433;

        cpu.pc = 0x200;
        cpu.v_registers[4] = 249;
        cpu.i_register = 0x660;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(mem[0x660], 2);
        assert_eq!(mem[0x661], 4);
        assert_eq!(mem[0x662], 9);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_opfx55()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        cpu.opcode = 0xF455;

        cpu.pc = 0x200;
        cpu.v_registers[0] = 0;
        cpu.v_registers[1] = 1;
        cpu.v_registers[2] = 2;
        cpu.v_registers[3] = 33;
        cpu.v_registers[4] = 244;
        cpu.i_register = 0x660;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(mem[0x660], 0);
        assert_eq!(mem[0x661], 1);
        assert_eq!(mem[0x662], 2);
        assert_eq!(mem[0x663], 33);
        assert_eq!(mem[0x664], 244);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_opfx65()
    {
        let mut cpu = Cpu::new();
        let mut mem = Memory::new();
        let key = Keyboard::new();
        cpu.opcode = 0xF465;

        cpu.pc = 0x200;
        mem[0x660] = 0;
        mem[0x661] = 1;
        mem[0x662] = 2;
        mem[0x663] = 33;
        mem[0x664] = 244;
        cpu.i_register = 0x660;
        cpu.execute_opcode(&mut mem, &key);
        assert_eq!(cpu.v_registers[0], 0);
        assert_eq!(cpu.v_registers[1], 1);
        assert_eq!(cpu.v_registers[2], 2);
        assert_eq!(cpu.v_registers[3], 33);
        assert_eq!(cpu.v_registers[4], 244);
        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }
}
