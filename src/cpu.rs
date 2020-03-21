use std::io;
use std::io::Read;
use std::ptr;

use rand::Rng;

pub const FONTSET: [u8; 80] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

// this probably results in a stack overflow on some systems
// TODO use the heap instead
pub struct CPU {
    opcode: u16,         // current opcode
    registers: [u8; 16], // 8 bit registers
    i: u16,              // address register
    sound_timer: u8,
    delay_timer: u8,
    stack: [u16; 16],
    sp: u16, // stack pointer
    pub memory: [u8; 4096],
    pc: u16, // program counter
    gfx: [u8; 64 * 32],
    draw_flag: bool,
    keyboard: [bool; 0xF],
    rng: rand::prelude::ThreadRng,
}

impl CPU {
    pub fn init() -> Self {
        let mut mem: [u8; 4096] = [0; 4096];

        // yes
        unsafe { ptr::copy_nonoverlapping(FONTSET.as_ptr(), mem.as_mut_ptr(), FONTSET.len()) };

        CPU {
            opcode: 0,
            registers: [0; 16],
            i: 0x200,
            sound_timer: 0,
            delay_timer: 0,
            stack: [0; 16],
            sp: 0,
            memory: mem,
            pc: 0x200,
            gfx: [0; 64 * 32],
            draw_flag: false,
            keyboard: [false; 0xF],
            rng: rand::thread_rng(),
        }
    }

    fn clear_screen(&self) {
        todo!()
    }

    #[inline]
    pub fn load_rom<R: Read>(&mut self, reader: &mut R) -> io::Result<usize> {
        reader.read(&mut self.memory[0x200..])
    }

    // separated those to make testing easier
    #[inline(always)]
    pub fn cycle(&mut self) {
        self.fetch();
        self.decode();
    }

    #[inline(always)]
    fn fetch(&mut self) {
        self.opcode =
            (self.memory[self.pc as usize] << 8 | self.memory[(self.pc + 1) as usize]).into();
    }

    pub fn decode(&mut self) {
        match self.opcode & 0xF000 {
            0x0000 => match self.opcode & 0x00FF {
                0x00E0 => self.clear_screen(),
                //return from subroutine
                0x00EE => {
                    self.sp -= 1;
                    self.pc = self.stack[self.sp as usize];
                    self.pc += 2;
                }
                _ => panic!("Unknown instruction {:#x}", self.opcode),
            },

            // goto
            0x1000 => {
                self.pc = self.opcode & 0x0FFF;
            }

            //call subroutine
            0x2000 => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = self.opcode & 0x0FFF;
            }

            // skip if true
            0x3000 => {
                if self.registers[((self.opcode & 0x0F00) >> 8) as usize]
                    == (self.opcode & 0x00FF) as u8
                {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }

            // skip if false
            0x4000 => {
                if self.registers[((self.opcode & 0x0F00) >> 8) as usize]
                    != (self.opcode & 0x00FF) as u8
                {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }

            // skip if true, comparing registers
            0x5000 => {
                if self.registers[((self.opcode & 0x0F00) >> 8) as usize]
                    == self.registers[((self.opcode & 0x00F0) >> 4) as usize]
                {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }

            // sets the value of a register
            0x6000 => {
                self.registers[((self.opcode & 0x0F00) >> 8) as usize] =
                    (self.opcode & 0x00FF) as u8;
                self.pc += 2;
            }

            // adds value to a register
            0x7000 => {
                let reg = ((self.opcode & 0x0F00) >> 8) as usize;
                let res = self.registers[reg].wrapping_add((self.opcode & 0x00FF) as u8);
                self.registers[reg] = res;
                self.pc += 2;
            }

            0x8000 => {
                match self.opcode & 0x000F {
                    // assign values from registers
                    0x0000 => {
                        self.registers[((self.opcode & 0x0F00) >> 8) as usize] =
                            self.registers[((self.opcode & 0x00F0) >> 4) as usize];
                        self.pc += 2;
                    }

                    // bitwise OR
                    0x0001 => {
                        self.registers[((self.opcode & 0x0F00) >> 8) as usize] |=
                            self.registers[((self.opcode & 0x00F0) >> 4) as usize];
                        self.pc += 2;
                    }

                    // bitwise AND
                    0x0002 => {
                        self.registers[((self.opcode & 0x0F00) >> 8) as usize] &=
                            self.registers[((self.opcode & 0x00F0) >> 4) as usize];
                        self.pc += 2;
                    }

                    // bitwise XOR
                    0x0003 => {
                        self.registers[((self.opcode & 0x0F00) >> 8) as usize] ^=
                            self.registers[((self.opcode & 0x00F0) >> 4) as usize];
                        self.pc += 2;
                    }

                    // plus equal
                    0x0004 => {
                        let regx = ((self.opcode & 0x0F00) >> 8) as usize;
                        let regy = ((self.opcode & 0x00F0) >> 4) as usize;

                        let (res, overflow) =
                            self.registers[regx].overflowing_add(self.registers[regy]);
                        self.registers[regx] = res;

                        self.registers[0xF] = if !overflow { 0 } else { 1 };

                        self.pc += 2;
                    }

                    // minus equal
                    0x0005 => {
                        let regx = ((self.opcode & 0x0F00) >> 8) as usize;
                        let regy = ((self.opcode & 0x00F0) >> 4) as usize;

                        let (res, overflow) =
                            self.registers[regx].overflowing_sub(self.registers[regy]);
                        self.registers[regx] = res;

                        self.registers[0xF] = if !overflow { 1 } else { 0 };

                        self.pc += 2;
                    }

                    // right shift
                    0x0006 => {
                        self.registers[0xF] =
                            self.registers[((self.opcode & 0x0F00) >> 8) as usize] & 0x1;
                        self.registers[((self.opcode & 0x0F00) >> 8) as usize] >>= 1;
                        self.pc += 2;
                    }

                    // minus assign, but the terms are reversed
                    0x0007 => {
                        let regx = ((self.opcode & 0x0F00) >> 8) as usize;
                        let regy = ((self.opcode & 0x00F0) >> 4) as usize;

                        let (res, overflow) =
                            self.registers[regy].overflowing_sub(self.registers[regx]);
                        self.registers[regx] = res;

                        self.registers[0xF] = if !overflow { 1 } else { 0 };

                        self.pc += 2;
                    }

                    // left shift
                    0x000E => {
                        self.registers[0xF] =
                            self.registers[((self.opcode & 0x0F00) >> 8) as usize] >> 7;
                        self.registers[((self.opcode & 0x0F00) >> 8) as usize] <<= 1;
                        self.pc += 2;
                    }

                    _ => panic!("Unkown instruction {:#x}", self.opcode),
                }
            }

            // skip if true, registers
            0x9000 => {
                if self.registers[((self.opcode & 0x0F00) >> 8) as usize]
                    != self.registers[((self.opcode & 0x00F0) >> 4) as usize]
                {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }

            // store value on the address pointer
            0xA000 => {
                self.i = self.opcode & 0x0FFF;
                self.pc += 2;
            }

            // jump to
            0xB000 => {
                self.pc = self.registers[0] as u16 + (self.opcode & 0x0FFF);
                self.pc += 2;
            }

            // rand
            0xC000 => {
                self.registers[((self.opcode & 0x0F00) >> 8) as usize] =
                    self.rng.gen::<u8>() & (self.opcode & 0x00FF) as u8;
            }

            // draw
            // the ugliest instruction here
            // TODO refactor this
            0xD000 => {
                let x = self.registers[((self.opcode & 0x0F00) >> 8) as usize];
                let y = self.registers[((self.opcode & 0x00F0) >> 4) as usize];
                let height = self.opcode & 0x000F;
                let mut pixel;

                self.registers[0xF] = 0;
                for yline in 0..height {
                    pixel = self.memory[(self.i + yline) as usize];
                    for xline in 0..8 {
                        if (pixel & (0x80 >> xline)) != 0
                            && self.gfx[((x + xline) as u16 + ((y as u16 + yline) * 64)) as usize]
                                == 1
                        {
                            self.registers[0xF] = 1;
                            self.gfx[((x + xline) as u16 + ((y as u16 + yline) * 64)) as usize] ^=
                                1;
                        }
                    }
                }
                self.draw_flag = true;
                self.pc += 2;
            }

            // keyboard
            0xE000 => todo!(),

            0xF000 => {
                match self.opcode & 0x00FF {
                    // store the delay in a register
                    0x0007 => {
                        self.registers[((self.opcode & 0x0F00) >> 8) as usize] = self.delay_timer;
                        self.pc += 2;
                    }

                    // get key
                    0x000A => todo!(),

                    // set delay timer
                    0x0015 => {
                        self.delay_timer = self.registers[((self.opcode & 0x0F00) >> 8) as usize];
                        self.pc += 2;
                    }

                    // set sound timer
                    0x0018 => {
                        self.sound_timer = self.registers[((self.opcode & 0x0F00) >> 8) as usize];
                        self.pc += 2;
                    }

                    // add assing to address register
                    0x001E => {
                        let res = self.i.wrapping_add(
                            self.registers[((self.opcode & 0x0F00) >> 8) as usize] as u16,
                        );
                        self.i = res;
                        self.pc += 2;
                    }

                    // store the sprite address in the adress register
                    0x0029 => {
                        self.i =
                            self.registers[((self.opcode & 0x0F00) >> 8) as usize] as u16 * 0x5;
                        self.pc += 2;
                    }

                    // store the binaryy coded decimal somewhere in memory
                    0x0033 => {
                        let reg_val = self.registers[((self.opcode & 0x0F00) >> 8) as usize];
                        let address = self.i as usize;

                        self.memory[address] = reg_val / 100;
                        self.memory[address + 1] = (reg_val / 10) % 10;
                        self.memory[address + 2] = reg_val % 10;
                        self.pc += 2;
                    }

                    // store data from registers 0 to register X into some memory area
                    0x0055 => {
                        let reg_x = ((self.opcode & 0x0F00) >> 8) as usize;
                        let index = self.i as usize;

                        // just a small guarantee
                        assert!(reg_x <= 0xF);

                        // yes, again
                        unsafe {
                            ptr::copy_nonoverlapping(
                                self.registers.as_ptr(),
                                self.memory[index..].as_mut_ptr(),
                                reg_x,
                            )
                        };

                        self.i += (reg_x + 1) as u16;
                        self.pc += 2;
                    }

                    // copy a region of memory into the registers
                    0x0065 => {
                        let reg_x = ((self.opcode & 0x0F00) >> 8) as usize;
                        let index = self.i as usize;

                        // just a small guarantee
                        assert!(reg_x <= 0xF);

                        // YES
                        unsafe {
                            ptr::copy_nonoverlapping(
                                self.memory[index..].as_ptr(),
                                self.registers.as_mut_ptr(),
                                reg_x,
                            )
                        };

                        self.i += (reg_x + 1) as u16;
                        self.pc += 2;
                    }

                    _ => panic!("Unknown instruction: {:#x}", self.opcode),
                }
            }
            _ => panic!("Unknown instruction: {:#x}", self.opcode),
        }
    }
}
