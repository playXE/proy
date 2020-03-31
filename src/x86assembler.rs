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

pub struct X86Assembler {
    buffer: AssemblerBuffer,
}

impl X86Assembler {
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
    2
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

ESCAPE_DD_FSTP_doubleReal = 3}
