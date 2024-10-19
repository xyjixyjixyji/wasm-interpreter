use std::collections::HashMap;
use std::rc::Rc;

use super::regalloc::{
    Register, X64Register, X86RegisterAllocator, REG_LOCAL_BASE, REG_MEMORY_BASE, REG_TEMP,
    REG_TEMP2, REG_TEMP_FP2,
};
use super::{JitLinearMemory, WasmJitCompiler};
use crate::jit::mov_reg_to_reg;
use crate::jit::regalloc::REG_TEMP_FP;
use crate::module::components::FuncDecl;
use crate::module::insts::{F64Binop, I32Binop, Instruction};
use crate::module::value_type::WasmValue;
use crate::module::wasm_module::WasmModule;

use anyhow::{anyhow, Result};
use debug_cell::RefCell;
use monoasm::{CodePtr, DestLabel, Disp, Imm, JitMemory, Reg, Rm, Scale};
use monoasm_macro::monoasm;
use wasmparser::ValType;

#[derive(Debug, Clone, Copy)]
enum ValueType {
    I32,
    F64,
}

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
        main_params: Vec<WasmValue>,
    ) -> Result<CodePtr> {
        // make labels for all functions
        let mut func_to_label: HashMap<usize, DestLabel> = HashMap::new();
        for (i, _) in module.borrow().get_funcs().iter().enumerate() {
            let label = self.jit.label();
            func_to_label.insert(i, label);
        }

        // setup vm_entry
        let main_index = module.borrow().get_main_index().unwrap();
        let main_label = func_to_label.get(&(main_index as usize)).unwrap();
        let vm_entry_label =
            self.setup_vm_entry(*main_label, initial_mem_size_in_byte, main_params);

        // compile all functions
        for (i, fdecl) in module.borrow().get_funcs().iter().enumerate() {
            let func_begin_label = func_to_label.get(&i).unwrap();
            self.compile_func(Rc::clone(&module), fdecl, *func_begin_label, &func_to_label)?;
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
        module: Rc<RefCell<WasmModule>>,
        fdecl: &FuncDecl,
        func_begin_label: DestLabel,
        func_to_label: &HashMap<usize, DestLabel>,
    ) -> Result<()> {
        monoasm!(
            &mut self.jit,
            func_begin_label:
        );

        self.reg_allocator.reset();

        let stack_size = self.get_stack_size_in_byte(fdecl);
        self.prologue(stack_size);

        // setup locals
        let local_types = self.setup_locals(fdecl);

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
                Instruction::Select => todo!(),
                Instruction::LocalGet { local_idx } => {
                    let dst = self.reg_allocator.next();
                    self.compile_local_get(dst, *local_idx, &local_types);
                }
                Instruction::LocalSet { local_idx } => todo!(),
                Instruction::LocalTee { local_idx } => todo!(),
                Instruction::GlobalGet { global_idx } => todo!(),
                Instruction::GlobalSet { global_idx } => todo!(),
                Instruction::I32Load { memarg } => {
                    let base = self.reg_allocator.pop();
                    let offset = memarg.offset;
                    let dst = self.reg_allocator.next();
                    self.compile_load(dst, base, offset, 4);
                }
                Instruction::F64Load { memarg } => {
                    let base = self.reg_allocator.pop();
                    let offset = memarg.offset;
                    let dst = self.reg_allocator.next();
                    self.compile_load(dst, base, offset, 8);
                }
                Instruction::I32Load8S { memarg } => todo!(),
                Instruction::I32Load8U { memarg } => todo!(),
                Instruction::I32Load16S { memarg } => todo!(),
                Instruction::I32Load16U { memarg } => todo!(),
                Instruction::I32Store { memarg } => {
                    let value = self.reg_allocator.pop();
                    let offset = memarg.offset;
                    let base = self.reg_allocator.pop();
                    self.compile_store(base, offset, value, 4);
                }
                Instruction::F64Store { memarg } => {
                    let value = self.reg_allocator.pop();
                    let offset = memarg.offset;
                    let base = self.reg_allocator.pop();
                    self.compile_store(base, offset, value, 8);
                }
                Instruction::I32Store8 { memarg } => {
                    let value = self.reg_allocator.pop();
                    let offset = memarg.offset;
                    let base = self.reg_allocator.pop();
                    self.compile_store(base, offset, value, 1);
                }
                Instruction::I32Store16 { memarg } => {
                    let value = self.reg_allocator.pop();
                    let offset = memarg.offset;
                    let base = self.reg_allocator.pop();
                    self.compile_store(base, offset, value, 2);
                }
                Instruction::MemorySize { mem } => {
                    if *mem != 0 {
                        return Err(anyhow!("memory.size: invalid memory index"));
                    }

                    let dst = self.reg_allocator.next();
                    self.store_mem_page_size(dst);
                }
                Instruction::MemoryGrow { mem } => {
                    if *mem != 0 {
                        return Err(anyhow!("memory.size: invalid memory index"));
                    }

                    let additional_pages = self.reg_allocator.pop();

                    let old_mem_size = self.reg_allocator.new_spill(); // avoid aliasing
                    self.jit_linear_mem
                        .read_memory_size_in_page(&mut self.jit, old_mem_size);

                    self.compile_memory_grow(additional_pages);
                }
                Instruction::F64Const { value } => {
                    let reg = self.reg_allocator.next_xmm();
                    self.mov_f64_to_reg(*value, reg);
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

        // return...
        let stack_top = self.reg_allocator.top();
        mov_reg_to_reg(&mut self.jit, Register::Reg(X64Register::Rax), stack_top);

        self.epilogue(stack_size);
        monoasm!(
            &mut self.jit,
            ret;
        );

        Ok(())
    }
}

impl X86JitCompiler {
    fn compile_call(&mut self, callee_func: &FuncDecl, callee: DestLabel) {
        // save caller-saved registers
        let caller_saved_regs = self.reg_allocator.get_used_caller_saved_registers();

        for reg in &caller_saved_regs {
            match reg {
                Register::Reg(r) => {
                    monoasm!(
                        &mut self.jit,
                        pushq R(r.as_index());
                    );
                }
                Register::FpReg(r) => {
                    monoasm!(
                        &mut self.jit,
                        movq R(REG_TEMP.as_index()), xmm(r.as_index());
                        pushq R(REG_TEMP.as_index());
                    );
                }
                Register::Stack(_) => panic!("stack should not be caller saved"),
            }
        }

        // setup arguments, top of the stack is the last argument
        self.setup_function_call_arguments(callee_func);

        // call the callee! and move the return value into the stack
        monoasm!(
            &mut self.jit,
            call callee;
        );

        // note that we don't want the return value to be in caller-saved registers
        // because we will pop them later in the call sequence
        let ret = self.reg_allocator.next_not_caller_saved();
        mov_reg_to_reg(&mut self.jit, ret, Register::Reg(X64Register::Rax));

        // restore the stack spaced we used.....
        let restore_size = (std::cmp::max(6, callee_func.get_sig().params().len()) - 6) * 8;
        monoasm!(
            &mut self.jit,
            addq rsp, (restore_size);
        );

        // restore caller-saved registers
        for reg in caller_saved_regs.iter().rev() {
            match reg {
                Register::Reg(r) => {
                    monoasm!(
                        &mut self.jit,
                        popq R(r.as_index());
                    );
                }
                Register::FpReg(r) => {
                    monoasm!(
                        &mut self.jit,
                        popq R(REG_TEMP.as_index());
                        movq xmm(r.as_index()), R(REG_TEMP.as_index());
                    );
                }
                Register::Stack(_) => panic!("stack should not be caller saved"),
            }
        }
    }
}

impl X86JitCompiler {
    fn compile_local_get(&mut self, dst: Register, local_idx: u32, local_types: &[ValueType]) {
        let local_type = local_types[local_idx as usize];
        match local_type {
            ValueType::I32 => {
                monoasm!(
                    &mut self.jit,
                    movq R(REG_TEMP.as_index()), R(REG_LOCAL_BASE.as_index()); // reg_temp = reg_local_base
                    addq R(REG_TEMP.as_index()), (local_idx * 8); // reg_temp = reg_local_base + local_idx * 8
                    movq R(REG_TEMP.as_index()), [R(REG_TEMP.as_index())]; // reg_temp = *reg_temp
                );
                mov_reg_to_reg(&mut self.jit, dst, Register::Reg(REG_TEMP));
            }
            ValueType::F64 => todo!(),
        }
    }

    fn compile_memory_grow(&mut self, npages: Register) {
        self.jit_linear_mem.grow(&mut self.jit, npages);
    }

    fn compile_load(&mut self, dst: Register, base: Register, offset: u32, width: u32) {
        self.get_effective_address(base, offset); // REG_TEMP stores the effective address

        // 2. load the result into dst
        monoasm!(
            &mut self.jit,
            addq R(REG_TEMP.as_index()), R(REG_MEMORY_BASE.as_index()); // <-- reg_temp = reg_memory_base + effective_addr
            movq R(REG_TEMP.as_index()), [R(REG_TEMP.as_index())]; // <-- reg_temp = *reg_temp
        );

        match width {
            8 => {}
            4 => {
                monoasm!(
                    &mut self.jit,
                    movl R(REG_TEMP.as_index()), R(REG_TEMP.as_index());
                );
            }
            2 => {
                monoasm!(
                    &mut self.jit,
                    movw R(REG_TEMP.as_index()), R(REG_TEMP.as_index());
                );
            }
            1 => {
                monoasm!(
                    &mut self.jit,
                    movb R(REG_TEMP.as_index()), R(REG_TEMP.as_index());
                );
            }
            _ => unreachable!("invalid width: {}", width),
        }

        mov_reg_to_reg(&mut self.jit, dst, Register::Reg(REG_TEMP));
    }

    fn compile_store(&mut self, base: Register, offset: u32, value: Register, width: u32) {
        self.get_effective_address(base, offset); // reg_temp = effective_addr

        // 2. store the value to dst
        monoasm!(
            &mut self.jit,
            addq R(REG_TEMP.as_index()), R(REG_MEMORY_BASE.as_index()); // <-- reg_temp = reg_memory_base + effective_addr
        );

        mov_reg_to_reg(&mut self.jit, Register::Reg(REG_TEMP2), value); // <-- reg_temp = value

        match width {
            8 => {
                monoasm!(
                    &mut self.jit,
                    movq [R(REG_TEMP.as_index())], R(REG_TEMP2.as_index());
                );
            }
            4 => {
                monoasm!(
                    &mut self.jit,
                    movl [R(REG_TEMP.as_index())], R(REG_TEMP2.as_index());
                );
            }
            2 => {
                monoasm!(
                    &mut self.jit,
                    movw [R(REG_TEMP.as_index())], R(REG_TEMP2.as_index());
                );
            }
            1 => {
                monoasm!(
                    &mut self.jit,
                    movb [R(REG_TEMP.as_index())], R(REG_TEMP2.as_index());
                );
            }
            _ => unreachable!("invalid width: {}", width),
        }
    }
}

impl X86JitCompiler {
    fn compile_f64_binop(&mut self, binop: &F64Binop) {
        let b = self.reg_allocator.pop();
        let a = self.reg_allocator.pop();

        mov_reg_to_reg(&mut self.jit, Register::FpReg(REG_TEMP_FP), a);
        mov_reg_to_reg(&mut self.jit, Register::FpReg(REG_TEMP_FP2), b);

        match binop {
            F64Binop::Add => {
                monoasm!(
                    &mut self.jit,
                    addsd xmm(REG_TEMP_FP.as_index()), xmm(REG_TEMP_FP2.as_index());
                );
            }
            F64Binop::Eq => todo!(),
            F64Binop::Ne => todo!(),
            F64Binop::Lt => todo!(),
            F64Binop::Gt => todo!(),
            F64Binop::Le => todo!(),
            F64Binop::Ge => todo!(),
            F64Binop::Sub => todo!(),
            F64Binop::Mul => todo!(),
            F64Binop::Div => todo!(),
            F64Binop::Min => todo!(),
            F64Binop::Max => todo!(),
        }

        mov_reg_to_reg(&mut self.jit, a, Register::FpReg(REG_TEMP_FP));
        self.reg_allocator.push(a);
    }

    // jit compile *a = a op b*
    fn compile_i32_binop(&mut self, binop: &I32Binop) {
        let b = self.reg_allocator.pop();
        let a = self.reg_allocator.pop();

        mov_reg_to_reg(&mut self.jit, Register::Reg(REG_TEMP), a);
        mov_reg_to_reg(&mut self.jit, Register::Reg(REG_TEMP2), b);

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
            I32Binop::DivS | I32Binop::RemS => {
                let trap_label = self.trap_label;
                let ok_label = self.jit.label();
                monoasm!(
                    &mut self.jit,
                    // div by zero check
                    testq R(REG_TEMP2.as_index()), R(REG_TEMP2.as_index());
                    jz trap_label;

                    // overflow check
                    pushq R(X64Register::Rax.as_index());
                    pushq R(X64Register::Rdx.as_index());
                    movq R(X64Register::Rax.as_index()), (0xFFFFFFFF80000000);
                    cmpq R(REG_TEMP.as_index()), R(X64Register::Rax.as_index());
                    jne ok_label;
                    movq R(X64Register::Rax.as_index()), (0xFFFFFFFFFFFFFFFF);
                    cmpq R(REG_TEMP2.as_index()), R(X64Register::Rax.as_index());
                    jne ok_label;
                    jmp trap_label;

                ok_label:
                    movq R(X64Register::Rax.as_index()), R(REG_TEMP.as_index());
                    cqo; // RDX:RAX
                    idiv R(REG_TEMP2.as_index()); // RAX: quotient, RDX: remainder
                );
                if matches!(binop, I32Binop::DivS) {
                    mov_reg_to_reg(
                        &mut self.jit,
                        Register::Reg(REG_TEMP),
                        Register::Reg(X64Register::Rax),
                    );
                } else {
                    mov_reg_to_reg(
                        &mut self.jit,
                        Register::Reg(REG_TEMP),
                        Register::Reg(X64Register::Rdx),
                    );
                }
                monoasm!(
                    &mut self.jit,
                    popq R(X64Register::Rdx.as_index());
                    popq R(X64Register::Rax.as_index());
                );
            }
            I32Binop::DivU | I32Binop::RemU => {
                let trap_label = self.trap_label;
                let ok_label = self.jit.label();
                monoasm!(
                    &mut self.jit,
                    // div by zero check
                    testq R(REG_TEMP2.as_index()), R(REG_TEMP2.as_index());
                    jz trap_label;

                ok_label:
                    pushq R(X64Register::Rax.as_index());
                    pushq R(X64Register::Rdx.as_index());

                    // Clear RDX (for unsigned division, RDX should be 0)
                    xorq R(X64Register::Rdx.as_index()), R(X64Register::Rdx.as_index());

                    // Move dividend into RAX
                    movq R(X64Register::Rax.as_index()), R(REG_TEMP.as_index());

                    // Perform the unsigned division
                    div R(REG_TEMP2.as_index()); // RAX: quotient, RDX: remainder
                );
                if matches!(binop, I32Binop::DivU) {
                    mov_reg_to_reg(
                        &mut self.jit,
                        Register::Reg(REG_TEMP),
                        Register::Reg(X64Register::Rax),
                    );
                } else {
                    mov_reg_to_reg(
                        &mut self.jit,
                        Register::Reg(REG_TEMP),
                        Register::Reg(X64Register::Rdx),
                    );
                }
                monoasm!(
                    &mut self.jit,
                    popq R(X64Register::Rdx.as_index());
                    popq R(X64Register::Rax.as_index());
                );
            }
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

        mov_reg_to_reg(&mut self.jit, a, Register::Reg(REG_TEMP));
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
        main_params: Vec<WasmValue>,
    ) -> DestLabel {
        let vm_entry_label = self.jit.label();
        monoasm!(
            &mut self.jit,
            vm_entry_label:
        );

        // setup linear memory info
        self.jit_linear_mem
            .init_size(&mut self.jit, initial_mem_size_in_byte);

        // setup main params
        for (i, param) in main_params.iter().enumerate() {
            if i < 6 {
                match param {
                    WasmValue::I32(v) => {
                        let reg = Register::from_ith_argument(i as u32, false);
                        self.mov_i32_to_reg(*v, reg);
                    }
                    WasmValue::F64(v) => {
                        let reg = Register::from_ith_argument(i as u32, true);
                        self.mov_f64_to_reg(*v, reg);
                    }
                }
            } else {
                // push the constant to stack
                match param {
                    WasmValue::I32(v) => {
                        self.mov_i32_to_reg(*v, Register::Reg(REG_TEMP));
                        monoasm!(
                            &mut self.jit,
                            pushq R(REG_TEMP.as_index());
                        );
                    }
                    WasmValue::F64(v) => {
                        self.mov_f64_to_reg(*v, Register::FpReg(REG_TEMP_FP));
                        monoasm!(
                            &mut self.jit,
                            pushq R(REG_TEMP_FP.as_index());
                        );
                    }
                }
            }
        }

        // jump to main
        monoasm!(
            &mut self.jit,
            jmp main_label;
        );

        vm_entry_label
    }

    fn setup_locals(&mut self, fdecl: &FuncDecl) -> Vec<ValueType> {
        let mut local_types = Vec::new();
        for (i, params) in fdecl.get_sig().params().iter().enumerate() {
            let r = self.reg_allocator.new_spill();

            if i == 0 {
                // store the first local to the base of the locals
                match r {
                    Register::Stack(o) => {
                        monoasm!(
                            &mut self.jit,
                            movq R(REG_LOCAL_BASE.as_index()), rsp;
                            addq R(REG_LOCAL_BASE.as_index()), (o);
                        );
                    }
                    _ => unreachable!("locals are all spilled"),
                }
            }

            if i < 6 {
                match params {
                    ValType::I32 => {
                        mov_reg_to_reg(
                            &mut self.jit,
                            r,
                            Register::from_ith_argument(i as u32, false),
                        );
                        local_types.push(ValueType::I32);
                    }
                    ValType::F64 => {
                        mov_reg_to_reg(
                            &mut self.jit,
                            r,
                            Register::from_ith_argument(i as u32, true),
                        );
                        local_types.push(ValueType::F64);
                    }
                    _ => unreachable!(),
                }
            } else {
                // the locals are spilled to the stack
                match params {
                    ValType::I32 => {
                        monoasm!(
                            &mut self.jit,
                            movq R(REG_TEMP.as_index()), [rbp + ((i as i32 - 6) * 8 + 8)];
                        );
                        mov_reg_to_reg(&mut self.jit, r, Register::Reg(REG_TEMP));
                        local_types.push(ValueType::I32);
                    }
                    ValType::F64 => {
                        todo!();
                        local_types.push(ValueType::F64);
                    }
                    _ => unreachable!(),
                }
            }
        }

        for l in fdecl.get_pure_locals() {
            let r = self.reg_allocator.new_spill();
            self.mov_i32_to_reg(0, r);

            match l {
                (_, ValType::I32) => local_types.push(ValueType::I32),
                (_, ValType::F64) => local_types.push(ValueType::F64),
                _ => unreachable!(),
            }
        }

        local_types
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

    /// REG_TEMP will store the effective address + width
    fn get_effective_address(&mut self, base: Register, offset: u32) {
        mov_reg_to_reg(&mut self.jit, Register::Reg(REG_TEMP), base); // <-- reg_temp2 = base
        monoasm!(
            &mut self.jit,
            addq R(REG_TEMP.as_index()), (offset);
        );
    }

    fn store_mem_page_size(&mut self, dst: Register) {
        self.jit_linear_mem
            .read_memory_size_in_page(&mut self.jit, dst);
    }
}

impl X86JitCompiler {
    fn setup_function_call_arguments(&mut self, callee_func: &FuncDecl) {
        let params = callee_func.get_sig().params();
        let mut args = Vec::new();
        let mut to_push = Vec::new();

        // Collect all arguments from reg_allocator (stack top first)
        for _ in 0..params.len() {
            let arg = self.reg_allocator.pop();
            args.insert(0, arg);
        }

        // Now process parameters and arguments from last to first
        for (i, param) in params.iter().enumerate().rev() {
            let arg = args.pop().unwrap(); // Gets arguments from first to last
            if i < 6 {
                // Handle register arguments
                match param {
                    ValType::I32 => {
                        log::debug!(
                            "moving {:?} to {:?}",
                            arg,
                            Register::from_ith_argument(i as u32, false)
                        );
                        mov_reg_to_reg(
                            &mut self.jit,
                            Register::from_ith_argument(i as u32, false),
                            arg,
                        );
                    }
                    ValType::F64 => {
                        mov_reg_to_reg(
                            &mut self.jit,
                            Register::from_ith_argument(i as u32, true),
                            arg,
                        );
                    }
                    _ => unimplemented!("Invalid param type for JIT: {:?}", param),
                }
            } else {
                to_push.push(arg);
            }
        }

        for arg in to_push.iter().rev() {
            match arg {
                Register::Reg(r) => {
                    monoasm!(
                        &mut self.jit,
                        pushq R(r.as_index());
                    );
                }
                Register::FpReg(r) => {
                    monoasm!(
                        &mut self.jit,
                        movq R(REG_TEMP.as_index()), xmm(r.as_index());
                        pushq R(REG_TEMP.as_index());
                    );
                }
                Register::Stack(o) => {
                    monoasm!(
                        &mut self.jit,
                        movq R(REG_TEMP.as_index()), [rsp + (*o)];
                        pushq R(REG_TEMP.as_index());
                    );
                }
            }
        }
    }

    // Get the stack size usage of the function, used for stack allocation
    // We get only an upper bound approximate, since we don't want too much overhead
    fn get_stack_size_in_byte(&self, fdecl: &FuncDecl) -> u64 {
        let nlocals = (fdecl.get_pure_locals().len() + fdecl.get_sig().params().len()) as u64;

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
