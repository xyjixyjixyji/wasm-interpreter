use std::collections::HashMap;
use std::rc::Rc;

use super::regalloc::{
    Register, X64Register, X86RegisterAllocator, REG_MEMORY_BASE, REG_TEMP, REG_TEMP2,
};
use super::{JitLinearMemory, WasmJitCompiler};
use crate::module::components::FuncDecl;
use crate::module::insts::{I32Binop, Instruction};
use crate::module::wasm_module::WasmModule;
use crate::vm::WASM_DEFAULT_PAGE_SIZE_BYTE;

use anyhow::Result;
use debug_cell::RefCell;
use monoasm::{CodePtr, DestLabel, Disp, Imm, JitMemory, Reg, Rm, Scale};
use monoasm_macro::monoasm;

// Jit compile through abstract interpretation
pub struct X86JitCompiler {
    reg_allocator: X86RegisterAllocator,
    jit: JitMemory,
    trap_label: DestLabel,
    jit_linear_mem: JitLinearMemory,
}

impl X86JitCompiler {
    pub fn new() -> Self {
        let mut jit = JitMemory::new();
        let trap_label = jit.label();

        let mut compiler = Self {
            reg_allocator: X86RegisterAllocator::new(),
            jit,
            trap_label,
            jit_linear_mem: JitLinearMemory::new(),
        };

        compiler.setup_trap_entry();

        compiler
    }
}

impl WasmJitCompiler for X86JitCompiler {
    fn compile(
        &mut self,
        module: Rc<RefCell<WasmModule>>,
        initial_mem_size_in_byte: u64,
    ) -> Result<CodePtr> {
        // make labels for all functions
        let mut func_to_label = HashMap::new();
        for (i, _) in module.borrow().get_funcs().iter().enumerate() {
            let label = self.jit.label();
            func_to_label.insert(i, label);
        }

        // setup vm_entry
        let main_index = module.borrow().get_main_index().unwrap();
        let main_label = func_to_label.get(&(main_index as usize)).unwrap();
        let vm_entry_label = self.setup_vm_entry(*main_label, initial_mem_size_in_byte);

        // compile all functions
        for (i, fdecl) in module.borrow().get_funcs().iter().enumerate() {
            let func_begin_label = func_to_label.get(&i).unwrap();
            self.compile_func(fdecl, *func_begin_label, &func_to_label)?;
        }

        self.jit.finalize();

        // return vm_entry address
        let codeptr = self.jit.get_label_u64(vm_entry_label);

        log::debug!("\n{}", self.jit.dump_code().unwrap());
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

        let stack_size = self.get_stack_size_in_byte(fdecl);
        self.prologue(stack_size);

        for inst in fdecl.get_insts() {
            match inst {
                Instruction::I32Const { value } => {
                    let reg = self.reg_allocator.next();
                    self.mov_i32_to_reg(*value, reg);
                }
                Instruction::Unreachable => {
                    self.trap();
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
                Instruction::Return => todo!(),
                // TODO: save all caller saved registers
                Instruction::Call { func_idx } => todo!(),
                Instruction::CallIndirect {
                    type_index,
                    table_index,
                } => todo!(),
                Instruction::Drop => {
                    self.reg_allocator.pop();
                }
                Instruction::Select => todo!(),
                Instruction::LocalGet { local_idx } => todo!(),
                Instruction::LocalSet { local_idx } => todo!(),
                Instruction::LocalTee { local_idx } => todo!(),
                Instruction::GlobalGet { global_idx } => todo!(),
                Instruction::GlobalSet { global_idx } => todo!(),
                Instruction::I32Load { memarg } => {
                    let base = self.reg_allocator.pop();
                    let offset = memarg.offset;
                    let dst = self.reg_allocator.next();
                    self.compile_i32_load(dst, base, offset);
                }
                Instruction::F64Load { memarg } => todo!(),
                Instruction::I32Load8S { memarg } => todo!(),
                Instruction::I32Load8U { memarg } => todo!(),
                Instruction::I32Load16S { memarg } => todo!(),
                Instruction::I32Load16U { memarg } => todo!(),
                Instruction::I32Store { memarg } => {
                    let value = self.reg_allocator.pop();
                    let offset = memarg.offset;
                    let base = self.reg_allocator.pop();
                    self.compile_i32_store(base, offset, value);
                }
                Instruction::F64Store { memarg } => todo!(),
                Instruction::I32Store8 { memarg } => todo!(),
                Instruction::I32Store16 { memarg } => todo!(),
                Instruction::MemorySize { mem } => todo!(),
                Instruction::MemoryGrow { mem } => todo!(),
                Instruction::F64Const { value } => {
                    let reg = self.reg_allocator.next_xmm();
                    self.mov_f64_to_reg(*value, reg);
                }
                Instruction::I32Unop(_) => todo!(),
                Instruction::I32Binop(binop) => {
                    self.compile_i32_binop(binop);
                }
                Instruction::F64Unop(_) => todo!(),
                Instruction::F64Binop(_) => todo!(),
            }
        }

        // return...
        let stack_top = self.reg_allocator.top();
        self.mov_reg_to_reg(Register::Reg(X64Register::Rax), stack_top);

        self.epilogue(stack_size);
        monoasm!(
            &mut self.jit,
            ret;
        );

        Ok(())
    }
}

impl X86JitCompiler {
    fn compile_i32_load(&mut self, dst: Register, base: Register, offset: u32) {
        // 1. if out of bounds, trap
        let width = 4; // i32 is 4 bytes
        self.check_memory_bounds(base, offset, width);

        // 2. load the result into dst
        monoasm!(
            &mut self.jit,
            subq R(REG_TEMP2.as_index()), (width); // <-- reg_temp2 = effective_addr
            addq R(REG_TEMP2.as_index()), R(REG_MEMORY_BASE.as_index()); // <-- reg_temp2 = reg_memory_base + effective_addr
            movq R(REG_TEMP.as_index()), [R(REG_TEMP2.as_index())]; // <-- reg_temp = *reg_temp2
        );

        self.mov_reg_to_reg(dst, Register::Reg(REG_TEMP));
    }

    fn compile_i32_store(&mut self, base: Register, offset: u32, value: Register) {
        // 1. if out of bounds, trap
        let width = 4; // i32 is 4 bytes
        self.check_memory_bounds(base, offset, width);

        // 2. store the value to dst
        monoasm!(
            &mut self.jit,
            subq R(REG_TEMP2.as_index()), (width); // <-- reg_temp2 = effective_addr
            addq R(REG_TEMP2.as_index()), R(REG_MEMORY_BASE.as_index()); // <-- reg_temp2 = reg_memory_base + effective_addr
        );

        self.mov_reg_to_reg(Register::Reg(REG_TEMP), value); // <-- reg_temp = value

        monoasm!(
            &mut self.jit,
            movq [R(REG_TEMP2.as_index())], R(REG_TEMP.as_index());
        );
    }
}

impl X86JitCompiler {
    // jit compile *a = a op b*
    fn compile_i32_binop(&mut self, binop: &I32Binop) {
        let b = self.reg_allocator.pop();
        let a = self.reg_allocator.pop();

        self.mov_reg_to_reg(Register::Reg(REG_TEMP), a);
        self.mov_reg_to_reg(Register::Reg(REG_TEMP2), b);

        match binop {
            I32Binop::Eq => {
                monoasm!(
                    &mut self.jit,
                    cmpq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    seteq R(REG_TEMP.as_index()); // a = a == b
                );
            }
            I32Binop::Ne => {
                monoasm!(
                    &mut self.jit,
                    cmpq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    setne R(REG_TEMP.as_index()); // a = a != b
                );
            }
            I32Binop::LtS => {
                monoasm!(
                    &mut self.jit,
                    cmpq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    sets R(REG_TEMP.as_index()); // a = a < b
                );
            }
            I32Binop::LtU => {
                monoasm!(
                    &mut self.jit,
                    cmpq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    setb R(REG_TEMP.as_index()); // a = a < b
                );
            }
            I32Binop::GtS => {
                monoasm!(
                    &mut self.jit,
                    cmpq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    setgt R(REG_TEMP.as_index()); // a = a > b
                );
            }
            I32Binop::GtU => {
                monoasm!(
                    &mut self.jit,
                    cmpq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    seta R(REG_TEMP.as_index()); // a = a > b
                );
            }
            I32Binop::LeS => {
                monoasm!(
                    &mut self.jit,
                    cmpq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    setle R(REG_TEMP.as_index()); // a = a <= b
                );
            }
            I32Binop::LeU => {
                monoasm!(
                    &mut self.jit,
                    cmpq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    setbe R(REG_TEMP.as_index()); // a = a <= b
                );
            }
            I32Binop::GeS => {
                monoasm!(
                    &mut self.jit,
                    cmpq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    setge R(REG_TEMP.as_index()); // a = a >= b
                );
            }
            I32Binop::GeU => {
                monoasm!(
                    &mut self.jit,
                    cmpq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index());
                    movq R(REG_TEMP.as_index()), (0);
                    setae R(REG_TEMP.as_index()); // a = a >= b
                );
            }
            I32Binop::Add => {
                monoasm!(
                    &mut self.jit,
                    addq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index()); // a = a + b
                );
            }
            I32Binop::Sub => {
                monoasm!(
                    &mut self.jit,
                    subq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index()); // a = a - b
                );
            }
            I32Binop::Mul => {
                monoasm!(
                    &mut self.jit,
                    imul R(REG_TEMP.as_index()), R(REG_TEMP2.as_index()); // a = a * b
                );
            }
            I32Binop::DivS => todo!(),
            I32Binop::DivU => todo!(),
            I32Binop::RemS => todo!(),
            I32Binop::RemU => todo!(),
            I32Binop::And => {
                monoasm!(
                    &mut self.jit,
                    andq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index()); // a = a & b
                );
            }
            I32Binop::Or => {
                monoasm!(
                    &mut self.jit,
                    orq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index()); // a = a | b
                );
            }
            I32Binop::Xor => {
                monoasm!(
                    &mut self.jit,
                    xorq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index()); // a = a ^ b
                );
            }
            I32Binop::Shl => todo!(),
            I32Binop::ShrS => todo!(),
            I32Binop::ShrU => todo!(),
            I32Binop::Rotl => todo!(),
            I32Binop::Rotr => todo!(),
        }

        self.mov_reg_to_reg(a, Register::Reg(REG_TEMP));
        self.reg_allocator.push(a);
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
            Register::FpReg(_) => panic!("invalid mov for i32 to fp reg"),
        }
    }

    fn mov_f64_to_reg(&mut self, value: f64, reg: Register) {
        let bits = value.to_bits();
        match reg {
            Register::FpReg(r) => {
                monoasm!(
                    &mut self.jit,
                    movq R(REG_TEMP.as_index()), (bits);
                    movq xmm(r.as_index()), R(REG_TEMP.as_index());
                );
            }
            Register::Stack(offset) => {
                monoasm!(
                    &mut self.jit,
                    movq [rsp + (offset)], (bits);
                );
            }
            _ => panic!("invalid mov for f32 to normal reg"),
        }
    }

    fn mov_reg_to_reg(&mut self, dst: Register, src: Register) {
        if dst == src {
            return;
        }

        match (dst, src) {
            (Register::Stack(o_dst), Register::Stack(o_src)) => {
                monoasm!(
                    &mut self.jit,
                    movq R(REG_TEMP.as_index()), [rsp + (o_src)];
                    movq [rsp + (o_dst)], R(REG_TEMP.as_index());
                );
            }
            (Register::Reg(r_dst), Register::Stack(o_src)) => {
                monoasm!(
                    &mut self.jit,
                    movq R(r_dst.as_index()), [rsp + (o_src)];
                );
            }
            (Register::FpReg(fpr_dst), Register::Stack(o_src)) => {
                monoasm!(
                    &mut self.jit,
                    movq xmm(fpr_dst.as_index()), [rsp + (o_src)];
                );
            }
            (Register::Reg(r_dst), Register::Reg(r_src)) => {
                monoasm!(
                    &mut self.jit,
                    movq R(r_dst.as_index()), R(r_src.as_index());
                );
            }
            (Register::Reg(r_dst), Register::FpReg(fpr_src)) => {
                monoasm!(
                    &mut self.jit,
                    movq R(r_dst.as_index()), xmm(fpr_src.as_index());
                );
            }
            (Register::FpReg(fpr_dst), Register::Reg(r_src)) => {
                monoasm!(
                    &mut self.jit,
                    movq xmm(fpr_dst.as_index()), R(r_src.as_index());
                );
            }
            (Register::FpReg(fpr_dst), Register::FpReg(fpr_src)) => {
                monoasm!(
                    &mut self.jit,
                    movq xmm(fpr_dst.as_index()), xmm(fpr_src.as_index());
                );
            }
            (Register::Stack(o_dst), Register::Reg(r_src)) => {
                monoasm!(
                    &mut self.jit,
                    movq [rsp + (o_dst)], R(r_src.as_index());
                );
            }
            (Register::Stack(o_dst), Register::FpReg(fpr_src)) => {
                monoasm!(
                    &mut self.jit,
                    movq [rsp + (o_dst)], xmm(fpr_src.as_index());
                );
            }
        }
    }

    fn setup_trap_entry(&mut self) -> DestLabel {
        let trap_label = self.trap_label;
        monoasm!(
            &mut self.jit,
            trap_label:
                movq rax, 0;
                movq [rax], 1;
        );

        trap_label
    }

    fn setup_vm_entry(
        &mut self,
        main_label: DestLabel,
        initial_mem_size_in_byte: u64,
    ) -> DestLabel {
        let vm_entry_label = self.jit.label();
        monoasm!(
            &mut self.jit,
            vm_entry_label:
        );

        self.jit_linear_mem
            .save_base(&mut self.jit, initial_mem_size_in_byte);

        let npages = initial_mem_size_in_byte / WASM_DEFAULT_PAGE_SIZE_BYTE as u64;
        self.jit_linear_mem
            .save_size_in_pages(&mut self.jit, npages);

        // todo: setup locals

        monoasm!(
            &mut self.jit,
            jmp main_label;
        );

        vm_entry_label
    }

    fn trap(&mut self) {
        let trap_label = self.trap_label;
        monoasm!(
            &mut self.jit,
            jmp trap_label;
        );
    }

    fn prologue(&mut self, stack_size: u64) {
        // NOTE: on x86-64 linux, xmms are temporary registers
        // so we don't need to save and restore them
        monoasm!(
            &mut self.jit,
            pushq rbp;
            movq rbp, rsp;
            subq rsp, (stack_size);
            pushq rbx;
            pushq r12;
            pushq r13;
            pushq r14;
            pushq r15;
        );
    }

    fn epilogue(&mut self, stack_size: u64) {
        // NOTE: on x86-64 linux, xmms are temporary registers
        // so we don't need to save and restore them
        monoasm!(
            &mut self.jit,
            popq r15;
            popq r14;
            popq r13;
            popq r12;
            popq rbx;
            addq rsp, (stack_size);
            popq rbp;
        );
    }

    /// Check if the memory access is out of bounds, and jump to trap if so.
    ///
    /// Note that after this function:
    /// - REG_TEMP will store the memory size in bytes
    /// - REG_TEMP2 will store the effective address + width
    fn check_memory_bounds(&mut self, base: Register, offset: u32, width: u32) {
        let trap_label = self.trap_label;
        let mem_size_addr = self.jit_linear_mem.get_mem_size_addr();
        monoasm!(
            &mut self.jit,
            movq R(REG_TEMP.as_index()), (mem_size_addr);
            movq R(REG_TEMP.as_index()), [R(REG_TEMP.as_index())]; // <-- reg_temp = mem_size_in_page
            movq R(REG_TEMP2.as_index()), (WASM_DEFAULT_PAGE_SIZE_BYTE);
            imul R(REG_TEMP.as_index()), R(REG_TEMP2.as_index()); // <-- reg_temp = mem_size_in_byte
        );
        self.mov_reg_to_reg(Register::Reg(REG_TEMP2), base); // <-- reg_temp2 = base
        monoasm!(
            &mut self.jit,
            addq R(REG_TEMP2.as_index()), (offset);
            addq R(REG_TEMP2.as_index()), (width); // <-- reg_temp2 = effective_addr + width
            cmpq R(REG_TEMP.as_index()), R(REG_TEMP2.as_index());
            jb trap_label; // jump if mem_size < effective_addr + width
        );
    }
}

impl X86JitCompiler {
    // Get the stack size usage of the function, used for stack allocation
    // We get only an upper bound approximate, since we don't want too much overhead
    fn get_stack_size_in_byte(&self, fdecl: &FuncDecl) -> u64 {
        let nlocals = fdecl.get_pure_locals().len() as u64;

        let mut max_stack_depth: u64 = 0;
        let mut current_stack_depth: u64 = 0;
        let mut block_stack = Vec::new();

        let insts = fdecl.get_insts();
        let mut pc = 0;
        while pc < insts.len() {
            let inst = &insts[pc];
            match inst {
                // Constants push a value onto the stack
                Instruction::I32Const { .. } | Instruction::F64Const { .. } => {
                    current_stack_depth += 1;
                }

                // Unreachable instruction; for approximation, reset stack depth
                Instruction::Unreachable => {
                    current_stack_depth = 0;
                }

                // No operation; stack depth remains the same
                Instruction::Nop => {}

                // Drop pops one value from the stack
                Instruction::Drop => {
                    current_stack_depth = current_stack_depth.saturating_sub(1);
                }

                // Binary operations pop two values and push one; net effect is -1
                Instruction::I32Binop(_) | Instruction::F64Binop(_) => {
                    current_stack_depth = current_stack_depth.saturating_sub(1);
                }

                // Unary operations consume one value and produce one; net effect is 0
                Instruction::I32Unop(_) | Instruction::F64Unop(_) => {}

                // Block, Loop, If: push current stack depth onto block stack
                Instruction::Block { .. } | Instruction::Loop { .. } | Instruction::If { .. } => {
                    block_stack.push(current_stack_depth);
                }

                // Else: reset stack depth to the depth at the start of the block
                Instruction::Else => {
                    if let Some(depth_at_if) = block_stack.last().cloned() {
                        current_stack_depth = depth_at_if;
                    }
                }

                // End: pop from block stack and take the maximum of current and block start depth
                Instruction::End => {
                    if let Some(depth_at_block_start) = block_stack.pop() {
                        current_stack_depth =
                            std::cmp::max(current_stack_depth, depth_at_block_start);
                    }
                }

                // Branch instructions; for approximation, we can reset or leave the stack depth
                Instruction::Br { .. } => {
                    // For simplicity, we'll leave the stack depth unchanged
                }

                // BrIf pops one value (the condition)
                Instruction::BrIf { .. } => {
                    current_stack_depth = current_stack_depth.saturating_sub(1);
                    // Stack depth after branch remains the same for upper bound
                }

                // BrTable pops one value (the index)
                Instruction::BrTable { .. } => {
                    current_stack_depth = current_stack_depth.saturating_sub(1);
                    // Stack depth remains unchanged for approximation
                }

                // Function calls; assume stack depth remains the same for upper bound
                Instruction::Call { .. } | Instruction::CallIndirect { .. } => {
                    // If you have type info, adjust current_stack_depth accordingly
                }

                // Return resets the current stack depth
                Instruction::Return => {
                    current_stack_depth = 0;
                }

                // Select pops three values and pushes one; net effect is -2
                Instruction::Select => {
                    current_stack_depth = current_stack_depth.saturating_sub(2);
                }

                // LocalGet pushes a value onto the stack
                Instruction::LocalGet { .. } => {
                    current_stack_depth += 1;
                }

                // LocalSet pops one value from the stack
                Instruction::LocalSet { .. } => {
                    current_stack_depth = current_stack_depth.saturating_sub(1);
                }

                // LocalTee pops and then pushes the same value; net effect is 0
                Instruction::LocalTee { .. } => {}

                // GlobalGet pushes a value onto the stack
                Instruction::GlobalGet { .. } => {
                    current_stack_depth += 1;
                }

                // GlobalSet pops one value from the stack
                Instruction::GlobalSet { .. } => {
                    current_stack_depth = current_stack_depth.saturating_sub(1);
                }

                // Memory load instructions pop one address and push one value; net effect is 0
                Instruction::I32Load { .. }
                | Instruction::F64Load { .. }
                | Instruction::I32Load8S { .. }
                | Instruction::I32Load8U { .. }
                | Instruction::I32Load16S { .. }
                | Instruction::I32Load16U { .. } => {
                    // Pops one, pushes one; stack depth remains the same
                }

                // Memory store instructions pop two values (value and address); net effect is -2
                Instruction::I32Store { .. }
                | Instruction::F64Store { .. }
                | Instruction::I32Store8 { .. }
                | Instruction::I32Store16 { .. } => {
                    if current_stack_depth >= 2 {
                        current_stack_depth -= 2;
                    } else {
                        current_stack_depth = 0;
                    }
                }

                // MemorySize pushes one value onto the stack
                Instruction::MemorySize { .. } => {
                    current_stack_depth += 1;
                }

                // MemoryGrow pops one and pushes one; net effect is 0
                Instruction::MemoryGrow { .. } => {}
            }

            // Update max_stack_depth if current_stack_depth exceeds it
            if current_stack_depth > max_stack_depth {
                max_stack_depth = current_stack_depth;
            }

            pc += 1;
        }

        // Calculate total stack size: locals + max stack depth
        // Each stack slot is 8 bytes (for alignment)
        //
        // +1 for storing the current memory size
        let total_stack_size = (nlocals + max_stack_depth + 1) * 8;

        // Align stack size to 16 bytes (common requirement for x86-64)
        (total_stack_size + 15) & !15
    }
}
