pub mod cpu;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cpu_build_font_test() {
        let cpu = cpu::CPU::init();

        assert_eq!(cpu.memory[0..80], cpu::FONTSET[..]);
    }

    #[test]
    fn rom_load_test() {
        use std::fs::File;

        let mut cpu = cpu::CPU::init();
        let mut file = File::open("roms/PONG").unwrap();
        let raw_rom = include_bytes!("../roms/PONG");

        cpu.load_rom(&mut file).unwrap();

        assert_eq!(raw_rom[..], cpu.memory[0x200..(0x200 + raw_rom.len())]);
    }
}
