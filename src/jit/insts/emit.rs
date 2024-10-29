use std::collections::HashMap;

use crate::{
    jit::{
        regalloc::{Register, REG_TEMP},
        utils::emit_mov_reg_to_reg,
        ValueType, X86JitCompiler,
    },
    module::insts::Instruction,
};

use anyhow::{anyhow, Result};
use monoasm::DestLabel;

impl X86JitCompiler<'_> {
    pub(crate) fn emit_asm(
        &mut self,
        func_index: u32,
        insts: &[Instruction],
        local_types: &[ValueType],
        stack_size: u64,
        else_labels: HashMap<usize, DestLabel>,
        end_labels: HashMap<usize, DestLabel>,
    ) -> Result<()> {
        let mut nbrtable = 0;
        for (i, inst) in insts.iter().enumerate() {
            match inst {
                Instruction::I32Const { value } => {
                    let reg = self.reg_allocator.next();
                    self.emit_mov_rawvalue_to_reg(*value as u64, reg.reg);
                }
                Instruction::Unreachable => {
                    self.emit_trap();
                }
                Instruction::Nop => {}
                Instruction::Block { ty } => {
                    let block_begin = self.jit.label();
                    let block_end = *end_labels
                        .get(&Self::find_matching_end_index(insts, i))
                        .expect("an matching end is needed");

                    self.emit_block(*ty, block_begin, block_end);
                }
                Instruction::Loop { ty } => {
                    let end_ind = Self::find_matching_end_index(insts, i);
                    let end_label = *end_labels.get(&end_ind).unwrap();
                    self.emit_loop(*ty, end_label);
                }
                Instruction::If { ty } => {
                    let else_ind = Self::find_closest_else_index(insts, i);
                    let else_label = else_ind.map(|ind| else_labels[&ind]);
                    let end_ind = Self::find_matching_end_index(insts, i);
                    let end_label = *end_labels.get(&end_ind).unwrap();

                    let cond = self.reg_allocator.pop_noopt();
                    self.emit_if(cond.reg, *ty, else_label, end_label);
                }
                Instruction::Else => {
                    let frame = self.control_flow_stack.back().unwrap();
                    let regalloc_snapshot = frame.entry_regalloc_snapshot.clone();
                    let end_label = frame.end_label;

                    self.emit_jmp(end_label);
                    self.emit_single_label(*else_labels.get(&i).unwrap());

                    // reset the register allocator to the snapshot in the else block
                    // to maintain a consistent view of the stack
                    self.reg_allocator = regalloc_snapshot;
                }
                Instruction::End => {
                    self.control_flow_stack.pop_back().unwrap();
                    let end_label = *end_labels.get(&i).unwrap();

                    self.emit_jmp(end_label);
                    self.emit_reg_reconciliation(end_label);
                    self.emit_single_label(end_label);
                }
                Instruction::Br { rel_depth } => {
                    self.emit_br(*rel_depth);
                }
                Instruction::BrIf { rel_depth } => {
                    let cond = self.reg_allocator.pop_noopt();
                    self.emit_br_if(cond.reg, *rel_depth);
                }
                Instruction::BrTable { table } => {
                    let index = self.reg_allocator.pop_noopt();
                    self.emit_br_table(index.reg, table, func_index, nbrtable);
                    nbrtable += 1;
                }
                Instruction::Return => {
                    self.emit_function_return(None, stack_size);
                }
                Instruction::Call { func_idx } => {
                    let nargs = self
                        .module
                        .borrow()
                        .get_func(*func_idx)
                        .unwrap()
                        .get_sig()
                        .params()
                        .len();
                    self.emit_mov_rawvalue_to_reg(*func_idx as u64, Register::Reg(REG_TEMP));
                    self.emit_call(REG_TEMP, nargs);
                }
                Instruction::CallIndirect {
                    type_index,
                    table_index,
                } => {
                    let callee_index_in_table = self.reg_allocator.pop_noopt();
                    self.emit_call_indirect(callee_index_in_table.reg, *type_index, *table_index);
                }
                Instruction::Drop => {
                    self.reg_allocator.pop_noopt();
                }
                Instruction::Select => {
                    let cond = self.reg_allocator.pop_noopt();
                    let b = self.reg_allocator.pop_noopt();
                    let a = self.reg_allocator.pop_noopt();
                    self.emit_select(a, cond, b, a);
                    self.reg_allocator.push(a);
                }
                Instruction::LocalGet { local_idx } => {
                    let dst = self.reg_allocator.next().reg;
                    self.emit_local_get(dst, *local_idx, local_types);
                }
                Instruction::LocalSet { local_idx } => {
                    let value = self.reg_allocator.pop_noopt();
                    let ty = local_types[*local_idx as usize];
                    self.emit_local_set(value.reg, *local_idx, ty);
                }
                Instruction::LocalTee { local_idx } => {
                    let value = self.reg_allocator.pop_noopt();
                    let ty = local_types[*local_idx as usize];
                    self.emit_local_tee(value.reg, *local_idx, ty);
                    self.reg_allocator.push(value);
                }
                Instruction::GlobalGet { global_idx } => {
                    let dst = self.reg_allocator.next().reg;
                    self.emit_global_get(dst, *global_idx);
                }
                Instruction::GlobalSet { global_idx } => {
                    let value = self.reg_allocator.pop_noopt();
                    self.emit_global_set(value.reg, *global_idx);
                }
                Instruction::I32Load { memarg } => {
                    let base = self.reg_allocator.pop_noopt();
                    let offset = memarg.offset;
                    let dst = self.reg_allocator.next().reg;
                    self.emit_load_mem(dst, base.reg, offset, 4, false);
                }
                Instruction::F64Load { memarg } => {
                    let base = self.reg_allocator.pop_noopt();
                    let offset = memarg.offset;
                    let dst = self.reg_allocator.next().reg;
                    self.emit_load_mem(dst, base.reg, offset, 8, false);
                }
                Instruction::I32Load8S { memarg } => {
                    let base = self.reg_allocator.pop_noopt();
                    let offset = memarg.offset;
                    let dst = self.reg_allocator.next().reg;
                    self.emit_load_mem(dst, base.reg, offset, 1, true);
                }
                Instruction::I32Load8U { memarg } => {
                    let base = self.reg_allocator.pop_noopt();
                    let offset = memarg.offset;
                    let dst = self.reg_allocator.next().reg;
                    self.emit_load_mem(dst, base.reg, offset, 1, false);
                }
                Instruction::I32Load16S { memarg } => {
                    let base = self.reg_allocator.pop_noopt();
                    let offset = memarg.offset;
                    let dst = self.reg_allocator.next().reg;
                    self.emit_load_mem(dst, base.reg, offset, 2, true);
                }
                Instruction::I32Load16U { memarg } => {
                    let base = self.reg_allocator.pop_noopt();
                    let offset = memarg.offset;
                    let dst = self.reg_allocator.next().reg;
                    self.emit_load_mem(dst, base.reg, offset, 2, false);
                }
                Instruction::I32Store { memarg } => {
                    let value = self.reg_allocator.pop_noopt();
                    let offset = memarg.offset;
                    let base = self.reg_allocator.pop_noopt();
                    self.emit_store_mem(base.reg, offset, value.reg, 4);
                }
                Instruction::F64Store { memarg } => {
                    let value = self.reg_allocator.pop_noopt();
                    let offset = memarg.offset;
                    let base = self.reg_allocator.pop_noopt();
                    self.emit_store_mem(base.reg, offset, value.reg, 8);
                }
                Instruction::I32Store8 { memarg } => {
                    let value = self.reg_allocator.pop_noopt();
                    let offset = memarg.offset;
                    let base = self.reg_allocator.pop_noopt();
                    self.emit_store_mem(base.reg, offset, value.reg, 1);
                }
                Instruction::I32Store16 { memarg } => {
                    let value = self.reg_allocator.pop_noopt();
                    let offset = memarg.offset;
                    let base = self.reg_allocator.pop_noopt();
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

                    let additional_pages = self.reg_allocator.pop_noopt();

                    // use a spill register to avoid aliasing
                    let dst = self.reg_allocator.new_spill(ValueType::I32);

                    self.emit_memory_grow(dst.reg, additional_pages.reg);
                }
                Instruction::F64Const { value } => {
                    let reg = self.reg_allocator.next_xmm();
                    self.emit_mov_rawvalue_to_reg(value.to_bits(), reg.reg);
                }
                Instruction::I32Unop(unop) => self.emit_i32_unop(unop),
                Instruction::I32Binop(binop) => self.emit_i32_binop(binop),
                Instruction::F64Unop(unop) => self.emit_f64_unop(unop),
                Instruction::F64Binop(binop) => self.emit_f64_binop(binop),
            }
        }

        Ok(())
    }

    fn find_closest_else_index(insts: &[Instruction], start: usize) -> Option<usize> {
        let end_index = Self::find_matching_end_index(insts, start);
        for (i, inst) in insts.iter().enumerate() {
            if i < start {
                continue;
            }
            if let Instruction::Else = inst {
                if i < end_index {
                    return Some(i);
                } else {
                    return None;
                }
            }
        }

        None
    }

    fn find_matching_end_index(insts: &[Instruction], start: usize) -> usize {
        let mut depth = 0;
        for (i, inst) in insts.iter().enumerate() {
            if i < start {
                continue;
            }

            if Instruction::is_control_block_start(inst) {
                depth += 1;
            } else if Instruction::is_control_block_end(inst) {
                depth -= 1;
            }

            if depth == 0 {
                return i;
            }
        }

        panic!("no matching end found");
    }

    fn emit_trap(&mut self) {
        self.emit_jmp(self.trap_label);
    }

    fn emit_reg_reconciliation(&mut self, end_label: DestLabel) {
        let infos = self
            .reg_reconcile_info
            .iter()
            .filter(|info| info.target_end_label == end_label)
            .cloned()
            .collect::<Vec<_>>();

        for info in infos {
            self.emit_single_label(info.reconcile_start_label);
            // reconciliation
            let branch_point_regvec = info.regalloc_snapshot.get_vec().clone();
            let now_regvec = self.reg_allocator.get_vec().clone();

            for i in 0..branch_point_regvec.len() {
                let branch_point_reg = branch_point_regvec[branch_point_regvec.len() - 1 - i];
                let now_reg = now_regvec[now_regvec.len() - 1 - i];
                if branch_point_reg != now_reg {
                    emit_mov_reg_to_reg(&mut self.jit, now_reg.reg, branch_point_reg.reg);
                }
            }

            self.emit_jmp(end_label);
        }
    }
}
