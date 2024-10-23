use crate::{
    jit::{ValueType, X86JitCompiler},
    module::insts::Instruction,
};

use anyhow::{anyhow, Result};
use monoasm_macro::monoasm;

impl X86JitCompiler<'_> {
    pub(crate) fn emit_asm(
        &mut self,
        insts: &[Instruction],
        local_types: &[ValueType],
    ) -> Result<()> {
        for inst in insts {
            match inst {
                Instruction::I32Const { value } => {
                    let reg = self.reg_allocator.next();
                    self.emit_mov_i32_to_reg(*value, reg.reg);
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
                    let callee_func = self.module.borrow().get_func(*func_idx).unwrap().clone();
                    self.emit_call(&callee_func, *func_idx);
                }
                Instruction::CallIndirect {
                    type_index,
                    table_index,
                } => {
                    let callee_index_in_table = self.reg_allocator.pop();
                    self.emit_call_indirect(callee_index_in_table.reg, *type_index, *table_index);
                }
                Instruction::Drop => {
                    self.reg_allocator.pop();
                }
                Instruction::Select => {
                    let cond = self.reg_allocator.pop();
                    let b = self.reg_allocator.pop();
                    let a = self.reg_allocator.pop();
                    self.emit_select(a, cond, b, a);
                    self.reg_allocator.push(a);
                }
                Instruction::LocalGet { local_idx } => {
                    let dst = self.reg_allocator.next().reg;
                    self.emit_local_get(dst, *local_idx, local_types);
                }
                Instruction::LocalSet { local_idx } => {
                    let value = self.reg_allocator.pop();
                    self.emit_local_set(value.reg, *local_idx, local_types);
                }
                Instruction::LocalTee { local_idx } => todo!(),
                Instruction::GlobalGet { global_idx } => todo!(),
                Instruction::GlobalSet { global_idx } => todo!(),
                Instruction::I32Load { memarg } => {
                    let base = self.reg_allocator.pop();
                    let offset = memarg.offset;
                    let dst = self.reg_allocator.next().reg;
                    self.emit_load_mem(dst, base.reg, offset, 4);
                }
                Instruction::F64Load { memarg } => {
                    let base = self.reg_allocator.pop();
                    let offset = memarg.offset;
                    let dst = self.reg_allocator.next().reg;
                    self.emit_load_mem(dst, base.reg, offset, 8);
                }
                Instruction::I32Load8S { memarg } => todo!(),
                Instruction::I32Load8U { memarg } => todo!(),
                Instruction::I32Load16S { memarg } => todo!(),
                Instruction::I32Load16U { memarg } => todo!(),
                Instruction::I32Store { memarg } => {
                    let value = self.reg_allocator.pop();
                    let offset = memarg.offset;
                    let base = self.reg_allocator.pop();
                    self.emit_store_mem(base.reg, offset, value.reg, 4);
                }
                Instruction::F64Store { memarg } => {
                    let value = self.reg_allocator.pop();
                    let offset = memarg.offset;
                    let base = self.reg_allocator.pop();
                    self.emit_store_mem(base.reg, offset, value.reg, 8);
                }
                Instruction::I32Store8 { memarg } => {
                    let value = self.reg_allocator.pop();
                    let offset = memarg.offset;
                    let base = self.reg_allocator.pop();
                    self.emit_store_mem(base.reg, offset, value.reg, 1);
                }
                Instruction::I32Store16 { memarg } => {
                    let value = self.reg_allocator.pop();
                    let offset = memarg.offset;
                    let base = self.reg_allocator.pop();
                    self.emit_store_mem(base.reg, offset, value.reg, 2);
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
                    // use a spill register to avoid aliasing
                    let old_mem_size = self.reg_allocator.new_spill(ValueType::I32);

                    self.linear_mem
                        .read_memory_size_in_page(&mut self.jit, old_mem_size.reg);

                    self.emit_memory_grow(additional_pages.reg);
                }
                Instruction::F64Const { value } => {
                    let reg = self.reg_allocator.next_xmm();
                    self.emit_mov_f64_to_reg(*value, reg.reg);
                }
                Instruction::I32Unop(_) => todo!(),
                Instruction::I32Binop(binop) => {
                    self.emit_i32_binop(binop);
                }
                Instruction::F64Unop(_) => todo!(),
                Instruction::F64Binop(binop) => {
                    self.emit_f64_binop(binop);
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
