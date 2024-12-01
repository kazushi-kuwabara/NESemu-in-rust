use std::{collections::btree_map::{self, Values}, ops::Add};

pub struct CPU {
    pub register_a:u8,
    pub register_x:u8,
    pub register_y:u8,
    pub status:u8,
    pub program_counter:u16,
    memory: [u8;0xFFFF],
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Accumulator,
    Immediate,
    Zeropage,
    Zeropage_X,
    Zeropage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    NoneAddressinng,
}


impl CPU {

    fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {

        match mode {
            AddressingMode::Accumulator => self.register_a as u16 ,
            AddressingMode::Immediate => self.program_counter,
            AddressingMode::Zeropage => self.mem_read(self.program_counter) as u16,
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
            AddressingMode::Zeropage_Y =>{
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            }
            AddressingMode::Zeropage_X =>{
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }
            AddressingMode::Absolute_X => {
                let pos = self.mem_read_u16(self.program_counter);
                let addr = pos.wrapping_add(self.register_x as u16);
                addr
            }
            AddressingMode::Absolute_Y => {
                let pos = self.mem_read_u16(self.program_counter);
                let addr = pos.wrapping_add(self.register_y as u16);
                addr
            }

            AddressingMode::Indirect_X => {
                let pos = self.mem_read(self.program_counter);

                let base = (pos as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(base as u16);
                let hi = self.mem_read(base.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)

            }

            AddressingMode::Indirect_Y => {
                let pos = self.mem_read(self.program_counter);

                let lo = self.mem_read(pos as u16);
                let hi = self.mem_read((pos as u8).wrapping_add(1) as u16);
                let ptr = (hi as u16) << 8 | (lo as u16);
                let addr = ptr.wrapping_add(self.register_y as u16);
                addr
            }

            AddressingMode::NoneAddressinng => {
                panic!("mode {:?} is not supported" , mode);
            }


        }
    }


    fn mem_read(&self,addr:u16) ->u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self , addr:u16, data:u8) {
        self.memory[addr as usize] = data;
    }


    pub fn load_and_run(&mut self, program:Vec<u8>) {
        self.load(program);
        self.reset();
        self.run()
    }

    pub fn load(&mut self , program:Vec<u8>) {
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    fn mem_read_u16(&mut self , pos:u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;

        (hi << 8) | (lo as u16)
    }

    fn mem_write_u16(&mut self, pos:u16, data:u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;

        self.mem_write(pos, lo);
        self.mem_write(pos+1, hi);
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.status = 0;

        self.program_counter = self.mem_read_u16(0xFFFC);
    }


    pub fn new() -> Self {
        CPU {
            register_a:0,
            register_x:0,
            register_y:0,
            status:0,
            program_counter:0,
            memory:[0u8; 0xFFFF],
        }
    }

    fn sec(&mut self){
        self.status = self.status | 0b0000_0001;
    }
    
    fn clc(&mut self){
        self.status = self.status & 0b1111_1110;
    }

    fn lda(&mut self, mode:&AddressingMode){
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a =  value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn ldx(&mut self , mode:&AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_x = value;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn ldy(&mut self , mode: &AddressingMode){
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_y = value;
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn sta(&mut self, mode:&AddressingMode){
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    fn stx(&mut self , mode: &AddressingMode){
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_x);
    }

    fn sty(&mut self , mode: &AddressingMode){
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_y);
    }



    fn adc(&mut self, mode:&AddressingMode) {
        let addr = self.get_operand_address(mode);
        let pos = self.mem_read(addr);
        let tmp = self.register_a;
        let touple = (pos as u8).overflowing_add(self.status & 0x01);

        let base = touple.0;

        let touple2 =  self.register_a.overflowing_add(base as u8);

        self.register_a = touple2.0;

        /* bit operation starts from here */

        if self.status & 0b0000_0001 == 0b0000_0001 {
            self.status = self.status & 0b1111_1110;
        }

        self.update_zero_and_negative_flags(self.register_a);

        if touple.1 | touple2.1 {
            self.status = self.status | 0x01;
        } else {
            self.status = self.status & 0b1111_1110;
        }

        if ((self.register_a ^ tmp) & (self.register_a ^ pos) & 0x80) != 0 {
            self.status = self.status | 0b01000000;
        } else {
            self.status = self.status & 0b10111111;
        }
        /* bit operation endsuu from here */

        
    }

    fn sbc(&mut self , mode:&AddressingMode){
        let addr = self.get_operand_address(mode);
        let pos = self.mem_read(addr);
        let tmp = self.register_a;
        let touple = (!(pos) as u8).overflowing_add(self.status & 0x01);

        let base = touple.0;

        let touple2 =  self.register_a.overflowing_add(base as u8);

        self.register_a = touple2.0;

        /* bit operation starts from here */

        if self.status & 0b0000_0001 == 0b0000_0001 {
            self.status = self.status & 0b1111_1110;
        }

        self.update_zero_and_negative_flags(self.register_a);


        if !(self.status & 0b1000_0000 == 0b1000_0000) {
            self.status = self.status | 0b0000_0001;
        } else {
            self.status = self.status & 0b1111_1110;
        }

        if ((self.register_a ^ tmp) & (self.register_a ^ !(pos)) & 0x80) != 0 {
            self.status = self.status | 0b01000000;
        } else {
            self.status = self.status & 0b10111111;
        }

        /* bit operation endsuu from here */

        
    }

    fn inc(&mut self, mode: &AddressingMode){
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let new_value = (value as u8).overflowing_add(1).0;

        self.mem_write(addr, new_value);
        self.update_zero_and_negative_flags(new_value);
    }

    fn dec(&mut self, mode: &AddressingMode){
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let new_value = (value as u8).overflowing_sub(1).0;

        self.mem_write(addr, new_value);
        self.update_zero_and_negative_flags(new_value);
    }

    fn inx(&mut self) {
       self.register_x = (self.register_x).overflowing_add(1).0;
       self.update_zero_and_negative_flags(self.register_x);
    }

    fn dex(&mut self) {
       self.register_x = (self.register_x).overflowing_sub(1).0;
       self.update_zero_and_negative_flags(self.register_x);
    }

    fn iny(&mut self) {
       self.register_y = (self.register_y).overflowing_add(1).0;
       self.update_zero_and_negative_flags(self.register_y);
    }

    fn dey(&mut self) {
       self.register_y = (self.register_y).overflowing_sub(1).0;
       self.update_zero_and_negative_flags(self.register_y);
    }

    fn tax(&mut self){
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }
    
    fn txa(&mut self){
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn tay(&mut self){
        self.register_y = self.register_a;
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn tya(&mut self){
        self.register_a = self.register_y;
        self.update_zero_and_negative_flags(self.register_a);
    }

    /*shift instruction starts from here */

    fn asl(&mut self,mode: &AddressingMode){

        match mode {
            AddressingMode::Accumulator => {
                let value = self.register_a;
                let bit7_tmp = value & 0b1000_0000;
                let new_value = value * 2;

                self.register_a = new_value;

                self.status = self.status & 0b1111_1110;

                self.status = self.status | (bit7_tmp >> 7); //このやり方だと事前にキャリーがセットされているときにうまくいかない。
                self.update_zero_and_negative_flags(self.register_a);
            }
            _ => {
                let addr = self.get_operand_address(mode);
                let value = self.mem_read(addr);
                let bit7_tmp = value & 0b1000_0000;
                let new_value = value * 2;

                self.mem_write(addr, new_value);

                self.status = self.status & 0b1111_1110;

                self.status = self.status | (bit7_tmp >> 7);
                self.update_zero_and_negative_flags(new_value);
            }
        };
      
    }

    fn lsr(&mut self,mode: &AddressingMode){

        match mode {
            AddressingMode::Accumulator => {
                let value = self.register_a;
                let bit0_tmp = value & 0b0000_0001;
                let new_value = value / 2;

                self.register_a = new_value;

                self.status = self.status & 0b1111_1110;

                self.status = self.status | (bit0_tmp); 
                self.update_zero_and_negative_flags(self.register_a);
            }
            _ => {
                let addr = self.get_operand_address(mode);
                let value = self.mem_read(addr);
                let bit0_tmp = value & 0b0000_0001;
                let new_value = value / 2;

                self.mem_write(addr, new_value);

                self.status = self.status & 0b1111_1110;

                self.status = self.status | (bit0_tmp);
                self.update_zero_and_negative_flags(new_value);
            }
        };
      
    }

    /*shift instruction ends here */

    fn update_zero_and_negative_flags(&mut self, result:u8) {
        if result == 0 {
            self.status = self.status | 0b0000_0010;
        } else {
            self.status = self.status & 0b1111_1101;
        }

        if result & 0b1000_0000 != 0 {
            self.status = self.status | 0b1000_0000;
        } else {
            self.status  = self.status & 0b0111_1111;
        }
    }

    pub fn run(&mut self) { 

        loop {
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;
            match code {
                /* ----------SEC is stats here --------------- */
                0x38 => {
                    self.sec();
                }
                /* ----------SEC is ends here --------------- */

                /* ----------CLC is stats here --------------- */
                0x18 => {
                    self.clc();
                }
                /* ----------CLC is stats here --------------- */

                /* ----------LDA is stats here --------------- */
                0xA9 => {
                    self.lda(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }
                0xA5 => {
                    self.lda(&AddressingMode::Zeropage);
                    self.program_counter += 1;
                }
                0xB5 => {
                    self.lda(&AddressingMode::Zeropage_X);
                    self.program_counter += 1;
                }
                0xAD => {
                    self.lda(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                0xBD => {
                    self.lda(&AddressingMode::Absolute_X);
                    self.program_counter += 2;
                }
                0xB9 => {
                    self.lda(&AddressingMode::Absolute_Y);
                    self.program_counter += 2;
                }
                0xA1 => {
                    self.lda(&AddressingMode::Indirect_X);
                    self.program_counter += 1;
                }
                0xB1 => {
                    self.lda(&AddressingMode::Indirect_Y);
                    self.program_counter += 1;
                }
                /* --------- LDA is over -------------- */

                /* --------- LDX starts here -------------- */
                0xA2 => {
                    self.ldx(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }
                0xA6 => {
                    self.ldx(&AddressingMode::Zeropage);
                    self.program_counter += 1;
                }
                0xB6 => {
                    self.ldx(&AddressingMode::Zeropage_Y);
                    self.program_counter += 1;
                }
                0xAE => {
                    self.ldx(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                0xBE => {
                    self.ldx(&AddressingMode::Absolute_Y);
                    self.program_counter += 2;
                }
                /* --------- LDX ends over -------------- */

                /* --------- LDY starts here -------------- */
                0xA0 => {
                    self.ldy(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }
                0xA4 => {
                    self.ldy(&AddressingMode::Zeropage);
                    self.program_counter += 1;
                }
                0xB4 => {
                    self.ldy(&AddressingMode::Zeropage_X);
                    self.program_counter += 1;
                }
                0xAC => {
                    self.ldy(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                0xBC => {
                    self.ldy(&AddressingMode::Absolute_X);
                    self.program_counter += 2;
                }
                /* --------- LDY ends over -------------- */

                /* --------- BRK starts here -------------- */
                0x00 => {
                    return ;
                }
                /* --------- BRK is over -------------- */

                /* --------- STA start from here -------------- */

                0x85 => {
                    self.sta(&AddressingMode::Zeropage);
                    self.program_counter += 1
                }
                0x95 => {
                    self.sta(&AddressingMode::Zeropage_X);
                    self.program_counter += 1
                }
                0x8D => {
                    self.sta(&AddressingMode::Absolute);
                    self.program_counter += 2
                }
                0x9D => {
                    self.sta(&AddressingMode::Absolute_X);
                    self.program_counter += 2
                }
                0x99 => {
                    self.sta(&AddressingMode::Absolute_Y);
                    self.program_counter += 2
                }
                0x81 => {
                    self.sta(&AddressingMode::Indirect_X);
                    self.program_counter += 1
                }
                0x91 => {
                    self.sta(&AddressingMode::Indirect_Y);
                    self.program_counter += 1
                }
                /* --------- STA ends over -------------- */

                /* --------- STX starts from here -------------- */

                0x86 => {
                    self.stx(&AddressingMode::Zeropage);
                    self.program_counter += 1
                }
                0x96 => {
                    self.stx(&AddressingMode::Zeropage_Y);
                    self.program_counter += 1
                }
                0x8E => {
                    self.stx(&AddressingMode::Absolute);
                    self.program_counter += 2
                }
                
                /* --------- STX ends here -------------- */

                /* --------- STY starts from here -------------- */

                0x84 => {
                    self.sty(&AddressingMode::Zeropage);
                    self.program_counter += 1
                }
                0x94 => {
                    self.sty(&AddressingMode::Zeropage_Y);
                    self.program_counter += 1
                }
                0x8C => {
                    self.sty(&AddressingMode::Absolute);
                    self.program_counter += 2
                }
                
                /* --------- STY ends here -------------- */

                /* --------- ADC starts here -------------- */
                0x69 => {
                    self.adc(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }
                0x65=> {
                    self.adc(&AddressingMode::Zeropage);
                    self.program_counter += 1;
                }
                0x75=> {
                    self.adc(&AddressingMode::Zeropage_X);
                    self.program_counter += 1;
                }
                0x6D=> {
                    self.adc(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                0x7D=> {
                    self.adc(&AddressingMode::Absolute_X);
                    self.program_counter += 2;
                }
                0x79=> {
                    self.adc(&AddressingMode::Absolute_Y);
                    self.program_counter += 2;
                }
                0x61=> {
                    self.adc(&AddressingMode::Indirect_X);
                    self.program_counter += 1;
                }
                0x71=> {
                    self.adc(&AddressingMode::Indirect_Y);
                    self.program_counter += 1;
                }
                /* --------- ADC ends here -------------- */

                /* --------- SBC starts here -------------- */
                0xE9 => {
                    self.sbc(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }
                0xE5=> {
                    self.sbc(&AddressingMode::Zeropage);
                    self.program_counter += 1;
                }
                0xF5=> {
                    self.sbc(&AddressingMode::Zeropage_X);
                    self.program_counter += 1;
                }
                0xED=> {
                    self.sbc(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                0xFD=> {
                    self.sbc(&AddressingMode::Absolute_X);
                    self.program_counter += 2;
                }
                0xF9=> {
                    self.sbc(&AddressingMode::Absolute_Y);
                    self.program_counter += 2;
                }
                0xE1=> {
                    self.sbc(&AddressingMode::Indirect_X);
                    self.program_counter += 1;
                }
                0xF1=> {
                    self.sbc(&AddressingMode::Indirect_Y);
                    self.program_counter += 1;
                }
                /* --------- SBC ends here -------------- */

                /* --------- INC starts here -------------- */
                0xE6 => {
                    self.inc(&AddressingMode::Zeropage);
                    self.program_counter += 1;
                }
                0xF6 => {
                    self.inc(&AddressingMode::Zeropage_X);
                    self.program_counter += 1;
                }
                0xEE => {
                    self.inc(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                0xFE => {
                    self.inc(&AddressingMode::Absolute_X);
                    self.program_counter += 2;
                }
                /* --------- INC ends here -------------- */

                /* --------- DEC starts here -------------- */
                0xC6 => {
                    self.dec(&AddressingMode::Zeropage);
                    self.program_counter += 1;
                }
                0xD6 => {
                    self.dec(&AddressingMode::Zeropage_X);
                    self.program_counter += 1;
                }
                0xCE => {
                    self.dec(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                0xDE => {
                    self.dec(&AddressingMode::Absolute_X);
                    self.program_counter += 2;
                }
                /* --------- DEC ends here -------------- */

                /* --------- tax ends here -------------- */
                0xAA => {
                    self.tax();
                }
                /* --------- tax ends here -------------- */

                /* --------- txa ends here -------------- */
                0x8A => {
                    self.txa();
                }
                /* --------- txa ends here -------------- */

                /* --------- tay ends here -------------- */
                0xA8 => {
                    self.tay();
                }
                /* --------- tay ends here -------------- */

                /* --------- tya ends here -------------- */
                0x98 => {
                    self.tya();
                }
                /* --------- tya ends here -------------- */

                /* --------- inx ends here -------------- */
                0xE8 => {
                    self.inx();
                }
                /* --------- inx ends here -------------- */

                /* --------- dex ends here -------------- */
                0xCA => {
                    self.dex();
                }
                /* --------- dex ends here -------------- */

                /* --------- iny ends here -------------- */
                0xC8 => {
                    self.iny();
                }
                /* --------- iny ends here -------------- */

                /* --------- dey starts here -------------- */
                0x88 => {
                    self.dey();
                }
                /* --------- dey ends here -------------- */

                /* --------- asl starts here -------------- */
                0x0A => {
                    self.asl(&AddressingMode::Accumulator);
                }
                0x06 => {
                    self.asl(&AddressingMode::Zeropage);
                    self.program_counter += 1;
                }
                0x16 => {
                    self.asl(&AddressingMode::Zeropage_X);
                    self.program_counter += 1;
                }
                0x0E => {
                    self.asl(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                0x1E => {
                    self.asl(&AddressingMode::Absolute_X);
                    self.program_counter += 2;
                }
                /* --------- asl ends here -------------- */

                /* --------- lsr starts here -------------- */
                0x4A => {
                    self.lsr(&AddressingMode::Accumulator);
                }
                0x46 => {
                    self.lsr(&AddressingMode::Zeropage);
                    self.program_counter += 1;
                }
                0x56 => {
                    self.lsr(&AddressingMode::Zeropage_X);
                    self.program_counter += 1;
                }
                0x4E => {
                    self.lsr(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                0x5E => {
                    self.lsr(&AddressingMode::Absolute_X);
                    self.program_counter += 2;
                }
                /* --------- lsr ends here -------------- */
                
                _ => {
                    todo!("")
                }
            }
        }
    }
}






fn main() {
    println!("Hello, world!");
}



#[cfg(test)]
mod test {
    use super::*;//superは親モジュールを指す。ここでの親モジュールとはファイル全体？

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9,0x05,0x00]);
        assert_eq!(cpu.register_a , 0x05);
        assert!(cpu.status & 0b0000_0010 == 0);
        assert!(cpu.status & 0b1000_0000 == 0); //assert!マクロは中身がTRUEなら問題ナシ
    }

    #[test]
    fn test_0xa5_lda_zeropage_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9,0x10,0x85,0x01,0xa5,0x01,0x00]);
        assert_eq!(cpu.register_a , 0x10);
        assert!(cpu.status & 0b0000_0010 == 0);
        assert!(cpu.status & 0b1000_0000 == 0); //assert!マクロは中身がTRUEなら問題ナシ
    }




    #[test]
    fn test_0x09_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9,0x00,0x00]);
        assert_eq!(cpu.register_a , 0x00);
        assert!(cpu.status & 0b0000_0010 == 0b10 );
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9,0x05,0xAA,0x00]);
        assert_eq!(cpu.register_x , 0x05);
    }

    #[test]
    fn test_5_ops_woriking_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9,0xc0,0xaa,0xe8,0x00]);
        assert_eq!(cpu.register_x, 0xc1);
    }

    #[test]
    fn inx_overflow_check() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9,0xff,0xaa,0xe8,0xe8,0x00]);
        println!(" register_x is {:?}" , cpu.register_x);
        assert_eq!(cpu.register_x, 0x01);
    }

    #[test]
    fn test_0x69_adc_immediate() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9,0x02,0x69,0x50,0x85,0x01,0x00]);
        println!(" accumulator is {:?}" , cpu.register_a as u8);
        assert_eq!(cpu.register_a, 0x52);
    }

    #[test]
    fn test_0x69_adc_immediate_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9,0x50,0x69,0x50,0x85,0x01,0x00]);
        println!(" accumulator is {:?}" , cpu.register_a as u8);
        assert_eq!(cpu.register_a, 0xa0);
        assert!(cpu.status  & 0b01000000  == 0b0100_0000);
    }

    #[test]
    fn test_0x69_adc_immediate_overflow_ver2() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9,0xd0,0x69,0x90,0x85,0x01,0x00]);
        println!(" accumulator is {:?}" , cpu.register_a as u8);
        assert_eq!(cpu.register_a, 0x60);
        assert!(cpu.status  & 0b01000000  == 0b0100_0000);
    }

    #[test]
    fn test_0x69_adc_immediate_overflow_ver3() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38,0xa9,0x50,0x69,0x50,0x00]);
        println!(" accumulator is {:?}" , cpu.register_a as u8);
        assert_eq!(cpu.register_a, 0xa1);
        println!(" status is {:b}" , cpu.status );
        assert!(cpu.status  & 0b11000001  == 0b1100_0000);

    }

    #[test]
    fn test_0x69_adc_immediate_carry() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38,0xa9,0x50,0x69,0xd0,0x00]);
        println!(" accumulator is {:?}" , cpu.register_a as u8);
        assert_eq!(cpu.register_a, 0x21);
        println!(" status is {:b}" , cpu.status );
        assert!(cpu.status  & 0b11000001  == 0b0000_0001);

    }

    #[test]
    fn test_0xe9_adc_immediate_notoverflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38,0xa9,0x50,0xe9,0x10,0x00]);
        println!(" accumulator is {:?}" , cpu.register_a as u8);
        assert_eq!(cpu.register_a, 0x40);
    }
    #[test]
    fn test_0xe9_adc_immediate_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38,0xa9,0x50,0xe9,0xb0,0x00]);
        println!(" accumulator is {:?}" , cpu.register_a as u8);
        assert_eq!(cpu.register_a, 0xa0);
        println!(" status is {:b}" , cpu.status as u8);
        assert!(cpu.status & 0b1100_0001 == 0b1100_0000)
    }

    #[test]
    fn test_0xe9_adc_immediate_overflow_ver4() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38,0xa9,0x50,0xe9,0xb0,0x00]);
        println!(" register_x is {:?}" , cpu.register_a as u8);
        assert_eq!(cpu.register_a, 0xa0);
        println!(" status is {:b}" , cpu.status as u8);
        assert!(cpu.status & 0b1100_0001 == 0b1100_0000)
    }

    #[test]
    fn test_0x8e_stx_absolute() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2,0x50,0x8e,0x00,0xff,0x00]);
        println!(" register_x is {:0x}" , cpu.register_x as u8);
        assert_eq!(cpu.register_x, 0x50);
        assert_eq!(cpu.mem_read(0xff00), 0x50);
    }

    #[test]
    fn test_0xa0_ldy_and_0x8c_sty_absolute() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa0,0x60,0x8c,0x00,0xff,0x00]);
        println!(" register_y is {:0x}" , cpu.register_y as u8);
        assert_eq!(cpu.register_y, 0x60);
        assert_eq!(cpu.mem_read(0xff00), 0x60)
    }

    #[test]
    fn test_0xaa_tax() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9,0x60,0xaa,0x00]);
        println!(" register_x is {:0x}" , cpu.register_x as u8);
        assert_eq!(cpu.register_x, 0x60);
    }

    #[test]
    fn test_0x8a_txa() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2,0x70,0x8a,0x00]);
        println!(" register_a is {:0x}" , cpu.register_a as u8);
        assert_eq!(cpu.register_a, 0x70);
    }

    #[test]
    fn test_0xa8_tay() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9,0x80,0xa8,0x00]);
        println!(" register_y is {:0x}" , cpu.register_y as u8);
        assert_eq!(cpu.register_y, 0x80);
    }

    #[test]
    fn test_0xa0_ldy_0x98_tya() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa0,0x90,0x98,0x00]);
        println!(" register_a is {:0x}" , cpu.register_a as u8);
        assert_eq!(cpu.register_a, 0x90);
    }

    #[test]
    fn test_0xee_inc_absolute() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2,0xff,0x8e,0x00,0x10,0xee,0x00,0x10]);
        println!(" register_x is {:0x}" , cpu.register_x as u8);
        println!(" memory[0x1000] is {:0x}" , cpu.mem_read(0x1000) as u8);
        assert_eq!(cpu.mem_read(0x1000), 0x00);
        println!(" status  is {:0b}" , cpu.status);
        assert_eq!(cpu.status & 0b0000_0010, 0b0000_0010);
    }

    #[test]
    fn test_0xce_decc_absolute() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2,0xff,0x8e,0x00,0x10,0xce,0x00,0x10]);
        println!(" register_x is {:0x}" , cpu.register_x as u8);
        println!(" memory[0x1000] is {:0x}" , cpu.mem_read(0x1000) as u8);
        assert_eq!(cpu.mem_read(0x1000), 0xfe);
        println!(" status  is {:0b}" , cpu.status);
        assert_eq!(cpu.status & 0b1000_0010, 0b1000_0000);
    }

    #[test]
    fn test_0xce_dec_absolute_ver2() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2,0x00,0x8e,0x00,0x10,0xce,0x00,0x10]);
        println!(" register_x is {:0x}" , cpu.register_x as u8);
        println!(" memory[0x1000] is {:0x}" , cpu.mem_read(0x1000) as u8);
        assert_eq!(cpu.mem_read(0x1000), 0xff);
        println!(" status  is {:0b}" , cpu.status);
        assert_eq!(cpu.status & 0b1000_0010, 0b1000_0000);
    }

    #[test]
    fn test_0xe8_inx_0xca_dex() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2,0x00,0xe8,0xca,0x00]);
        println!(" register_x is {:0x}" , cpu.register_x as u8);
        assert_eq!(cpu.register_x,0x00);
        println!(" status  is {:0b}" , cpu.status);
        assert_eq!(cpu.status & 0b1000_0010, 0b0000_0010);
    }

    #[test]
    fn test_0xc8_iny_0x88_dey() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa0,0x00,0xc8,0x88,0x00]);
        println!(" register_y is {:0x}" , cpu.register_y as u8);
        assert_eq!(cpu.register_y,0x00);
        println!(" status  is {:0b}" , cpu.status);
        assert_eq!(cpu.status & 0b1000_0010, 0b0000_0010);
    }

    #[test]
    fn test_0x0a_accumulator() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38,0xa9,0x2a,0x0a,0x00]);
        println!(" register_a is {:0x}" , cpu.register_a as u8);
        assert_eq!(cpu.register_a, 0x54);
        println!(" status  is {:0b}" , cpu.status);
        assert_eq!(cpu.status & 0b1000_0011, 0b0000_0000);
    }

    #[test]
    fn test_0x0e_asl_absolute() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38,0xa9,0x2a,0x8d,0x10,0x00,0x0e,0x10,0x00,0x00]);
        println!(" memory[0x0010] is {:0x}" , cpu.mem_read(0x0010) as u8);
        assert_eq!(cpu.mem_read(0x0010), 0x54);
        println!(" status  is {:0b}" , cpu.status);
        assert_eq!(cpu.status & 0b1000_0011, 0b0000_0000);
    }

    #[test]
    fn test_0x4e_lsr_accumulator() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38,0xa9,0x2a,0x8d,0x10,0x00,0x4e,0x10,0x00,0x00]);
        println!(" memory[0x0010] is {:0x}" , cpu.mem_read(0x0010) as u8);
        assert_eq!(cpu.mem_read(0x0010), 0x15);
        println!(" status  is {:0b}" , cpu.status);
        assert_eq!(cpu.status & 0b1000_0011, 0b0000_0000);
    }

    
}