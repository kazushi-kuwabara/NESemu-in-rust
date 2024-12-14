use core::borrow;
use std::{collections::btree_map::{self, Values}, fs::read_link, ops::Add, path::is_separator, result};

pub const CARRY_FLAG:u8 = 0b0000_0001;
pub const INTERRUPT_FLAG:u8 = 0b0000_0100;
pub const DECIMAL_FLAG:u8 = 0b0000_1000;
pub const BREAK_FLAG:u8 = 0b0001_0000;
pub const INVALID_FLAG:u8 = 0b0010_0000;
pub const NEGATIVE_FLAG:u8 = 0b1000_0000;
pub const ZERO_FLAG:u8 = 0b0000_0010;
pub const OVERFOW_FLAG:u8 = 0b0100_0000;

pub fn is_flag_set(flag:u8,x:u8) -> bool {
      x & flag > 0
}

pub struct CPU {
    pub register_a:u8,
    pub register_x:u8,
    pub register_y:u8,
    pub status:u8,
    pub program_counter:u16,
    pub stackpointer:u8,
    memory: [u8;0xFFFF],
}

#[derive(Debug, PartialEq)]
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
    Indirect,
    Indirect_X,
    Indirect_Y,
    Relative,
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

            AddressingMode::Indirect => {
                let addr = self.mem_read_u16(self.program_counter);
                let value = self.mem_read_u16(addr);
                value
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

            AddressingMode::Relative => {
                let addr = self.mem_read(self.program_counter);
                let tmp = (addr as i8) as i32 + self.program_counter as i32;
                tmp as u16
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
        self.register_y = 0;
        self.stackpointer = 0xff;
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
            stackpointer:0xff,
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
                let new_value = value.wrapping_mul(2);

                self.register_a = new_value;

                self.status = self.status & 0b1111_1110;

                self.status = self.status | (bit7_tmp >> 7); //このやり方だと事前にキャリーがセットされているときにうまくいかない。
                self.update_zero_and_negative_flags(self.register_a);
            }
            _ => {
                let addr = self.get_operand_address(mode);
                let value = self.mem_read(addr);
                let bit7_tmp = value & 0b1000_0000;
                let new_value = value.wrapping_mul(2);

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
                let new_value = value.wrapping_div(2);

                self.register_a = new_value;

                self.status = self.status & 0b1111_1110;

                self.status = self.status | (bit0_tmp); 
                self.update_zero_and_negative_flags(self.register_a);
            }
            _ => {
                let addr = self.get_operand_address(mode);
                let value = self.mem_read(addr);
                let bit0_tmp = value & 0b0000_0001;
                let new_value = value.wrapping_div(2);

                self.mem_write(addr, new_value);

                self.status = self.status & 0b1111_1110;

                self.status = self.status | (bit0_tmp);
                self.update_zero_and_negative_flags(new_value);
            }
        };
      
    }

    fn rol(&mut self,mode: &AddressingMode){

        match mode {
            AddressingMode::Accumulator => {
                let value = self.register_a;
                let bit7_tmp = value & 0b1000_0000;
                let carry_tmp = self.status & 0b0000_0001;
                let tmp_value = value.wrapping_mul(2);

                let tmp_value_without_carry = tmp_value & 0b1111_1110;

                let modified_value = tmp_value_without_carry | carry_tmp;

                self.register_a = modified_value;

                self.status = self.status & 0b1111_1110;

                self.status = self.status | (bit7_tmp >> 7); 
                self.update_zero_and_negative_flags(self.register_a);
            }
            _ => {
                let addr = self.get_operand_address(mode);
                let value = self.mem_read(addr);
                let bit7_tmp = value & 0b1000_0000;
                let carry_tmp = self.status & 0b0000_0001;
                let tmp_value = value.wrapping_mul(2);

                let tmp_value_without_carry = tmp_value & 0b1111_1110;

                let modified_value = tmp_value_without_carry | carry_tmp;

                self.mem_write(addr, modified_value);

                self.status = self.status & 0b1111_1110;

                self.status = self.status | (bit7_tmp >> 7);
                self.update_zero_and_negative_flags(modified_value);
            }
        };
      
    }

    fn ror(&mut self , mode:&AddressingMode){

        let (value,borrow) = match mode {
            AddressingMode::Accumulator => {

                let mut  value = self.register_a;
                let borrow = value % 2;
                value = value.wrapping_div(2);
                value = value | ((self.status & 0b0000_0001) << 7);
                self.register_a = value;
                (value , borrow)
            }
            _ => { 
                let addr = self.get_operand_address(mode);
                let mut  value = self.mem_read(addr);

                let borrow = value % 2;
                value = value.wrapping_div(2);
                value = value | ((self.status & 0b0000_0001) <<  7);
                self.mem_write(addr, value);
                (value , borrow)
            }
        };
        self.status = if borrow == 1 {
            self.status | 0b0000_0001
        } else {
            self.status & 0b1111_1110
        };
        self.update_zero_and_negative_flags(value);

    }

    /*shift instruction ends here */

    /*arithmetic instruction starts here */
    fn and(&mut self , mode:&AddressingMode){
        let addr = self.get_operand_address(mode);
        let  value = self.mem_read(addr);

        self.register_a = self.register_a & value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn ora(&mut self , mode:&AddressingMode){
        let addr = self.get_operand_address(mode);
        let  value = self.mem_read(addr);

        self.register_a = self.register_a | value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn eor(&mut self , mode:&AddressingMode){
        let addr = self.get_operand_address(mode);
        let  value = self.mem_read(addr);

        self.register_a = self.register_a ^ value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn bit(&mut self , mode:&AddressingMode){
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let bit6_tmp = value & 0b0100_0000;
        let bit7_tmp = value & 0b1000_0000;

        let result = self.register_a & value;

        self.status = if result == 0 {
            self.status | 0b0000_0010
        } else {
            self.status & 0b1111_1101
        };

        self.status = if bit6_tmp == 0b0100_0000 {
            self.status | ((bit6_tmp))
        } else {
            self.status & 0b1011_1111
        };

        self.status = if bit7_tmp == 0b1000_0000 {
            self.status |  ((bit7_tmp))
        } else {
            self.status & 0b0111_1111
        };


    }
    /*arithmetic instruction ends here */

    /*compare instruction starts here */
    fn cmp(&mut self, mode:&AddressingMode){
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let result = self.register_a - value;

        self.status = if result == 0 {
            self.status | 0b0000_0010
        } else {
            self.status & !(0b0000_0010)
        };

        self.status = if !(result & 0b1000_0000 == 0b1000_0000) {
            self.status | 0b0000_0001
        } else {
            self.status & !(0b0000_0001)
        };

        self.status = if result & 0b1000_0000 == 0b1000_0000 {
            self.status | 0b1000_0000
        } else {
            self.status & !(0b1000_0000)
        };
    }
    fn cpx(&mut self, mode:&AddressingMode){
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let result = self.register_x - value;

        self.status = if result == 0 {
            self.status | 0b0000_0010
        } else {
            self.status & !(0b0000_0010)
        };

        self.status = if !(result & 0b1000_0000 == 0b1000_0000) {
            self.status | 0b0000_0001
        } else {
            self.status & !(0b0000_0001)
        };

        self.status = if result & 0b1000_0000 == 0b1000_0000 {
            self.status | 0b1000_0000
        } else {
            self.status & !(0b1000_0000)
        };
    }
    fn cpy(&mut self, mode:&AddressingMode){
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let result = self.register_y - value;

        self.status = if result == 0 {
            self.status | 0b0000_0010
        } else {
            self.status & !(0b0000_0010)
        };

        self.status = if !(result & 0b1000_0000 == 0b1000_0000) {
            self.status | 0b0000_0001
        } else {
            self.status & !(0b0000_0001)
        };

        self.status = if result & 0b1000_0000 == 0b1000_0000 {
            self.status | 0b1000_0000
        } else {
            self.status & !(0b1000_0000)
        };
    }
    /*compare instruction ends here */

    /*branch instruction starts from here */
    fn branch(&mut self, mode: &AddressingMode) {
        if mode == &AddressingMode::Relative {
          let addr = self.get_operand_address(mode);
          self.program_counter = addr;
        }
    }

    fn bcc(&mut self , mode:&AddressingMode){
        if is_flag_set(CARRY_FLAG, self.status) == false {
            self.branch(mode);
        }
    }
    fn bcs(&mut self , mode:&AddressingMode){
        let addr = self.get_operand_address(mode);

        if is_flag_set(CARRY_FLAG, self.status) == true {
            self.program_counter = addr;
        }
    }
    fn beq(&mut self , mode:&AddressingMode){
        let addr = self.get_operand_address(mode);

        if is_flag_set(ZERO_FLAG, self.status) == true {
            self.program_counter = addr;
        }
    }
    fn bne(&mut self , mode:&AddressingMode){
        if is_flag_set(ZERO_FLAG, self.status) == false {
            self.branch(mode);
        }
    }
    fn bpl(&mut self , mode:&AddressingMode){
        if is_flag_set(NEGATIVE_FLAG, self.status) == false {
            self.branch(mode);
        }
    }
    fn bmi(&mut self , mode:&AddressingMode){
        if is_flag_set(NEGATIVE_FLAG, self.status) == true {
            self.branch(mode);
        }
    }
    fn bvc(&mut self , mode:&AddressingMode){
        if is_flag_set(OVERFOW_FLAG, self.status) == false {
            self.branch(mode);
        }
    }

    fn bvs(&mut self , mode:&AddressingMode){
        if is_flag_set(OVERFOW_FLAG, self.status) == true {
            self.branch(mode);
        }
    }
    /*branch instruction ends from here */

    /*jump instruction starts from here */
    fn push(&mut self,data:u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;

        self.mem_write(0x0100 + (self.stackpointer as u16), hi);
        self.mem_write(0x0100 + ((self.stackpointer -1) as u16), lo);
        self.stackpointer = self.stackpointer - 2;
    }
    //サブルーチン後はjsrの次の命令から開始する必要がある。
    fn push_pc(&mut self){
        self.push(self.program_counter + 2);
    }
    //ただスタックからpcをとり出すだけ。
    fn pop_pc(&mut self) -> u16 {
        self.stackpointer = self.stackpointer + 2;

        let hi = self.mem_read(0x0100 + (self.stackpointer as u16));
        let lo = self.mem_read(0x0100 + ((self.stackpointer - 1 ) as u16));


        let pc = (((hi as u16) << 8)) | (lo as u16) + 1;
        pc
    }

    fn pop_flag(&mut self) -> u8 {
        self.stackpointer += 1;
        let flag = self.mem_read(0x0100 + (self.stackpointer as u16));
        flag
    }

   

    fn jmp(&mut self , mode: &AddressingMode){
        let value = match mode {
            &AddressingMode::Absolute => {
                let value = self.mem_read_u16(self.program_counter);
                value
            }
            &AddressingMode::Indirect => {
                let value = self.get_operand_address(&AddressingMode::Indirect);
                value
            }
            _ => {
                panic!("mode {:?} is not supported" , mode);
            }
        };
            
        self.program_counter = value;
    }

    fn jsr(&mut self, mode:&AddressingMode){
        let _value = match mode  {
            &AddressingMode::Absolute => {
                let value = self.mem_read_u16(self.program_counter);
                value
            }

            _ => {
                panic!("mode {:?} is not supported" , mode);
            }
        };
  
        self.push_pc();
        self.program_counter = _value;
    }
    /*jump instruction ends from here */

    fn rts(&mut self) {
         self.program_counter = self.pop_pc() + 1;
    }

    fn rti(&mut self){
        //pop status flags
        self.status = self.pop_flag() & !BREAK_FLAG;
        //bit 5 is always 1
        self.status = self.status | INVALID_FLAG;

        self.program_counter = self.pop_pc();
    }

    fn brk(&mut self) {
      
      self.program_counter  = self.mem_read_u16(0xFFFE);
      self.status = self.status | BREAK_FLAG;

    }
    
    fn pha(&mut self){
        self.mem_write(0x0100 + (self.stackpointer as u16), self.register_a);
        self.stackpointer = self.stackpointer - 1;
    }

    fn pla(&mut self){
        self.stackpointer += 1;
        self.register_a = self.mem_read(0x0100 + (self.stackpointer as u16));
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn php(&mut self){
        self.mem_write(0x0100 + (self.stackpointer as u16), self.status | BREAK_FLAG | INVALID_FLAG );
        self.stackpointer = self.stackpointer - 1;
    }

    fn plp(&mut self){
        self.stackpointer += 1;
        self.status = self.mem_read(0x0100 + (self.stackpointer as u16)) ;
    }

    fn txs(&mut self){
        self.stackpointer = self.register_x;
    }

    fn tsx(&mut self){
        self.register_x = self.stackpointer;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn cli(&mut self){
        //CLI命令による割り込み禁止フラグの更新は1命令文遅れる。次の命令が行われるのと同タイミングでフラグを更新する。
        //self.program_counter += 1;
        self.status = self.status & !INTERRUPT_FLAG;

    }

    fn sei(&mut self){
        //CLI命令と同様に１命令文更新が遅れる。
        //self.program_counter += 1;
        self.status  = self.status | INTERRUPT_FLAG;
    }

    fn cld(&mut self){
        self.status = self.status & !DECIMAL_FLAG;
    }

    fn sed(&mut self){
        self.status = self.status | DECIMAL_FLAG;
    }

    fn clv(&mut self){
        self.status = self.status & !OVERFOW_FLAG;
    }





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

                /* --------- rol starts here -------------- */
                0x2A => {
                    self.rol(&AddressingMode::Accumulator);
                }
                0x26 => {
                    self.rol(&AddressingMode::Zeropage);
                    self.program_counter += 1;
                }
                0x36 => {
                    self.rol(&AddressingMode::Zeropage_X);
                    self.program_counter += 1;
                }
                0x2E => {
                    self.rol(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                0x3E => {
                    self.rol(&AddressingMode::Absolute_X);
                    self.program_counter += 2;
                }
                /* --------- rol ends here -------------- */

                /* --------- ror starts here -------------- */
                0x6A => {
                    self.ror(&AddressingMode::Accumulator);
                }
                0x66 => {
                    self.ror(&AddressingMode::Zeropage);
                    self.program_counter += 1;
                }
                0x76 => {
                    self.ror(&AddressingMode::Zeropage_X);
                    self.program_counter += 1;
                }
                0x6E => {
                    self.ror(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                0x7E => {
                    self.ror(&AddressingMode::Absolute_X);
                    self.program_counter += 2;
                }
                /* --------- ror ends here -------------- */

                /* --------- and starts here -------------- */
                0x29 => {
                    self.and(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }
                0x25 => {
                    self.and(&AddressingMode::Zeropage);
                    self.program_counter += 1;
                }
                
                0x35 => {
                    self.and(&AddressingMode::Zeropage_X);
                    self.program_counter += 1;
                }
                0x2D => {
                    self.and(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                
                0x3D => {
                    self.and(&AddressingMode::Absolute_X);
                    self.program_counter += 2;
                }
                0x39 => {
                    self.and(&AddressingMode::Absolute_Y);
                    self.program_counter += 2;
                }
                
                0x21 => {
                    self.and(&AddressingMode::Indirect_X);
                    self.program_counter += 1;
                }
                
                0x31 => {
                    self.and(&AddressingMode::Indirect_Y);
                    self.program_counter += 1;
                }
                    
                /* --------- and ends here -------------- */

                /* --------- ora starts here -------------- */
                0x09 => {
                    self.ora(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }
                0x05 => {
                    self.ora(&AddressingMode::Zeropage);
                    self.program_counter += 1;
                }
                
                0x15 => {
                    self.ora(&AddressingMode::Zeropage_X);
                    self.program_counter += 1;
                }
                0x0D => {
                    self.ora(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                
                0x1D => {
                    self.ora(&AddressingMode::Absolute_X);
                    self.program_counter += 2;
                }
                0x19 => {
                    self.ora(&AddressingMode::Absolute_Y);
                    self.program_counter += 2;
                }
                
                0x01 => {
                    self.ora(&AddressingMode::Indirect_X);
                    self.program_counter += 1;
                }
                
                0x11 => {
                    self.ora(&AddressingMode::Indirect_Y);
                    self.program_counter += 1;
                }
                    
                /* --------- ora ends here -------------- */

                /* --------- eor starts here -------------- */
                0x49 => {
                    self.eor(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }
                0x45 => {
                    self.eor(&AddressingMode::Zeropage);
                    self.program_counter += 1;
                }
                
                0x55 => {
                    self.eor(&AddressingMode::Zeropage_X);
                    self.program_counter += 1;
                }
                0x4D => {
                    self.eor(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                
                0x5D => {
                    self.eor(&AddressingMode::Absolute_X);
                    self.program_counter += 2;
                }
                0x59 => {
                    self.eor(&AddressingMode::Absolute_Y);
                    self.program_counter += 2;
                }
                
                0x41 => {
                    self.eor(&AddressingMode::Indirect_X);
                    self.program_counter += 1;
                }
                
                0x51 => {
                    self.eor(&AddressingMode::Indirect_Y);
                    self.program_counter += 1;
                }
                    
                /* --------- ora ends here -------------- */

                /* --------- bit starts here -------------- */
                0x24 => {
                    self.bit(&AddressingMode::Zeropage);
                    self.program_counter += 1;
                }
                0x2C => {
                    self.bit(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }

                /* --------- bit ends here -------------- */

                /* --------- cmp starts here -------------- */
                0xC9 => {
                    self.cmp(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }
                0xC5 => {
                    self.cmp(&AddressingMode::Zeropage);
                    self.program_counter += 1;
                }
                0xD5 => {
                    self.cmp(&AddressingMode::Zeropage_X);
                    self.program_counter += 1;
                }
                0xCD => {
                    self.cmp(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                0xDD => {
                    self.cmp(&AddressingMode::Absolute_X);
                    self.program_counter += 2;
                }
                0xD9 => {
                    self.cmp(&AddressingMode::Absolute_Y);
                    self.program_counter += 2;
                }
                0xC1 => {
                    self.cmp(&AddressingMode::Indirect_X);
                    self.program_counter += 1;
                }
                0xD1 => {
                    self.cmp(&AddressingMode::Indirect_Y);
                    self.program_counter += 1;
                }
                /* --------- cmp ends here -------------- */
                /* --------- cpx starts here -------------- */
                0xE0 => {
                    self.cpx(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }
                0xE4 => {
                    self.cpx(&AddressingMode::Zeropage);
                    self.program_counter += 1;
                }
                0xEC => {
                    self.cpx(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                /* --------- cpx ends here -------------- */
                /* --------- cpy starts here -------------- */
                0xC0 => {
                    self.cpy(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }
                0xC4 => {
                    self.cpy(&AddressingMode::Zeropage);
                    self.program_counter += 1;
                }
                0xCC => {
                    self.cpy(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                /* --------- cpy ends here -------------- */

                /* --------- bcc starts here -------------- */
                0x90 => {
                    self.bcc(&AddressingMode::Relative);
                    self.program_counter += 1;
                }
                /* --------- bcc ends here -------------- */

                /* --------- bcs starts here -------------- */
                0xB0 => {
                    self.bcs(&AddressingMode::Relative);
                    self.program_counter += 1;
                }
                /* --------- bcc ends here -------------- */
                /* --------- beq starts here -------------- */
                0xF0 => {
                    self.beq(&AddressingMode::Relative);
                    self.program_counter += 1;
                }
                /* --------- beq ends here -------------- */

                /* --------- bne starts here -------------- */
                0xD0 => {
                    self.bne(&AddressingMode::Relative);
                    self.program_counter += 1;
                }
                /* --------- bne ends here -------------- */

                /* --------- bpl starts here -------------- */
                0x10 => {
                    self.bpl(&AddressingMode::Relative);
                    self.program_counter += 1;
                }
                /* --------- bpl ends here -------------- */

                /* --------- bmi starts here -------------- */
                0x30 => {
                    self.bmi(&AddressingMode::Relative);
                    self.program_counter += 1;
                }
                /* --------- bmi ends here -------------- */

                /* --------- bvc starts here -------------- */
                0x50 => {
                    self.bvc(&AddressingMode::Relative);
                    self.program_counter += 1;
                }
                /* --------- bvc ends here -------------- */

                /* --------- bvs starts here -------------- */
                0x70 => {
                    self.bvs(&AddressingMode::Relative);
                    self.program_counter += 1;
                }
                /* --------- bvs ends here -------------- */

                /* --------- jmp starts here -------------- */
                0x4C => {
                    self.jmp(&AddressingMode::Absolute);
                    //self.program_counter += 2;
                }
                0x6C => {
                    self.jmp(&AddressingMode::Absolute);
                    //self.program_counter += 2;
                }
                /* --------- jmp ends here -------------- */

                0x00 => {
                    return;
                }


                0x48 => {
                    self.pha();
                }

                0x68 => {
                    self.pla();
                }

                0x08 => {
                    self.php();
                }

                0x28 => {
                    self.plp();
                }

                0x9A => {
                    self.txs();
                }

                0xBA => {
                    self.tsx();
                }

                0x58 => {
                    self.cli();
                }

                0x78 => {
                    self.sei();
                }

                0xD8 => {
                    self.cld();
                }

                0xF8 => {
                    self.sed();
                }
                
                0xB8 => {
                    self.clv();
                }

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
    fn test_0x0a_asl_accumulator_carry() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x18,0xa9,0x80,0x0a,0x00]);
        println!(" register_a is {:0x}" , cpu.register_a as u8);
        assert_eq!(cpu.register_a, 0x00);
        println!(" status  is {:0b}" , cpu.status);
        assert_eq!(cpu.status & 0b1000_0011, 0b0000_0011);
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

    #[test]
    fn test_0x4a_lsr_accumulator_with_carry() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x18,0xa9,0x01,0x4a,0x00]);
        println!(" register_a is {:0x}" , cpu.register_a as u8);
        assert_eq!(cpu.register_a, 0x00);
        println!(" status  is {:0b}" , cpu.status);
        assert_eq!(cpu.status & 0b1000_0011, 0b0000_0011);
    }

    #[test]
    fn test_0x2a_rol_accumulator() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38,0xa9,0x2a,0x2a,0x00]);
        println!(" register_a is {:0x}" , cpu.register_a as u8);
        assert_eq!(cpu.register_a, 0x55);
        println!(" status  is {:0b}" , cpu.status);
        assert_eq!(cpu.status & 0b1000_0011, 0b0000_0000);
    }

    #[test] //overflow_multiple ?
    fn test_0x2a_rol_accumulator_with_no_carry() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x18,0xa9,0x80,0x2a,0x00]);
        println!(" register_a is {:0x}" , cpu.register_a as u8);
        assert_eq!(cpu.register_a, 0x00);
        println!(" status  is {:0b}" , cpu.status);
        assert_eq!(cpu.status & 0b1000_0011, 0b0000_0011);
    }

    #[test] //overflow_multiple ?
    fn test_0x2a_rol_accumulator_with_carry() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38,0xa9,0x00,0x2a,0x00]);
        println!(" register_a is {:0x}" , cpu.register_a as u8);
        assert_eq!(cpu.register_a, 0x01);
        println!(" status  is {:0b}" , cpu.status);
        assert_eq!(cpu.status & 0b1000_0011, 0b0000_0000);
    }

    #[test] //overflow_multiple ?
    fn test_0x6a_ror_accumulator_with_zero() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x18,0xa9,0x01,0x6a,0x00]);
        println!(" register_a is {:0x}" , cpu.register_a as u8);
        assert_eq!(cpu.register_a, 0x00);
        println!(" status  is {:0b}" , cpu.status);
        assert_eq!(cpu.status & 0b1000_0011, 0b0000_0011);
    }
    

    #[test] 
    fn test_0x29_and_absolute() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9,0xaa,0x8d,0x10,0x00,0xa9,0x5d,0x2d,0x10,0x00,0x00]);
        println!(" register_a is {:0x}" , cpu.register_a as u8);
        assert_eq!(cpu.register_a, 0x08);
        println!(" status  is {:0b}" , cpu.status);
        assert_eq!(cpu.status & 0b1000_0011, 0b0000_0000);
    }

    #[test] 
    fn test_0x29_and_absolute_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9,0xaa,0x8d,0x10,0x00,0xa9,0xd5,0x2d,0x10,0x00,0x00]);
        println!(" register_a is {:0x}" , cpu.register_a as u8);
        assert_eq!(cpu.register_a, 0x80);
        println!(" status  is {:0b}" , cpu.status);
        assert_eq!(cpu.status & 0b1000_0011, 0b1000_0000);
    }
    #[test] 
    fn test_0x0d_ora_absolute() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9,0xaa,0x8d,0x10,0x00,0xa9,0x55,0x0d,0x10,0x00,0x00]);
        println!(" register_a is {:0x}" , cpu.register_a as u8);
        assert_eq!(cpu.register_a, 0xff);
        println!(" status  is {:0b}" , cpu.status);
        assert_eq!(cpu.status & 0b1000_0011, 0b1000_0000);
    }

    #[test] 
    fn test_0x4d_eor_absolute_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9,0x80,0x8d,0x10,0x00,0xa9,0x01,0x4d,0x10,0x00,0x00]);
        println!(" register_a is {:0x}" , cpu.register_a as u8);
        assert_eq!(cpu.register_a, 0x81);
        println!(" status  is {:0b}" , cpu.status);
        assert_eq!(cpu.status & 0b1000_0011, 0b1000_0000);
    }

    #[test] 
    fn test_0x2c_bit_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9,0x80,0x2c,0x00]);
        println!(" status  is {:0b}" , cpu.status);
        assert_eq!(cpu.status & 0b1000_0011, 0b0000_0010);
    }

    #[test] 
    fn test_0x2c_bit_absolute_clear_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9,0x40,0x8d,0x10,0x00,0xa9,0x00,0x2c,0x10,0x00,0x00]);
        println!(" status  is {:0b}" , cpu.status);
        assert_eq!(cpu.status & 0b1100_0011, 0b0100_0010);
    }

    #[test] 
    fn test_0xcd_cmp_absolute_with_carry_and_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9,0x40,0x8d,0x10,0x00,0xcd,0x10,0x00,0x00]);
        println!(" status  is {:0b}" , cpu.status);
        assert_eq!(cpu.status & 0b1100_0011, 0b0000_0011);
    }

    #[test] 
    fn test_0xec_cpx_absolute_with_carry_and_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2,0x40,0x8e,0x10,0x00,0xec,0x10,0x00,0x00]);
        println!(" status  is {:0b}" , cpu.status);
        assert_eq!(cpu.status & 0b1100_0011, 0b0000_0011);
    }
    #[test] 
    fn test_0xcc_cpy_absolute_with_carry_and_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa0,0x40,0x8c,0x10,0x00,0xcc,0x10,0x00,0x00]);
        println!(" status  is {:0b}" , cpu.status);
        assert_eq!(cpu.status & 0b1100_0011, 0b0000_0011);
    }

    #[test]
    fn test_0x90_bcc_not_carry() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x90,0x02,0xe8,0xe8,0xe8,0x00]);
        assert_eq!(cpu.register_x, 1);
    }

    #[test]
    fn test_0x90_bcc_wiht_carry() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38,0x90,0x02,0xe8,0xe8,0xe8,0x00]);
        assert_eq!(cpu.register_x, 3);
    }

    #[test]
    fn test_0x90_bcc_wiht_no_carry_minus() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9,0x00,0x8d,0xff,0x7f,0xa9,0xe8,0x8d,0xfe,0x7f,0x90,0xf2,0x00]);

        println!("register_x is {}" , cpu.register_x);
        assert_eq!(cpu.register_x,1)
    }

    #[test]
    fn test_0x4c_jmp_absolute() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9,0x00,0x8d,0xff,0x7f,0xa9,0xe8,0x8d,0xfe,0x7f,0x4c,0xfe,0x7f,0x00]);

        println!("register_x is {}" , cpu.register_x);
        assert_eq!(cpu.register_x,1)
    }

    #[test]
    fn test_0x48_pha() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9 ,0x50 ,0x48 ,0xa9 ,0x05 ,0x48,0x00]);
        println!("stack pointer is {}", cpu.stackpointer);
        assert_eq!(cpu.mem_read(0x01ff), 0x50);
        assert_eq!(cpu.mem_read(0x1fe), 0x05);
        assert_eq!(cpu.stackpointer,0xfd);
    }

    #[test]
    fn test_0x68_pla() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9 ,0x50 ,0x48 ,0xa9 ,0x05 ,0x48,0x68,0x68,0x00]);
        println!("stack pointer is {}", cpu.stackpointer);
        assert_eq!(cpu.register_a, 0x50);
        assert_eq!(cpu.stackpointer,0xff);
    }

    #[test]
    fn test_0x08_php() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38 ,0x08 ,0x00]);
        println!("stack pointer is {}", cpu.stackpointer);
        assert_eq!(cpu.mem_read(0x1ff), 0x31);
        assert_eq!(cpu.stackpointer,0xfe);
    }

    #[test]
    fn test_0x28_plp() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38 ,0x08,0x18,0x28 ,0x00]);
        println!("stack pointer is {}", cpu.stackpointer);
        println!("cpu status is {}", cpu.status);
        assert_eq!(cpu.mem_read(0x1ff), 0x31);
        assert_eq!(cpu.status , 0b00110001);
        assert_eq!(cpu.stackpointer,0xff);
    }

    #[test]
    fn test_0x9a_txs() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2 ,0x50 ,0x9a ,0x00]);
        assert_eq!(cpu.stackpointer,0x50);
    }
    #[test]
    fn test_0xba_tsx() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2 ,0x50 ,0x9a,0xa2,0x40,0xba ,0x00]);
        assert_eq!(cpu.register_x,0x50);
    }

    #[test]
    fn test_0x78_sei() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x78 ,0x00]);
        assert_eq!(is_flag_set(INTERRUPT_FLAG, cpu.status),true);
    }

    #[test]
    fn test_0x58_cli() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x78,0x58 ,0x00]);
        assert_eq!(is_flag_set(INTERRUPT_FLAG, cpu.status),false);
    }

    #[test]
    fn test_0xf8_sed() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xf8,0x00]);
        assert_eq!(is_flag_set(DECIMAL_FLAG, cpu.status),true);
    }

    #[test]
    fn test_0xd8_cld() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xf8,0xd8,0x00]);
        assert_eq!(is_flag_set(DECIMAL_FLAG, cpu.status),false);
    }

    #[test]
    fn test_0xb8_clv() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9 ,0x7f ,0x18, 0x69, 0x01 ,0x8d ,0x00 ,0x02,0xb8,0x00]);
        assert_eq!(is_flag_set(OVERFOW_FLAG, cpu.status),false);
    }

    
}