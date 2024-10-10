use std::collections::HashMap;
use std::rc::Rc;

use super::regalloc::{Register, X64Register, X86RegisterAllocator};
use super::WasmJitCompiler;
use crate::jit::I32ReturnFunc;
use crate::module::components::FuncDecl;
use crate::module::insts::Instruction;
use crate::module::wasm_module::WasmModule;

use anyhow::Result;
use debug_cell::RefCell;
use monoasm::{CodePtr, DestLabel, Disp, Imm, JitMemory, Reg, Rm, Scale};
use monoasm_macro::monoasm;

// Jit compile through abstract interpretation
pub struct X86JitCompiler {
    reg_allocator: X86RegisterAllocator,
    jit: JitMemory,
}

impl X86JitCompiler {
    pub fn new() -> Self {
        Self {
            reg_allocator: X86RegisterAllocator::new(),
            jit: JitMemory::new(),
        }
    }
}

impl WasmJitCompiler for X86JitCompiler {
    fn compile(&mut self, module: Rc<RefCell<WasmModule>>) -> Result<CodePtr> {
        // make labels for all functions
        let mut func_to_label = HashMap::new();
        for (i, _) in module.borrow().get_funcs().iter().enumerate() {
            let label = self.jit.label();
            func_to_label.insert(i, label);
        }

        for (i, fdecl) in module.borrow().get_funcs().iter().enumerate() {
            let func_begin_label = func_to_label.get(&i).unwrap();
            self.compile_func(fdecl, *func_begin_label, &func_to_label)?;
        }

        let main_index = module.borrow().get_main_index().unwrap();
        let main_label = func_to_label.get(&(main_index as usize)).unwrap();

        self.jit.finalize();

        let codeptr = self.jit.get_label_u64(*main_label);

        Ok(unsafe { std::mem::transmute::<u64, CodePtr>(codeptr) })
    }
}

impl X86JitCompiler {
    fn compile_func(
        &mut self,
        fdecl: &FuncDecl,
        func_begin_label: DestLabel,
        func_to_label: &HashMap<usize, DestLabel>,
    ) -> Result<()> {
        monoasm!(
            &mut self.jit,
            func_begin_label:
        );

        for inst in fdecl.get_insts() {
            match inst {
                Instruction::I32Const { value } => {
                    let reg = self.reg_allocator.next();
                    self.mov_i32_to_reg(*value, reg);
                }
                Instruction::Unreachable => todo!(),
                Instruction::Nop => todo!(),
                Instruction::Block { ty } => todo!(),
                Instruction::Loop { ty } => todo!(),
                Instruction::If { ty } => todo!(),
                Instruction::Else => todo!(),
                Instruction::End => {}
                Instruction::Br { rel_depth } => todo!(),
                Instruction::BrIf { rel_depth } => todo!(),
                Instruction::BrTable { table } => todo!(),
                Instruction::Return => todo!(),
                Instruction::Call { func_idx } => todo!(),
                Instruction::CallIndirect {
                    type_index,
                    table_index,
                } => todo!(),
                Instruction::Drop => todo!(),
                Instruction::Select => todo!(),
                Instruction::LocalGet { local_idx } => todo!(),
                Instruction::LocalSet { local_idx } => todo!(),
                Instruction::LocalTee { local_idx } => todo!(),
                Instruction::GlobalGet { global_idx } => todo!(),
                Instruction::GlobalSet { global_idx } => todo!(),
                Instruction::I32Load { memarg } => todo!(),
                Instruction::F64Load { memarg } => todo!(),
                Instruction::I32Load8S { memarg } => todo!(),
                Instruction::I32Load8U { memarg } => todo!(),
                Instruction::I32Load16S { memarg } => todo!(),
                Instruction::I32Load16U { memarg } => todo!(),
                Instruction::I32Store { memarg } => todo!(),
                Instruction::F64Store { memarg } => todo!(),
                Instruction::I32Store8 { memarg } => todo!(),
                Instruction::I32Store16 { memarg } => todo!(),
                Instruction::MemorySize { mem } => todo!(),
                Instruction::MemoryGrow { mem } => todo!(),
                Instruction::F64Const { value } => todo!(),
                Instruction::I32Unop(_) => todo!(),
                Instruction::I32Binp(_) => todo!(),
                Instruction::F64Unop(_) => todo!(),
                Instruction::F64Binop(_) => todo!(),
            }
        }

        // return...
        monoasm!(
            &mut self.jit,
            ret;
        );
        Ok(())
    }
}

impl X86JitCompiler {
    fn mov_i32_to_reg(&mut self, value: i32, reg: Register) {
        match reg {
            Register::Reg(r) => {
                monoasm!(
                    &mut self.jit,
                    movq R(r.as_index()), (value);
                );
            }
            Register::Stack(offset) => {
                monoasm!(
                    &mut self.jit,
                    movq [rsp + (offset)], (value);
                );
            }
        }
    }
}
