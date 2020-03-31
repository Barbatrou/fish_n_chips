//!
//! CPU emulator
//!

use crate::hardware::memory::Memory;

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
            stack: [0; 16],
            stack_pointer: 0,
        }
    }

    pub fn push(&mut self, address: u16)
    {
        self.stack[self.stack_pointer] = address;
        self.stack_pointer += 1;
        if self.stack_pointer >= 16 {
            panic!("ERROR: cpu stack overflow, too many nested subroutines: {:#?}", self);
        }
    }

    pub fn top(&self) -> u16 { self.stack[self.stack_pointer] }

    pub fn pop(&mut self) -> u16
    {
        self.stack_pointer -= 1;
        if self.stack_pointer < 0 {
            panic!("ERROR: cpu stack underflow: {:#?}", self);
        }
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
        }
    }

    pub fn fetch_opcode(&mut self, memory: &Memory) -> u16
    {
        self.opcode = (memory[self.pc] as u16) << 8 | memory[self.pc + 1] as u16;
        self.opcode
    }

    pub fn execute_opcode(&mut self, memory: &Memory)
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
            (0x00, 0x00, 0x0e, 0x00) => self.op_00e0(),
            (0x00, 0x00, 0x0e, 0x0e) => self.op_00ee(),
            (0x01, _, _, _) => self.op_1nnn(nnn),
            (0x02, _, _, _) => self.op_2nnn(nnn),
            (0x03, _, _, _) => self.op_3xkk(x, kk),
            (0x04, _, _, _) => self.op_4xkk(x, kk),
            (0x05, _, _, 0x00) => self.op_5xy0(x, y),
            _ => ProgramCounter::NEXT,
        };
        match program_counter_next_operation {
            ProgramCounter::NEXT => self.pc += OPCODE_SIZE,
            ProgramCounter::SKIP => self.pc += OPCODE_SIZE * 2,
            ProgramCounter::JUMP(address) => self.pc = address as usize,
        }
    }

    pub fn do_cycle(&mut self, memory: &Memory)
    {
        // execute new instruction
        self.fetch_opcode(memory);
        self.execute_opcode(memory);


        // decrement timers
        if self.delay_timer_register > 0 {
            self.delay_timer_register -= 1;
        }
        if self.sound_timer_register > 0 {
            self.sound_timer_register -=1;
            // TODO make a beep
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

    fn op_00e0(&self) -> ProgramCounter // CLS - clear the display
    {
       // TODO clear display
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
        self.v_registers[x] = self.v_registers[x] + kk;
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
        self.v_registers[0x0f] = if self.v_registers[x] > self.v_registers[y] { 1 } else { 0 };
        self.v_registers[x] = self.v_registers[x].wrapping_sub(self.v_registers[y]);
        ProgramCounter::NEXT
    }
/*
    fn op_8xy6(&mut self) -> ProgramCounter // SHR Vx {, Vy} - Set Vx = Vx SHR 1.
    // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
    {

    }

    fn op_8xy7(&mut self) -> ProgramCounter // SUBN Vx, Vy - Set Vx = Vy - Vx, set VF = NOT borrow.
    // If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results stored in Vx.
    {

    }

    fn op_8xyE(&mut self) -> ProgramCounter // SHL Vx {, Vy} - Set Vx = Vx SHL 1.
    // If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
    {

    }

    fn op_9xy0(&mut self) -> ProgramCounter // SNE Vx, Vy - Skip next instruction if Vx != Vy.
    // The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2.
    {

    }

    fn op_Annn(&mut self) -> ProgramCounter // LD I, addr - Set I = nnn.
    // The value of register I is set to nnn.
    {

    }

    fn op_Bnnn(&mut self) -> ProgramCounter // JP V0, addr - Jump to location nnn + V0.
    // The program counter is set to nnn plus the value of V0.
    {

    }

    fn op_Cxkk(&mut self) -> ProgramCounter // RND Vx, byte - Set Vx = random byte AND kk.
    // The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk. The results are stored in Vx. See instruction 8xy2 for more information on AND.
    {

    }

    fn op_Dxyn(&mut self) -> ProgramCounter // DRW Vx, Vy, nibble - Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
// The interpreter reads n bytes from memory, starting at the address stored in I. These bytes are then displayed as sprites on screen at coordinates (Vx, Vy). Sprites are XORed onto the existing screen. If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0. If the sprite is positioned so part of it is outside the coordinates of the display, it wraps around to the opposite side of the screen. See instruction 8xy3 for more information on XOR, and section 2.4, Display, for more information on the Chip-8 screen and sprites.
    {

    }

    fn op_Ex9E(&mut self) -> ProgramCounter // SKP Vx - Skip next instruction if key with the value of Vx is pressed.
    // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down position, PC is increased by 2.
    {

    }

    fn op_ExA1(&mut self) -> ProgramCounter // SKNP Vx - Skip next instruction if key with the value of Vx is not pressed.
    // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up position, PC is increased by 2.
    {

    }

    fn op_Fx07(&mut self) -> ProgramCounter // LD Vx, DT - Set Vx = delay timer value.
    // The value of DT is placed into Vx.
    {

    }

    fn op_Fx0A(&mut self) -> ProgramCounter // LD Vx, K - Wait for a key press, store the value of the key in Vx.
    // All execution stops until a key is pressed, then the value of that key is stored in Vx.
    {

    }

    fn op_Fx15(&mut self) -> ProgramCounter // LD DT, Vx - Set delay timer = Vx.
    // DT is set equal to the value of Vx.
    {

    }

    fn op_Fx18(&mut self) -> ProgramCounter // LD ST, Vx - Set sound timer = Vx.
    // ST is set equal to the value of Vx.
    {

    }

    fn op_Fx1E(&mut self) -> ProgramCounter // ADD I, Vx - Set I = I + Vx.
    // The values of I and Vx are added, and the results are stored in I.
    {

    }

    fn op_Fx29(&mut self) -> ProgramCounter // LD F, Vx - Set I = location of sprite for digit Vx.
// The value of I is set to the location for the hexadecimal sprite corresponding to the value of Vx. See section 2.4, Display, for more information on the Chip-8 hexadecimal font.
    {

    }

    fn op_Fx33(&mut self) -> ProgramCounter // LD B, Vx - Store BCD representation of Vx in memory locations I, I+1, and I+2.
    // The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.
    {

    }

    fn op_Fx55(&mut self) -> ProgramCounter // LD [I], Vx - Store registers V0 through Vx in memory starting at location I.
    //The interpreter copies the values of registers V0 through Vx into memory, starting at the address in I.
    {

    }

    fn op_Fx65(&mut self) -> ProgramCounter // LD Vx, [I] - Read registers V0 through Vx from memory starting at location I.
    // The interpreter reads values from memory starting at location I into registers V0 through Vx.
    {

    }
*/

}

#[cfg(test)]
mod tests{
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
        let mem = Memory::new();
        cpu.pc = 0x200;
        cpu.opcode = 0x00e0;

        cpu.execute_opcode(&mem);

        // TODO check that the display has indeed been cleaned

        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE)
    }

    #[test]
    fn test_op00ee()
    {
        let mut cpu = Cpu::new();
        let mem = Memory::new();
        cpu.opcode = 0x00ee;

        cpu.stack.stack_pointer = 5;
        cpu.stack.stack[4] = 0x4444;
        cpu.execute_opcode(&mem);

        assert_eq!(cpu.stack.stack_pointer, 4);
        assert_eq!(cpu.pc, 0x4444);
    }

    #[test]
    fn test_op1nnn()
    {
        let mut cpu = Cpu::new();
        let mem = Memory::new();
        cpu.opcode = 0x1300;

        cpu.pc = 0x200;
        cpu.execute_opcode(&mem);

        assert_eq!(cpu.pc, 0x300);
    }

    #[test]
    fn test_op2nnn()
    {
        let mut cpu = Cpu::new();
        let mem = Memory::new();
        cpu.opcode = 0x2300;

        cpu.pc = 0x200;
        cpu.stack.stack_pointer = 2;
        cpu.stack.stack[2] = 0x4444;
        cpu.execute_opcode(&mem);

        assert_eq!(cpu.stack.stack_pointer, 3);
        assert_eq!(cpu.stack.stack[2], 0x200 + OPCODE_SIZE as u16);
        assert_eq!(cpu.pc, 0x300);
    }

    #[test]
    fn test_op3xkk()
    {
        let mut cpu = Cpu::new();
        let mem = Memory::new();
        // With satisfied predicate
        cpu.opcode = 0x3469;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x69;
        cpu.execute_opcode(&mem);

        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE * 2);

        // With unsatisfied predicate
        cpu.opcode = 0x3469;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x49;
        cpu.execute_opcode(&mem);

        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_op4xkk()
    {
        let mut cpu = Cpu::new();
        let mem = Memory::new();
        // With satisfied predicate
        cpu.opcode = 0x4469;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x49;
        cpu.execute_opcode(&mem);

        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE * 2);

        // With unsatisfied predicate
        cpu.opcode = 0x4469;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x69;
        cpu.execute_opcode(&mem);

        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

    #[test]
    fn test_op5xy0()
    {
        let mut cpu = Cpu::new();
        let mem = Memory::new();
        // With satisfied predicate
        cpu.opcode = 0x5440;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x04;
        cpu.execute_opcode(&mem);

        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE * 2);

        // With unsatisfied predicate
        cpu.opcode = 0x5460;

        cpu.pc = 0x200;
        cpu.v_registers[0x04] = 0x04;
        cpu.v_registers[0x06] = 0x06;
        cpu.execute_opcode(&mem);

        assert_eq!(cpu.pc, 0x200 + OPCODE_SIZE);
    }

}
