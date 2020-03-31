extern crate proy;
use capstone::prelude::*;
use proy::mem;
use proy::x86assembler::*;
fn main() {
    let mut asm = X86Assembler::new();
    asm.movq_rr(X86Gpr::Esi as _, X86Gpr::Eax as _);
    asm.addq_rr(X86Gpr::Edi as u8, X86Gpr::Eax as u8);
    asm.ret();
    let f: fn(i64, i64) -> i64 =
        unsafe { std::mem::transmute(asm.formatter.executable_readable()) };
    println!("{}", f(2, 3));
    let cs = Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .syntax(arch::x86::ArchSyntax::Att)
        .detail(true)
        .build()
        .expect("Failed to create Capstone object");

    let insns = cs.disasm_all(asm.code(), 0x0).unwrap();
    for i in insns.iter() {
        println!("{}", i);
    }
}
