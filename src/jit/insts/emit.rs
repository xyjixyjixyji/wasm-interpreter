use std::{collections::HashMap, rc::Rc};

use crate::{
    jit::{ValueType, X86JitCompiler},
    module::{insts::Instruction, wasm_module::WasmModule},
};

use anyhow::{anyhow, Result};
use debug_cell::RefCell;
use monoasm::*;
use monoasm_macro::monoasm;

impl X86JitCompiler {
    pub(crate) fn emit_asm(
        &mut self,
        module: Rc<RefCell<WasmModule>>,
        insts: &[Instruction],
        local_types: &[ValueType],
        func_to_label: &HashMap<usize, DestLabel>,
    ) -> Result<()> {
        for inst in insts {
            match inst {
                Instruction::I32Const { value } => {
                    let reg = self.reg_allocator.next();
                    self.mov_i32_to_reg(*value, reg.reg);
                }
                Instruction::Unreachable => {
                    self.emit_trap();
                }
                Instruction::Nop => {}
                Instruction::Block { ty } => todo!(),
                Instruction::Loop { ty } => todo!(),
                Instruction::If { ty } => todo!(),
                Instruction::Else => todo!(),
                Instruction::End => {}
                Instruction::Br { rel_depth } => todo!(),
                Instruction::BrIf { rel_depth } => todo!(),
                Instruction::BrTable { table } => todo!(),
                Instruction::Return => {
                    monoasm!(
                        &mut self.jit,
                        ret;
                    );
                }
                Instruction::Call { func_idx } => {
                    let label = func_to_label.get(&(*func_idx as usize)).unwrap();
                    let callee_func = module.borrow().get_func(*func_idx).unwrap().clone();

                    // compile the call instruction
                    self.compile_call(&callee_func, *label);
                }
                Instruction::CallIndirect {
                    type_index,
                    table_index,
                } => todo!(),
                Instruction::Drop => {
                    self.reg_allocator.pop();
                }
                Instruction::Select => {
                    let cond = self.reg_allocator.pop();
                    let b = self.reg_allocator.pop();
                    let a = self.reg_allocator.pop();
                    self.compile_select(a, cond, b, a);
                    self.reg_allocator.push(a);
                }
                Instruction::LocalGet { local_idx } => {
                    let dst = self.reg_allocator.next().reg;
                    self.compile_local_get(dst, *local_idx, &local_types);
                }
                Instruction::LocalSet { local_idx } => {
                    let value = self.reg_allocator.pop();
                    self.compile_local_set(value.reg, *local_idx, &local_types);
                }
                Instruction::LocalTee { local_idx } => todo!(),
                Instruction::GlobalGet { global_idx } => todo!(),
                Instruction::GlobalSet { global_idx } => todo!(),
                Instruction::I32Load { memarg } => {
                    let base = self.reg_allocator.pop();
                    let offset = memarg.offset;
                    let dst = self.reg_allocator.next().reg;
                    self.compile_load(dst, base.reg, offset, 4);
                }
                Instruction::F64Load { memarg } => {
                    let base = self.reg_allocator.pop();
                    let offset = memarg.offset;
                    let dst = self.reg_allocator.next().reg;
                    self.compile_load(dst, base.reg, offset, 8);
                }
                Instruction::I32Load8S { memarg } => todo!(),
                Instruction::I32Load8U { memarg } => todo!(),
                Instruction::I32Load16S { memarg } => todo!(),
                Instruction::I32Load16U { memarg } => todo!(),
                Instruction::I32Store { memarg } => {
                    let value = self.reg_allocator.pop();
                    let offset = memarg.offset;
                    let base = self.reg_allocator.pop();
                    self.compile_store(base.reg, offset, value.reg, 4);
                }
                Instruction::F64Store { memarg } => {
                    let value = self.reg_allocator.pop();
                    let offset = memarg.offset;
                    let base = self.reg_allocator.pop();
                    self.compile_store(base.reg, offset, value.reg, 8);
                }
                Instruction::I32Store8 { memarg } => {
                    let value = self.reg_allocator.pop();
                    let offset = memarg.offset;
                    let base = self.reg_allocator.pop();
                    self.compile_store(base.reg, offset, value.reg, 1);
                }
                Instruction::I32Store16 { memarg } => {
                    let value = self.reg_allocator.pop();
                    let offset = memarg.offset;
                    let base = self.reg_allocator.pop();
                    self.compile_store(base.reg, offset, value.reg, 2);
                }
                Instruction::MemorySize { mem } => {
                    if *mem != 0 {
                        return Err(anyhow!("memory.size: invalid memory index"));
                    }

                    let dst = self.reg_allocator.next();
                    self.store_mem_page_size(dst.reg);
                }
                Instruction::MemoryGrow { mem } => {
                    if *mem != 0 {
                        return Err(anyhow!("memory.size: invalid memory index"));
                    }

                    let additional_pages = self.reg_allocator.pop();

                    let old_mem_size = self.reg_allocator.new_spill(ValueType::I32); // avoid aliasing
                    self.linear_mem
                        .read_memory_size_in_page(&mut self.jit, old_mem_size.reg);

                    self.compile_memory_grow(additional_pages.reg);
                }
                Instruction::F64Const { value } => {
                    let reg = self.reg_allocator.next_xmm();
                    self.mov_f64_to_reg(*value, reg.reg);
                }
                Instruction::I32Unop(_) => todo!(),
                Instruction::I32Binop(binop) => {
                    self.compile_i32_binop(binop);
                }
                Instruction::F64Unop(_) => todo!(),
                Instruction::F64Binop(binop) => {
                    self.compile_f64_binop(binop);
                }
            }
        }

        Ok(())
    }

    fn emit_trap(&mut self) {
        let trap_label = self.trap_label;
        monoasm!(
            &mut self.jit,
            jmp trap_label;
        );
    }
}
