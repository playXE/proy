use super::assembler::*;
#[derive(Copy, Clone, PartialOrd, Ord, Eq, PartialEq, Debug, Hash)]
#[repr(u8)]
pub enum X86Gpr {
    Eax,
    Ecx,
    Edx,
    Ebx,
    Esp,
    Ebp,
    Esi,
    Edi,
    #[cfg(target_arch = "x86_64")]
    R8,
    #[cfg(target_arch = "x86_64")]
    R9,
    #[cfg(target_arch = "x86_64")]
    R10,
    #[cfg(target_arch = "x86_64")]
    R11,
    #[cfg(target_arch = "x86_64")]
    R12,
    #[cfg(target_arch = "x86_64")]
    R13,
    #[cfg(target_arch = "x86_64")]
    R14,
    #[cfg(target_arch = "x86_64")]
    R15,
}
#[derive(Copy, Clone, PartialOrd, Ord, Eq, PartialEq, Debug, Hash)]
#[repr(u8)]
pub enum X86Fpr {
    XMM0,
    XMM1,
    XMM2,
    XMM3,
    XMM4,
    XMM5,
    XMM6,
    XMM7,
}

#[derive(Copy, Clone, PartialOrd, Ord, Eq, PartialEq, Debug, Hash)]
#[repr(u8)]
enum ModRmMode {
    NoDisp,
    Disp8,
    Disp32,
    Reg,
}

const HAS_SIB: u8 = X86Gpr::Esp as u8;
const NO_BASE: u8 = X86Gpr::Ebp as u8;
const NO_INDEX: u8 = X86Gpr::Esp as u8;
#[cfg(target_arch = "x86_64")]
const NO_BASE2: u8 = X86Gpr::R13 as u8;
#[cfg(target_arch = "x86_64")]
const HAS_SIB2: u8 = X86Gpr::R12 as u8;

pub struct X86InsFormatter {
    buffer: AssemblerBuffer,
}

impl X86InsFormatter {
    fn put_modrm(&mut self, mode: ModRmMode, r: u8, rm: i32) {
        self.buffer
            .put_byte(((mode as u8) << 6) | ((r as u8 & 7) << 3) | (rm as u8 & 7));
    }

    fn put_modrm_sib(&mut self, mode: ModRmMode, r: u8, base: u8, index: u8, scale: i32) {
        self.put_modrm(mode, r, HAS_SIB as _);
        self.buffer
            .put_byte((scale << 6) as u8 | ((index & 7) << 3) | (base & 7));
    }

    fn register_modrm(&mut self, reg: u8, rm: u8) {
        self.put_modrm(ModRmMode::Reg, reg, rm as _);
    }
    fn memory_modrm_1(&mut self, r: u8, base: u8, offset: i32) {
        let cond;
        #[cfg(target_arch = "x86_64")]
        {
            cond = base == HAS_SIB || base == HAS_SIB2
        };
        #[cfg(target_arch = "x86")]
        {
            cond = base == HAS_SIB
        };
        if cond {
            if offset == 0 {
                self.put_modrm_sib(ModRmMode::NoDisp, r, base, NO_INDEX as _, 0);
            } else if can_sign_extend(offset) {
                self.put_modrm_sib(ModRmMode::Disp8, r, base, NO_INDEX, 0);
                self.buffer.put_byte(offset as _);
            } else {
                self.put_modrm_sib(ModRmMode::Disp32, r, base, NO_INDEX, 0);
                self.buffer.put_int(offset as _);
            }
        } else {
            let additional;
            #[cfg(target_arch = "x86_64")]
            {
                additional = base != NO_BASE2;
            }
            #[cfg(target_arch = "x86")]
            {
                additional = true;
            }
            if offset == 0 && (base != NO_BASE) && additional {
                self.put_modrm(ModRmMode::NoDisp, r, base as _);
            } else if can_sign_extend(offset) {
                self.put_modrm(ModRmMode::Disp8, r, base as _);
            } else {
                self.put_modrm(ModRmMode::Disp32, r, base as _);
                self.buffer.put_int(offset);
            }
        }
    }

    fn memory_modrm_disp8(&mut self, r: u8, base: u8, offset: i32) {
        assert!(can_sign_extend(offset));
        let cond;
        #[cfg(target_arch = "x86_64")]
        {
            cond = base == HAS_SIB || base == HAS_SIB2
        };
        #[cfg(target_arch = "x86")]
        {
            cond = base == HAS_SIB
        };
        if cond {
            self.put_modrm_sib(ModRmMode::Disp8, r, base, NO_INDEX, 0);
            self.buffer.put_byte(offset as _);
        } else {
            self.put_modrm(ModRmMode::Disp8, r, base as _);
            self.buffer.put_byte(offset as _);
        }
    }
    fn memory_modrm_disp32(&mut self, r: u8, base: u8, offset: i32) {
        assert!(can_sign_extend(offset));
        let cond;
        #[cfg(target_arch = "x86_64")]
        {
            cond = base == HAS_SIB || base == HAS_SIB2
        };
        #[cfg(target_arch = "x86")]
        {
            cond = base == HAS_SIB
        };
        if cond {
            self.put_modrm_sib(ModRmMode::Disp32, r, base, NO_INDEX, 0);
            self.buffer.put_int(offset as _);
        } else {
            self.put_modrm(ModRmMode::Disp32, r, base as _);
            self.buffer.put_int(offset as _);
        }
    }

    fn memory_modrm_2(&mut self, r: u8, base: u8, index: u8, scale: i32, offset: i32) {
        assert!(index != NO_INDEX);
        let cond;
        #[cfg(target_arch = "x86_64")]
        {
            cond = offset == 0 && (base != NO_BASE) && (base != NO_BASE2)
        };
        #[cfg(target_arch = "x86")]
        {
            cond = offset == 0 && (base != NO_BASE)
        };
        if cond {
            self.put_modrm_sib(ModRmMode::NoDisp, r, base, index, scale);
        } else if can_sign_extend(offset) {
            self.put_modrm_sib(ModRmMode::Disp8, r, base, index, scale);
            self.buffer.put_byte(offset as _);
        } else {
            self.put_modrm_sib(ModRmMode::Disp32, r, base, index, scale);
            self.buffer.put_int(offset as _);
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[inline]
    pub fn reg_requires_rex(r: u8) -> bool {
        r >= X86Gpr::R8 as u8
    }
    #[cfg(target_arch = "x86_64")]
    #[inline]
    pub fn byte_reg_requires_rex(r: u8) -> bool {
        r >= X86Gpr::Esp as u8
    }
    #[cfg(target_arch = "x86_64")]
    #[inline]
    pub fn emit_rex(&mut self, w: u8, r: u8, x: u8, b: u8) {
        self.buffer
            .put_byte(PRE_REX | (w << 3) | ((r >> 3) << 2) | ((x >> 3) << 1) | (b >> 3));
    }

    #[cfg(target_arch = "x86_64")]
    #[inline]
    pub fn emit_rexw(&mut self, r: u8, x: u8, b: u8) {
        self.emit_rex(1, r, x, b);
    }
    #[cfg(target_arch = "x86_64")]
    #[inline]
    pub fn emit_rex_if(&mut self, c: bool, r: u8, x: u8, b: u8) {
        if c {
            self.emit_rex(0, r, x, b);
        }
    }
    #[cfg(target_arch = "x86_64")]
    #[inline]
    pub fn emit_rex_if_needed(&mut self, r: u8, x: u8, b: u8) {
        self.emit_rex_if(
            Self::reg_requires_rex(r) || Self::reg_requires_rex(x) || Self::reg_requires_rex(b),
            r,
            x,
            b,
        );
    }
    #[cfg(target_arch = "x86")]
    #[inline]
    pub const fn reg_requires_rex(_: u8) -> bool {
        false
    }
    #[cfg(target_arch = "x86")]
    #[inline]
    pub const fn byte_reg_requires_rex(_: u8) -> bool {
        false
    }
    #[cfg(target_arch = "x86")]
    #[inline]
    pub const fn emit_rex_if(&mut self, _: u8, _: u8, _: u8) {}
    #[cfg(target_arch = "x86")]
    #[inline]
    pub const fn emit_rex_if_needed(&mut self, _: u8, _: u8, _: u8) {}

    pub fn executable_writable(&mut self) -> *mut u8 {
        self.buffer.executable_writable_memory().unwrap()
    }

    pub fn executable_readable(&mut self) -> *const u8 {
        self.buffer.executable_memory().unwrap()
    }

    pub fn code_size(&self) -> usize {
        self.buffer.code_size()
    }
    pub fn data(&self) -> &[u8] {
        self.buffer.data()
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        self.buffer.data_mut()
    }

    pub fn imm_rel(&mut self) -> AssemblerLabel {
        self.buffer.put_int(0);
        self.label()
    }

    pub fn imm64(&mut self, imm: i64) {
        self.buffer.put_long(imm as _);
    }

    pub fn imm32(&mut self, imm: i32) {
        self.buffer.put_int(imm as _);
    }

    pub fn imm16(&mut self, imm: i16) {
        self.buffer.put_short(imm as _);
    }

    pub fn imm8(&mut self, imm: i8) {
        self.buffer.put_byte(imm as _);
    }

    pub fn label(&mut self) -> AssemblerLabel {
        self.buffer.label()
    }

    pub fn one_byte_op8_1(&mut self, op: u8, g: u8, rm: u8) {
        self.emit_rex_if(Self::byte_reg_requires_rex(rm), 0, 0, rm);
        self.buffer.put_byte(op);
        self.register_modrm(g, rm);
    }
    pub fn one_byte_op8_2(&mut self, op: u8, reg: u8, rm: u8) {
        self.emit_rex_if(
            Self::byte_reg_requires_rex(reg) || Self::byte_reg_requires_rex(rm),
            reg,
            0,
            rm,
        );
        self.buffer.put_byte(op);
        self.register_modrm(reg, rm);
    }

    pub fn one_byte_op8_3(
        &mut self,
        op: u8,
        reg: u8,
        base: u8,
        index: u8,
        scale: i32,
        offset: i32,
    ) {
        self.emit_rex_if(
            Self::byte_reg_requires_rex(reg)
                || Self::byte_reg_requires_rex(base)
                || Self::byte_reg_requires_rex(index),
            reg,
            index,
            base,
        );
        self.buffer.put_byte(op);
        self.memory_modrm_2(reg, base, index, scale, offset);
    }

    pub fn two_byte_op8_1(&mut self, op: u8, reg: u8, rm: u8) {
        self.emit_rex_if(
            Self::byte_reg_requires_rex(reg) || Self::byte_reg_requires_rex(rm),
            reg,
            0,
            rm,
        );
        self.buffer.append(&[OP_2BYTE_ESCAPE, op]);
        self.register_modrm(reg, rm);
    }

    pub fn two_byte_op8_2(&mut self, op: u8, g: u8, rm: u8) {
        self.emit_rex_if(Self::byte_reg_requires_rex(rm), 0, 0, rm);
        self.buffer.append(&[OP_2BYTE_ESCAPE, op]);
        self.register_modrm(g, rm);
    }

    cfg_if::cfg_if! {
        if #[cfg(target_arch="x86_64")]
        {
            pub fn one_byte_op64(&mut self,op: u8) {
                self.emit_rexw(0, 0,0);
                self.buffer.put_byte(op);
            }
            pub fn one_byte_op64_1(&mut self,op: u8,r: u8) {
                self.emit_rexw(0, 0, r);
                self.buffer.put_byte(op + (r & 7));
            }

            pub fn one_byte_op64_2(&mut self,op: u8,r: u8,rm: u8) {
                self.emit_rexw(r, 0,rm);
                self.buffer.put_byte(op);
                self.register_modrm(r, rm);
            }

            pub fn one_byte_op64_3(&mut self,op: u8,reg: u8,base: u8,offset: i32) {
                self.emit_rexw(reg, 0, base);
                self.buffer.put_byte(op);
                self.memory_modrm_1(reg,base,offset);
            }

            pub fn one_byte_op64_disp32(&mut self,op: u8,reg: u8,base: u8,offset: i32) {
                self.emit_rexw(reg, 0,base);
                self.buffer.put_byte(op);
                self.memory_modrm_disp32(reg, base,offset);
            }

            pub fn one_byte_op64_disp8(&mut self,op: u8,reg: u8,base: u8,offset: i32) {
                self.emit_rexw(reg, 0,base);
                self.buffer.put_byte(op);
                self.memory_modrm_disp8(reg, base,offset);
            }

            pub fn one_byte_op64_4(&mut self,op: u8,reg: u8,base: u8,index: u8,scale: i32,offset: i32) {
                self.emit_rexw(reg, index,base);
                self.buffer.put_byte(op);
                self.memory_modrm_2(reg, base,index,scale,offset);
            }

            pub fn two_byte_op64(&mut self,op: u8,reg: u8,rm: u8) {
                self.emit_rexw(reg,0,rm);
                self.buffer.put_byte(OP_2BYTE_ESCAPE);
                self.buffer.put_byte(op);
                self.register_modrm(reg, rm);
            }
        } // x86assembler is included in build only on x86_32 and x86_64 so we do not need to check for other platforms
    }

    pub fn prefix(&mut self, x: u8) {
        self.buffer.put_byte(x);
    }
    pub fn one_byte_op_1(&mut self, op: u8) {
        self.buffer.put_byte(op);
    }
    pub fn one_byte_op_2(&mut self, op: u8, reg: u8) {
        self.emit_rex_if_needed(0, 0, reg);
        self.buffer.put_byte(op + (reg & 7));
    }

    pub fn one_byte_op_3(&mut self, op: u8, reg: u8, base: u8, offset: i32) {
        self.emit_rex_if_needed(reg, 0, base);
        self.buffer.put_byte(op);
        self.memory_modrm_1(reg, base, offset);
    }
    pub fn one_byte_op_disp32(&mut self, op: u8, reg: u8, base: u8, offset: i32) {
        self.emit_rex_if_needed(reg, 0, base);
        self.buffer.put_byte(op);
        self.memory_modrm_disp32(reg, base, offset);
    }
    pub fn one_byte_op_disp8(&mut self, op: u8, reg: u8, base: u8, offset: i32) {
        self.emit_rex_if_needed(reg, 0, base);
        self.buffer.put_byte(op);
        self.memory_modrm_disp8(reg, base, offset);
    }
    pub fn one_byte_op_4(&mut self, op: u8, reg: u8, base: u8, index: u8, scale: i32, offset: i32) {
        self.emit_rex_if_needed(reg, index, base);
        self.buffer.put_byte(op);
        self.memory_modrm_2(reg, base, index, scale, offset);
    }

    pub fn one_byte_op_6(&mut self, op: u8, reg: u8, rm: u8) {
        self.emit_rex_if_needed(reg, 0, rm);
        self.buffer.put_byte(op);
        self.register_modrm(reg, rm);
    }

    #[cfg(target_arch = "x86")]
    pub fn one_byte_op_5(&mut self, _op: u8, _reg: u8, _address: usize) {
        unimplemented!()
    }
}

pub const fn can_sign_extend(x: i32) -> bool {
    return x == (x as i8) as i32;
}

macro_rules! opcodes {
    (1 $($i: ident = $e: expr),*) => {
       $( const $i: u8 = $e;)*
    };
    (2 $($i: ident = $e: expr),*) => {
        $( const $i: u16 = $e;)*
     };
}

opcodes! {1
        OP_ADD_EvGv                     = 0x01,
        OP_ADD_GvEv                     = 0x03,
        OP_OR_EvGv                      = 0x09,
        OP_OR_GvEv                      = 0x0B,
        OP_2BYTE_ESCAPE                 = 0x0F,
        OP_AND_EvGv                     = 0x21,
        OP_AND_GvEv                     = 0x23,
        OP_SUB_EvGv                     = 0x29,
        OP_SUB_GvEv                     = 0x2B,
        PRE_PREDICT_BRANCH_NOT_TAKEN    = 0x2E,
        OP_XOR_EvGv                     = 0x31,
        OP_XOR_GvEv                     = 0x33,
        OP_CMP_EvGv                     = 0x39,
        OP_CMP_GvEv                     = 0x3B,
        PRE_REX                         = 0x40,
        OP_PUSH_EAX                     = 0x50,
        OP_POP_EAX                      = 0x58,
        OP_MOVSXD_GvEv                  = 0x63,
        PRE_OPERAND_SIZE                = 0x66,
        PRE_SSE_66                      = 0x66,
        OP_PUSH_Iz                      = 0x68,
        OP_IMUL_GvEvIz                  = 0x69,
        OP_GROUP1_EbIb                  = 0x80,
        OP_GROUP1_EvIz                  = 0x81,
        OP_GROUP1_EvIb                  = 0x83,
        OP_TEST_EbGb                    = 0x84,
        OP_TEST_EvGv                    = 0x85,
        OP_XCHG_EvGv                    = 0x87,
        OP_MOV_EbGb                     = 0x88,
        OP_MOV_EvGv                     = 0x89,
        OP_MOV_GvEv                     = 0x8B,
        OP_LEA                          = 0x8D,
        OP_GROUP1A_Ev                   = 0x8F,
        OP_NOP                          = 0x90,
        OP_CDQ                          = 0x99,
        OP_MOV_EAXOv                    = 0xA1,
        OP_MOV_OvEAX                    = 0xA3,
        OP_MOV_EAXIv                    = 0xB8,
        OP_GROUP2_EvIb                  = 0xC1,
        OP_RET                          = 0xC3,
        OP_GROUP11_EvIb                 = 0xC6,
        OP_GROUP11_EvIz                 = 0xC7,
        OP_INT3                         = 0xCC,
        OP_GROUP2_Ev1                   = 0xD1,
        OP_GROUP2_EvCL                  = 0xD3,
        OP_ESCAPE_DD                    = 0xDD,
        OP_CALL_rel32                   = 0xE8,
        OP_JMP_rel32                    = 0xE9,
        PRE_SSE_F2                      = 0xF2,
        PRE_SSE_F3                      = 0xF3,
        OP_HLT                          = 0xF4,
        OP_GROUP3_EbIb                  = 0xF6,
        OP_GROUP3_Ev                    = 0xF7,
        OP_GROUP3_EvIz                  = 0xF7, // OP_GROUP3_Ev has an immediate, when instruction is a test.
        OP_GROUP5_Ev                    = 0xFF
}

opcodes! {
    1
    OP2_MOVSD_VsdWsd    = 0x10,
        OP2_MOVSD_WsdVsd    = 0x11,
        OP2_MOVSS_VsdWsd    = 0x10,
        OP2_MOVSS_WsdVsd    = 0x11,
        OP2_CVTSI2SD_VsdEd  = 0x2A,
        OP2_CVTTSD2SI_GdWsd = 0x2C,
        OP2_UCOMISD_VsdWsd  = 0x2E,
        OP2_ADDSD_VsdWsd    = 0x58,
        OP2_MULSD_VsdWsd    = 0x59,
        OP2_CVTSD2SS_VsdWsd = 0x5A,
        OP2_CVTSS2SD_VsdWsd = 0x5A,
        OP2_SUBSD_VsdWsd    = 0x5C,
        OP2_DIVSD_VsdWsd    = 0x5E,
        OP2_SQRTSD_VsdWsd   = 0x51,
        OP2_ANDNPD_VpdWpd   = 0x55,
        OP2_XORPD_VpdWpd    = 0x57,
        OP2_MOVD_VdEd       = 0x6E,
        OP2_MOVD_EdVd       = 0x7E,
        OP2_JCC_rel32       = 0x80,
        OP_SETCC            = 0x90,
        OP2_IMUL_GvEv       = 0xAF,
        OP2_MOVZX_GvEb      = 0xB6,
        OP2_MOVSX_GvEb      = 0xBE,
        OP2_MOVZX_GvEw      = 0xB7,
        OP2_MOVSX_GvEw      = 0xBF,
        OP2_PEXTRW_GdUdIb   = 0xC5,
        OP2_PSLLQ_UdqIb     = 0x73,
        OP2_PSRLQ_UdqIb     = 0x73,
        OP2_POR_VdqWdq      = 0xEB
}

opcodes! {1
    GROUP1_OP_ADD = 0,
    GROUP1_OP_OR  = 1,
    GROUP1_OP_ADC = 2,
    GROUP1_OP_AND = 4,
    GROUP1_OP_SUB = 5,
    GROUP1_OP_XOR = 6,
    GROUP1_OP_CMP = 7,

    GROUP1A_OP_POP = 0,

    GROUP2_OP_ROL = 0,
    GROUP2_OP_ROR = 1,
    GROUP2_OP_RCL = 2,
    GROUP2_OP_RCR = 3,

    GROUP2_OP_SHL = 4,
    GROUP2_OP_SHR = 5,
    GROUP2_OP_SAR = 7,

    GROUP3_OP_TEST = 0,
    GROUP3_OP_NOT  = 2,
    GROUP3_OP_NEG  = 3,
    GROUP3_OP_IDIV = 7,

    GROUP5_OP_CALLN = 2,
    GROUP5_OP_JMPN  = 4,
    GROUP5_OP_PUSH  = 6,

    GROUP11_MOV = 0,

    GROUP14_OP_PSLLQ = 6,
    GROUP14_OP_PSRLQ = 2,

    ESCAPE_DD_FSTP_doubleReal = 3
}

pub struct X86Assembler {
    pub formatter: X86InsFormatter,
    idx_of_last_watchpoint: i32,
    idx_of_tail_last_watchpoint: i32,
}

impl X86Assembler {
    pub fn new() -> Self {
        Self {
            formatter: X86InsFormatter {
                buffer: AssemblerBuffer {
                    storage: Vec::with_capacity(128),
                    index: 0,
                },
            },
            idx_of_last_watchpoint: 0,
            idx_of_tail_last_watchpoint: 0,
        }
    }
    pub fn code(&self) -> &[u8] {
        &self.formatter.data()
    }
    fn store_possibly_unaligned<T: Sized>(location: *mut u8, idx: i32, value: T) {
        unsafe {
            let ptr = (location.cast::<T>()).offset(idx as _);
            std::ptr::copy_nonoverlapping(&value, ptr, std::mem::size_of::<T>());
        }
    }

    fn set_i32(location: *mut u8, value: i32) {
        Self::store_possibly_unaligned(location, -1, value);
    }

    fn set_ptr(location: *mut u8, value: *mut u8) {
        Self::store_possibly_unaligned(location, -1, value);
    }

    fn set_i8(location: *mut u8, value: i8) {
        unsafe {
            *location.cast::<i8>().offset(-1) = value;
        }
    }

    fn set_rel32(from: *mut u8, to: *mut u8) {
        let offset = to as usize - from as usize;
        Self::set_i32(from, offset as _);
    }

    fn get_relocate_offset(code: *mut u8, lbl: AssemblerLabel) -> *mut u8 {
        assert!(lbl.is_set());
        return (code as usize + lbl.offset as usize) as *mut u8;
    }

    fn replace_with_address_computation(mut ptr: *mut u8) {
        unsafe {
            #[cfg(target_arch = "x86_64")]
            {
                if (*ptr & !15) == PRE_REX {
                    ptr = ptr.offset(1);
                }
            }
            match *ptr {
                OP_MOV_GvEv => *ptr = OP_LEA,
                OP_LEA => (),
                _ => unreachable!(),
            }
        }
    }
    fn replace_with_load(mut ptr: *mut u8) {
        unsafe {
            #[cfg(target_arch = "x86_64")]
            {
                if (*ptr & !15) == PRE_REX {
                    ptr = ptr.offset(1);
                }
            }
            match *ptr {
                OP_MOV_GvEv => (),
                OP_LEA => *ptr = OP_MOV_GvEv,
                _ => unreachable!(),
            }
        }
    }
    fn revert_jump_to_cmpl_im_force32(mut ptr: *mut u8, imm: i32, offset: i32, dst: u8) {
        const OPCODE_BYTES: u8 = 1;
        const MODRM_BYTES: u8 = 1;
        unsafe {
            *ptr.offset(0) = OP_GROUP1_EvIz;
            *ptr.offset(1) = ((ModRmMode::NoDisp as u8) << 6) | (GROUP1_OP_CMP << 3) | dst;
            let bytes: [u8; 4] = std::mem::transmute(imm);
            for i in OPCODE_BYTES + MODRM_BYTES..5 {
                *ptr.offset(i as isize) =
                    bytes[i as usize - OPCODE_BYTES as usize - MODRM_BYTES as usize];
            }
        }
    }

    fn revert_jump_to_cmpl_ir_force32(mut ptr: *mut u8, imm: i32, dst: u8) {
        const OPCODE_BYTES: u8 = 1;
        const MODRM_BYTES: u8 = 1;
        unsafe {
            *ptr.offset(0) = OP_GROUP1_EvIz;
            *ptr.offset(1) = ((ModRmMode::Reg as u8) << 6) | (GROUP1_OP_CMP << 3) | dst;
            let bytes: [u8; 4] = std::mem::transmute(imm);
            for i in OPCODE_BYTES + MODRM_BYTES..5 {
                *ptr.offset(i as isize) =
                    bytes[i as usize - OPCODE_BYTES as usize - MODRM_BYTES as usize];
            }
        }
    }

    fn revert_jump_to_movq_i64r(mut ptr: *mut u8, imm: i64, dst: u8) {
        const REX_BYTES: u8 = 1;
        const OPCODE_BYTES: u8 = 1;
        unsafe {
            *ptr.offset(0) = PRE_REX | (1 << 3) | (dst >> 3);
            *ptr.offset(1) = OP_MOV_EAXIv | (dst & 7);
            let bytes: [u8; 8] = std::mem::transmute(imm);
            for i in REX_BYTES + OPCODE_BYTES..5 {
                *ptr.offset(i as isize) =
                    bytes[i as usize - REX_BYTES as usize - OPCODE_BYTES as usize];
            }
        }
    }
    fn read_ptr(loc: *mut u8) -> *mut u8 {
        unsafe { *loc.cast::<*mut u8>().offset(-1) }
    }
    fn replace_with_jump(mut ptr: *mut u8, to: *mut u8) {
        unsafe {
            let dist = (to as usize - (ptr as usize + 5));
            *ptr.offset(0) = OP_JMP_rel32;
            *ptr.offset(1).cast::<i32>() = dist as i32;
        }
    }

    fn repatch_ptr(location: *mut u8, value: *mut u8) {
        Self::set_ptr(location, value)
    }

    fn repatch_i32(location: *mut u8, value: i32) {
        Self::set_i32(location, value)
    }

    fn relink_jump(from: *mut u8, to: *mut u8) {
        Self::set_rel32(from, to);
    }
    fn relink_call(from: *mut u8, to: *mut u8) {
        Self::set_rel32(from, to);
    }

    fn link_ptr(code: *mut u8, w: AssemblerLabel, value: *mut u8) {
        Self::set_ptr(unsafe { code.offset(w.offset as isize) }, value);
    }
    fn link_call(code: *mut u8, w: AssemblerLabel, value: *mut u8) {
        Self::set_ptr(unsafe { code.offset(w.offset as isize) }, value);
    }
    fn slink_jump(code: *mut u8, w: AssemblerLabel, value: *mut u8) {
        Self::set_ptr(unsafe { code.offset(w.offset as isize) }, value);
    }

    fn link_jump(&mut self, from: AssemblerLabel, to: AssemblerLabel) {
        let code = self.formatter.data_mut().as_mut_ptr();
        Self::set_rel32(unsafe { code.offset(from.offset as _) }, unsafe {
            code.offset(to.offset as _)
        });
    }

    pub fn align(&mut self, alignment: usize) -> AssemblerLabel {
        while self.formatter.code_size() & (alignment - 1) == 0 {
            self.formatter.one_byte_op_1(OP_HLT);
        }

        return self.label();
    }

    pub fn label(&mut self) -> AssemblerLabel {
        let mut r = self.formatter.label();
        while (r.offset as i32) < self.idx_of_tail_last_watchpoint {
            self.formatter.one_byte_op_1(OP_NOP);
            r = self.formatter.label();
        }
        r
    }

    pub fn label_ignoring_watchpoints(&mut self) -> AssemblerLabel {
        self.formatter.label()
    }

    pub fn label_for_watchpoit(&mut self) -> AssemblerLabel {
        let mut result = self.formatter.label();
        if result.offset as i32 != self.idx_of_last_watchpoint {
            result = self.label();
        }
        self.idx_of_last_watchpoint = result.offset as _;
        self.idx_of_tail_last_watchpoint = result.offset as i32 + 5;
        return result;
    }

    pub fn ret(&mut self) {
        self.formatter.one_byte_op_1(OP_RET);
    }
    pub fn int3(&mut self) {
        self.formatter.one_byte_op_1(OP_INT3);
    }

    pub fn predict_not_taken(&mut self) {
        self.formatter.prefix(PRE_PREDICT_BRANCH_NOT_TAKEN);
    }

    pub fn push_r(&mut self, r: u8) {
        self.formatter.one_byte_op_2(OP_PUSH_EAX, r);
    }
    pub fn pop_r(&mut self, r: u8) {
        self.formatter.one_byte_op_2(OP_POP_EAX, r);
    }
    pub fn push_i32(&mut self, imm: i32) {
        self.formatter.one_byte_op_1(OP_PUSH_Iz);
        self.formatter.imm32(imm);
    }

    pub fn push_m(&mut self, offset: i32, base: u8) {
        self.formatter
            .one_byte_op_3(OP_GROUP5_Ev, GROUP5_OP_PUSH, base, offset);
    }

    pub fn pop_m(&mut self, offset: i32, base: u8) {
        self.formatter
            .one_byte_op_3(OP_GROUP1A_Ev, GROUP1A_OP_POP, base, offset);
    }

    #[cfg(target_arch = "x86")]
    pub fn adcl_im(&mut self, imm: i32, addr: *mut u8) {
        if can_sign_extend(imm) {
            self.formatter
                .one_byte_op_5(OP_GROUP1_EvIb, GROUP1_OP_ADC, addr as _);
            self.formatter.imm8(imm as u8);
        } else {
            self.formatter
                .one_byte_op_5(OP_GROUP1_EvIz, GROUP1_OP_ADC, addr as _);
            self.formatter.imm32(imm as u8);
        }
    }

    pub fn addl_rr(&mut self, src: u8, dst: u8) {
        self.formatter.one_byte_op_6(OP_ADD_EvGv, src, dst);
    }

    pub fn addl_mr(&mut self, offset: i32, base: u8, dst: u8) {
        self.formatter.one_byte_op_3(OP_ADD_GvEv, dst, base, offset);
    }

    pub fn addl_rm(&mut self, src: u8, offset: i32, base: u8) {
        self.formatter.one_byte_op_3(OP_ADD_EvGv, src, base, offset);
    }
    pub fn addl_ir(&mut self, imm: i32, dst: u8) {
        if can_sign_extend(imm) {
            self.formatter
                .one_byte_op_6(OP_GROUP1_EvIb, GROUP1_OP_ADD, dst);
            self.formatter.imm8(imm as _);
        } else {
            self.formatter
                .one_byte_op_6(OP_GROUP1_EvIz, GROUP1_OP_ADD, dst);
            self.formatter.imm32(imm as _);
        }
    }
    cfg_if::cfg_if! {
    if #[cfg(target_arch="x86_64")] {
        pub fn addq_rr(&mut self,src: u8,dst: u8) {
            self.formatter.one_byte_op64_2(OP_ADD_EvGv,src,dst);
        }

        pub fn addq_mr(&mut self,offset: i32,base: u8,dst: u8) {
            self.formatter.one_byte_op64_3(OP_ADD_GvEv,dst,base,offset);
        }
        pub fn addq_ir(&mut self,imm: i32,dst: u8) {
            if can_sign_extend(imm) {
                self.formatter.one_byte_op64_2(OP_GROUP1_EvIb,GROUP1_OP_ADD,dst);
                self.formatter.imm8(imm as _);
            } else {
                self.formatter.one_byte_op64_2(OP_GROUP1_EvIz,GROUP1_OP_ADD,dst);
                self.formatter.imm8(imm as _);
            }
        }

        pub fn addq_im(&mut self,imm: i32,offset: i32,base: u8) {
            if can_sign_extend(imm) {
                self.formatter.one_byte_op64_3(OP_GROUP1_EvIb,GROUP1_OP_ADD,base,offset);
                self.formatter.imm8(imm as _);
            } else {
                self.formatter.one_byte_op64_3(OP_GROUP1_EvIz,GROUP1_OP_ADD,base,offset);
                self.formatter.imm32(imm as _);
            }
        }
    }
    }
    pub fn andl_rr(&mut self, src: u8, dst: u8) {
        self.formatter.one_byte_op_6(OP_AND_EvGv, src, dst);
    }

    pub fn andl_mr(&mut self, offset: i32, base: u8, dst: u8) {
        self.formatter.one_byte_op_3(OP_AND_GvEv, dst, base, offset);
    }

    pub fn andl_rm(&mut self, src: u8, offset: i32, base: u8) {
        self.formatter.one_byte_op_3(OP_AND_EvGv, src, base, offset);
    }

    pub fn andl_ir(&mut self, imm: i32, dst: u8) {
        if can_sign_extend(imm) {
            self.formatter
                .one_byte_op_6(OP_GROUP1_EvIb, GROUP1_OP_AND, dst);
            self.formatter.imm8(imm as _);
        } else {
            self.formatter
                .one_byte_op_6(OP_GROUP1_EvIz, GROUP1_OP_AND, dst);
            self.formatter.imm32(imm);
        }
    }
    pub fn andl_im(&mut self, imm: i32, offset: i32, base: u8) {
        if can_sign_extend(imm) {
            self.formatter
                .one_byte_op_3(OP_GROUP1_EvIb, GROUP1_OP_AND, base, offset);
            self.formatter.imm8(imm as _);
        } else {
            self.formatter
                .one_byte_op_3(OP_GROUP1_EvIz, GROUP1_OP_AND, base, offset);
        }
    }
    cfg_if::cfg_if! {
        if #[cfg(target_arch="x86_64")] {
            pub fn andq_rr(&mut self,src: u8,dst: u8) {
                self.formatter.one_byte_op64_2(OP_AND_EvGv,src,dst);
            }

            pub fn andq_ir(&mut self,imm: i32,dst: u8) {
                self.formatter.one_byte_op64_2(OP_GROUP1_EvIz,GROUP1_OP_AND,dst);
                self.formatter.imm32(imm);
            }
        }
    }

    pub fn negl_r(&mut self, r: u8) {
        self.formatter.one_byte_op_6(OP_GROUP3_Ev, GROUP3_OP_NEG, r);
    }
    #[cfg(target_arch = "x86_64")]
    pub fn negq_r(&mut self, r: u8) {
        self.formatter
            .one_byte_op64_2(OP_GROUP3_Ev, GROUP3_OP_NEG, r);
    }

    pub fn notl_r(&mut self, r: u8) {
        self.formatter.one_byte_op_6(OP_GROUP3_Ev, GROUP3_OP_NOT, r);
    }

    pub fn orl_rr(&mut self, src: u8, dst: u8) {
        self.formatter.one_byte_op_6(OP_OR_EvGv, src, dst);
    }
    pub fn orl_ir(&mut self, imm: i32, dst: u8) {
        self.formatter
            .one_byte_op_6(OP_GROUP1_EvIz, GROUP1_OP_OR, dst);
        self.formatter.imm32(imm);
    }
    #[cfg(target_arch = "x86_64")]
    pub fn oqr_rr(&mut self, src: u8, dst: u8) {
        self.formatter.one_byte_op64_2(OP_OR_EvGv, src, dst);
    }

    #[cfg(target_arch = "x86_64")]
    pub fn orq_ir(&mut self, imm: i32, dst: u8) {
        self.formatter
            .one_byte_op64_2(OP_GROUP1_EvIz, GROUP1_OP_OR, dst);
        self.formatter.imm32(imm);
    }

    pub fn subl_rr(&mut self, src: u8, dst: u8) {
        self.formatter.one_byte_op_6(OP_SUB_EvGv, src, dst);
    }

    pub fn subl_ir(&mut self, imm: i32, dst: u8) {
        if can_sign_extend(imm) {
            self.formatter
                .one_byte_op_6(OP_GROUP1_EvIb, GROUP1_OP_SUB, dst);
            self.formatter.imm8(imm as _);
        } else {
            self.formatter
                .one_byte_op_6(OP_GROUP1_EvIz, GROUP1_OP_SUB, dst);
            self.formatter.imm8(imm as _);
        }
    }

    pub fn subl_im(&mut self, imm: i32, offset: i32, base: u8) {
        if can_sign_extend(imm) {
            self.formatter
                .one_byte_op_3(OP_GROUP1_EvIb, GROUP1_OP_SUB, base, offset);
            self.formatter.imm8(imm as _);
        } else {
            self.formatter
                .one_byte_op_3(OP_GROUP1_EvIz, GROUP1_OP_SUB, base, offset);
            self.formatter.imm32(imm as _);
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn movq_rr(&mut self, src: u8, dst: u8) {
        self.formatter.one_byte_op64_2(OP_MOV_EvGv, src, dst);
    }
}

pub const fn diff_between_labels(a: AssemblerLabel, b: AssemblerLabel) -> u32 {
    b.offset - a.offset
}
