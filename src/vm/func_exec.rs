use anyhow::{anyhow, Result};
use debug_cell::RefCell;
use log::debug;
use wasmparser::{BinaryReader, BlockType, ConstExpr, ValType, WasmFeatures};

use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use super::{interpreter::LinearMemory, WasmFunctionExecutor};
use crate::module::{
    components::FuncDecl,
    insts::{F64Binop, F64Unop, I32Binop, I32Unop, Instruction, MemArg},
    value_type::WasmValue,
    wasm_module::WasmModule,
    wasmops::{WASM_OP_END, WASM_OP_F64_CONST, WASM_OP_I32_CONST},
};

type Pc = usize;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum BlockControlFlowType {
    Block,
    If {
        else_pc: Option<Pc>,
        condition_met: bool,
    },
    Loop,
}

/// Control flow frame for a code block, start with Block, If, Loop, etc.
pub(super) struct BlockControlFlowFrame {
    /// Could be If, Else, Block, Loop, etc.
    pub(super) control_type: BlockControlFlowType,
    /// the height of the stack that expected when the block ends, for unwinding
    pub(super) expected_stack_height: usize,
    /// Program counter where the block starts
    pub(super) start_pc: Pc,
    /// Program counter of the `end` instruction for the block
    pub(super) end_pc: Pc,
}

pub(crate) struct WasmFunctionExecutorImpl<'a> {
    /// The function to execute.
    func: FuncDecl,
    /// The program counter. Point into function's instructions.
    pc: Pc,
    /// The operand stack.
    operand_stack: VecDeque<WasmValue>,
    /// local variables
    locals: Vec<WasmValue>,
    /// The control flow frame for code blocks
    control_flow_frames: VecDeque<BlockControlFlowFrame>,
    /// The reference to the linear memory for the Wasm VM instance.
    mem: Rc<RefCell<LinearMemory>>,
    /// The reference to the Wasm module for the Wasm VM instance.
    module: Rc<RefCell<WasmModule<'a>>>,
}

impl<'a> WasmFunctionExecutor for WasmFunctionExecutorImpl<'a> {
    fn execute(&mut self) -> Result<WasmValue> {
        // function frame
        self.control_flow_frames.push_back(BlockControlFlowFrame {
            control_type: BlockControlFlowType::Block,
            expected_stack_height: 0,
            start_pc: 0,
            end_pc: Self::find_matching_end(&self.func.get_insts(), 0)?,
        });

        let mut done_exec = false;
        while !done_exec && self.pc < self.func.get_insts().len() {
            let inst = self.func.get_inst(self.pc).clone();

            if self.should_skip(self.pc) {
                self.inc_pc();
                continue;
            }

            match inst {
                Instruction::Return => {
                    done_exec = true;
                }
                Instruction::Unreachable => {
                    Err(anyhow!("unreachable instruction"))?;
                }
                Instruction::Nop => {
                    self.inc_pc();
                }
                Instruction::Block { ty } => {
                    let insts = self.func.get_insts().clone();
                    self.run_block(&insts, ty)?;
                    self.inc_pc();
                }
                Instruction::Loop { ty } => todo!(),
                Instruction::If { ty } => {
                    let insts = self.func.get_insts().clone();
                    self.run_if(&insts, ty)?;
                    self.inc_pc();
                }
                // we use control flow frames to handle else blocks, instructions
                // check the top of the stack and conditionally execute, so we
                // don't need to handle them here.
                Instruction::Else => {
                    self.inc_pc();
                }
                Instruction::End => {
                    self.inc_pc();
                }
                Instruction::Br { rel_depth } => todo!(),
                Instruction::BrIf { rel_depth } => todo!(),
                Instruction::BrTable { table } => todo!(),
                Instruction::Call { func_idx } => {
                    let func = self.module.borrow().get_func(func_idx).unwrap().clone();
                    let v = self.call_func(func);
                    self.push_operand_stack(v);
                    self.inc_pc();
                }
                Instruction::CallIndirect {
                    type_index,
                    table_index,
                } => {
                    self.run_call_indirect(type_index, table_index)?;
                    self.inc_pc();
                }
                Instruction::Drop => {
                    self.pop_operand_stack();
                    self.inc_pc();
                }
                Instruction::Select => {
                    let cond = self.pop_operand_stack().as_i32();
                    let b = self.pop_operand_stack();
                    let a = self.pop_operand_stack();
                    self.push_operand_stack(if cond != 0 { a } else { b });
                    self.inc_pc();
                }
                Instruction::LocalGet { local_idx } => {
                    let local = self.locals[local_idx as usize];
                    self.push_operand_stack(local);
                    self.inc_pc();
                }
                Instruction::LocalSet { local_idx } => {
                    let value = self.pop_operand_stack();
                    self.locals[local_idx as usize] = value;
                    self.inc_pc();
                }
                Instruction::LocalTee { local_idx } => {
                    let value = self.pop_operand_stack();
                    self.locals[local_idx as usize] = value;
                    self.push_operand_stack(value);
                    self.inc_pc();
                }
                Instruction::GlobalGet { global_idx } => {
                    self.run_global_get(global_idx)?;
                    self.inc_pc();
                }
                Instruction::GlobalSet { global_idx } => {
                    self.run_global_set(global_idx)?;
                    self.inc_pc();
                }
                Instruction::I32Load { memarg } => {
                    let v = self.run_i32_load(&memarg, 4)?;
                    self.push_operand_stack(v);
                    self.inc_pc();
                }
                Instruction::F64Load { memarg } => {
                    let v = self.run_f64_load(&memarg)?;
                    self.push_operand_stack(v);
                    self.inc_pc();
                }
                Instruction::I32Load8S { memarg } => {
                    let v = self.run_i32_load(&memarg, 1)?.as_i32();
                    let v = ((v & 0xFF) as i8) as i32;
                    self.push_operand_stack(WasmValue::I32(v));
                    self.inc_pc();
                }
                Instruction::I32Load8U { memarg } => {
                    let v = self.run_i32_load(&memarg, 1)?.as_i32();
                    let v = v & 0xFF;
                    self.push_operand_stack(WasmValue::I32(v));
                    self.inc_pc();
                }
                Instruction::I32Load16S { memarg } => {
                    let v = self.run_i32_load(&memarg, 2)?.as_i32();
                    let v = ((v & 0xFFFF) as i16) as i32;
                    self.push_operand_stack(WasmValue::I32(v));
                    self.inc_pc();
                }
                Instruction::I32Load16U { memarg } => {
                    let v = self.run_i32_load(&memarg, 2)?.as_i32();
                    let v = v & 0xFFFF;
                    self.push_operand_stack(WasmValue::I32(v));
                    self.inc_pc();
                }
                Instruction::I32Store { memarg } => {
                    self.run_i32_store(&memarg, 4)?;
                    self.inc_pc();
                }
                Instruction::F64Store { memarg } => {
                    self.run_f64_store(&memarg)?;
                    self.inc_pc();
                }
                Instruction::I32Store8 { memarg } => {
                    self.run_i32_store(&memarg, 1)?;
                    self.inc_pc();
                }
                Instruction::I32Store16 { memarg } => {
                    self.run_i32_store(&memarg, 2)?;
                    self.inc_pc();
                }
                Instruction::MemorySize { mem } => {
                    self.run_memory_size(mem)?;
                    self.inc_pc();
                }
                Instruction::MemoryGrow { mem } => {
                    self.run_memory_grow(mem)?;
                    self.inc_pc();
                }
                Instruction::I32Const { value } => {
                    self.push_operand_stack(WasmValue::I32(value));
                    self.inc_pc();
                }
                Instruction::F64Const { value } => {
                    self.push_operand_stack(WasmValue::F64(value));
                    self.inc_pc();
                }
                Instruction::I32Unop(i32_unop) => {
                    self.run_i32_unop(&i32_unop)?;
                    self.inc_pc();
                }
                Instruction::I32Binp(i32_binop) => {
                    self.run_i32_binop(&i32_binop)?;
                    self.inc_pc();
                }
                Instruction::F64Unop(f64_unop) => {
                    self.run_f64_unop(&f64_unop)?;
                    self.inc_pc();
                }
                Instruction::F64Binop(f64_binop) => {
                    self.run_f64_binop(&f64_binop)?;
                    self.inc_pc();
                }
            }
        }

        Ok(self.pop_operand_stack())
    }
}

impl<'a> WasmFunctionExecutorImpl<'a> {
    pub fn new(
        func: FuncDecl,
        module: Rc<RefCell<WasmModule<'a>>>,
        mem: Rc<RefCell<LinearMemory>>,
        init_locals: Option<Vec<WasmValue>>,
    ) -> Self {
        let locals = Self::setup_locals(init_locals, &func);
        Self {
            func,
            pc: 0,
            mem,
            module,
            locals,
            control_flow_frames: VecDeque::new(),
            operand_stack: VecDeque::new(),
        }
    }

    // constructor helpers
    fn setup_locals(main_locals: Option<Vec<WasmValue>>, func: &FuncDecl) -> Vec<WasmValue> {
        let mut locals = main_locals.unwrap_or_default();

        locals.extend(func.get_pure_locals().iter().flat_map(|(cnt, ty)| {
            vec![WasmValue::default_value(ty); usize::try_from(*cnt).expect("local count overflow")]
        }));

        locals
    }
}

impl<'a> WasmFunctionExecutorImpl<'a> {
    pub fn inc_pc(&mut self) {
        self.pc += 1;
    }

    pub fn set_pc(&mut self, pc: Pc) {
        self.pc = pc;
    }

    pub fn push_operand_stack(&mut self, value: WasmValue) {
        self.operand_stack.push_front(value);
    }

    pub fn pop_operand_stack(&mut self) -> WasmValue {
        self.operand_stack
            .pop_front()
            .expect("operand stack underflow")
    }

    pub fn mem_size(&self) -> usize {
        self.mem.borrow().size()
    }

    pub fn grow_mem(&mut self, additional_pages: u32) {
        self.mem.borrow_mut().grow(additional_pages);
    }

    pub fn call_func(&mut self, func: FuncDecl) -> WasmValue {
        // prepare the argument locals
        let mut args = VecDeque::new();
        for param in func.get_sig().params().iter().rev() {
            let v = self.pop_operand_stack();
            match param {
                ValType::I32 => {
                    if !matches!(v, WasmValue::I32(_)) {
                        panic!("call_func: invalid argument type");
                    }
                }
                ValType::F64 => {
                    if !matches!(v, WasmValue::F64(_)) {
                        panic!("call_func: invalid argument type");
                    }
                }
                _ => panic!("unsupported param type"),
            }
            args.push_front(v);
        }

        let mut executor = WasmFunctionExecutorImpl::new(
            func,
            Rc::clone(&self.module),
            Rc::clone(&self.mem),
            Some(args.into()),
        );

        executor.execute().unwrap()
    }
}

/// Instruction execution
impl<'a> WasmFunctionExecutorImpl<'a> {
    fn run_call_indirect(&mut self, type_index: u32, table_index: u32) -> Result<()> {
        let callee_index_in_table = self.pop_operand_stack().as_i32();

        let module_ref = self.module.borrow();
        let elem = module_ref
            .get_elems()
            .iter()
            .find(|e| match e.kind {
                wasmparser::ElementKind::Passive => {
                    panic!("passive element segment not implemented")
                }
                wasmparser::ElementKind::Active { table_index: i, .. } => Some(table_index) == i,
                wasmparser::ElementKind::Declared => {
                    panic!("declared element segment not implemented")
                }
            })
            .ok_or_else(|| anyhow!("element segment not found"))?;

        let func_indices = match &elem.items {
            wasmparser::ElementItems::Functions(r) => r
                .clone()
                .into_iter()
                .map(|i| i.expect("invalid function index"))
                .collect::<Vec<_>>(),
            _ => {
                panic!("Should be function elements in the segment");
            }
        };

        let callee_index = func_indices[callee_index_in_table as usize];
        let callee = module_ref
            .get_func(callee_index)
            .expect("callee not found")
            .clone();

        // check callee signature
        let expected_sig = module_ref
            .get_sig(type_index)
            .expect("callee signature not found");
        let actual_sig = callee.get_sig();
        if expected_sig != actual_sig {
            panic!("call_indirect: callee signature mismatch");
        }
        drop(module_ref);

        let v = self.call_func(callee.clone());

        self.push_operand_stack(v);

        Ok(())
    }

    fn run_global_get(&mut self, global_index: u32) -> Result<()> {
        let module = self.module.borrow();
        let global = module
            .get_globals()
            .get(global_index as usize)
            .expect("global not found");

        let value = match global.get_ty().content_type {
            ValType::I32 => {
                let init_expr = global.get_init_expr();
                let mut reader = BinaryReader::new(init_expr, 0, WasmFeatures::all());
                let op = reader.read_var_u32()?;
                if op != WASM_OP_I32_CONST {
                    return Err(anyhow!(
                        "global.get: invalid init expr, should start with i32.const"
                    ));
                }
                WasmValue::I32(reader.read_var_i32()?)
            }
            ValType::F64 => {
                let init_expr = global.get_init_expr();
                let mut reader = BinaryReader::new(init_expr, 0, WasmFeatures::all());
                let op = reader.read_var_u32()?;
                if op != WASM_OP_F64_CONST {
                    return Err(anyhow!(
                        "global.get: invalid init expr, should start with f64.const"
                    ));
                }
                WasmValue::F64(f64::from(reader.read_f64()?))
            }
            _ => panic!("unsupported global type"),
        };

        drop(module);

        self.push_operand_stack(value);

        Ok(())
    }

    fn run_global_set(&mut self, global_index: u32) -> Result<()> {
        let value = self.pop_operand_stack();

        let mut module = self.module.borrow_mut();
        let global = module
            .get_globals_mut()
            .get_mut(global_index as usize)
            .expect("global not found");

        // TODO: check mutability

        match global.get_ty().content_type {
            ValType::I32 => {
                if !matches!(value, WasmValue::I32(_)) {
                    return Err(anyhow!("global.set: invalid value type"));
                }
            }
            ValType::F64 => {
                if !matches!(value, WasmValue::F64(_)) {
                    return Err(anyhow!("global.set: invalid value type"));
                }
            }
            _ => panic!("unsupported global type"),
        }

        let mut init_expr = vec![];
        match value {
            WasmValue::I32(v) => {
                init_expr.push(WASM_OP_I32_CONST as u8);
                init_expr.extend(encode_i32leb(v));
                init_expr.push(WASM_OP_END as u8);
            }
            WasmValue::F64(v) => {
                init_expr.push(WASM_OP_F64_CONST as u8);
                init_expr.extend(encode_f64(v));
                init_expr.push(WASM_OP_END as u8);
            }
        }

        global.set_init_expr(init_expr);

        Ok(())
    }

    fn run_memory_size(&mut self, mem: u32) -> Result<()> {
        if mem != 0 {
            return Err(anyhow!("memory.size: invalid memory index"));
        }

        self.operand_stack
            .push_back(WasmValue::I32(i32::try_from(self.mem_size()).unwrap()));

        Ok(())
    }

    fn run_memory_grow(&mut self, mem: u32) -> Result<()> {
        if mem != 0 {
            return Err(anyhow!("memory.grow: invalid memory index"));
        }

        let additional_pages = self.pop_operand_stack().as_i32();
        self.grow_mem(u32::try_from(additional_pages)?);

        self.operand_stack
            .push_back(WasmValue::I32(i32::try_from(self.mem_size()).unwrap()));

        Ok(())
    }

    fn run_i32_load(&mut self, memarg: &MemArg, width: u32) -> Result<WasmValue> {
        let base = u32::try_from(self.pop_operand_stack().as_i32())?;
        let effective_addr = base + memarg.offset;

        let mem = self.mem.borrow();
        let mem_size = mem.size();

        if effective_addr + width > mem_size as u32 {
            return Err(anyhow!(
                "out of bounds memory access, effective_addr: {}, width: {}, mem_size: {}",
                effective_addr,
                width,
                mem_size
            ));
        }

        // little endian read
        let mut value = 0u32;
        for i in 0..width {
            value |= (mem.0[(effective_addr + i) as usize] as u32) << (i * 8);
        }
        drop(mem);

        let i32_value = i32::from_le_bytes(value.to_le_bytes());
        Ok(WasmValue::I32(i32_value))
    }

    fn run_i32_store(&mut self, memarg: &MemArg, width: u32) -> Result<()> {
        let value = self.pop_operand_stack().as_i32();
        let base = u32::try_from(self.pop_operand_stack().as_i32())?;
        let effective_addr = base + memarg.offset;

        let mut mem = self.mem.borrow_mut();
        let mem_size = mem.size();

        if effective_addr + width > mem_size as u32 {
            return Err(anyhow!(
                "out of bounds memory access, effective_addr: {}, width: {}, mem_size: {}",
                effective_addr,
                width,
                mem_size
            ));
        }

        for i in 0..width {
            mem.0[(effective_addr + i) as usize] = ((value >> (i * 8)) & 0xFF) as u8;
        }

        Ok(())
    }

    fn run_f64_load(&mut self, memarg: &MemArg) -> Result<WasmValue> {
        let base = u32::try_from(self.pop_operand_stack().as_i32())?;
        let effective_addr = base + memarg.offset;

        let mem = self.mem.borrow();
        let mem_size = mem.size();

        if effective_addr + 8 > mem_size as u32 {
            return Err(anyhow!(
                "out of bounds memory access, effective_addr: {}, width: {}, mem_size: {}",
                effective_addr,
                8,
                mem_size
            ));
        }

        let mut value = 0u64;
        for i in 0..8 {
            value |= (mem.0[(effective_addr + i) as usize] as u64) << (i * 8);
        }
        drop(mem);

        let f64_value = f64::from_le_bytes(value.to_le_bytes());
        Ok(WasmValue::F64(f64_value))
    }

    fn run_f64_store(&mut self, memarg: &MemArg) -> Result<()> {
        unimplemented!()
    }

    fn run_i32_unop(&mut self, i32_unop: &I32Unop) -> Result<()> {
        let a = self.pop_operand_stack().as_i32();
        let result = match i32_unop {
            I32Unop::Eqz => i32::try_from(a == 0)?,
            I32Unop::Clz => i32::try_from(a.leading_zeros())?,
            I32Unop::Ctz => i32::try_from(a.trailing_zeros())?,
            I32Unop::Popcnt => i32::try_from(a.count_ones())?,
            I32Unop::Extend8S => todo!(),
            I32Unop::Extend16S => todo!(),
            I32Unop::F64ConvertI32S => todo!(),
            I32Unop::F64ConvertI32U => todo!(),
        };

        self.push_operand_stack(WasmValue::I32(result));

        Ok(())
    }

    fn run_i32_binop(&mut self, i32_binop: &I32Binop) -> Result<()> {
        let b = self.pop_operand_stack().as_i32();
        let a = self.pop_operand_stack().as_i32();
        let result = match i32_binop {
            I32Binop::Eq => i32::try_from(a == b)?,
            I32Binop::Ne => i32::try_from(a != b)?,
            I32Binop::LtS => i32::try_from(a < b)?,
            I32Binop::LtU => i32::try_from((a as u32) < (b as u32))?,
            I32Binop::GtS => i32::try_from(a > b)?,
            I32Binop::GtU => i32::try_from((a as u32) > (b as u32))?,
            I32Binop::LeS => i32::try_from(a <= b)?,
            I32Binop::LeU => i32::try_from((a as u32) <= (b as u32))?,
            I32Binop::GeS => i32::try_from(a >= b)?,
            I32Binop::GeU => i32::try_from((a as u32) >= (b as u32))?,
            I32Binop::Add => i32::try_from(a.wrapping_add(b))?,
            I32Binop::Sub => i32::try_from(a.wrapping_sub(b))?,
            I32Binop::Mul => i32::try_from(a.wrapping_mul(b))?,
            I32Binop::DivS => todo!(),
            I32Binop::DivU => todo!(),
            I32Binop::RemS => todo!(),
            I32Binop::RemU => todo!(),
            I32Binop::And => i32::try_from(a & b)?,
            I32Binop::Or => i32::try_from(a | b)?,
            I32Binop::Xor => i32::try_from(a ^ b)?,
            I32Binop::Shl => i32::try_from(a.wrapping_shl((b & 0x1f) as u32))?,
            I32Binop::ShrS => todo!(),
            I32Binop::ShrU => todo!(),
            I32Binop::Rotl => todo!(),
            I32Binop::Rotr => todo!(),
        };

        self.push_operand_stack(WasmValue::I32(result));

        Ok(())
    }

    fn run_f64_unop(&mut self, f64_unop: &F64Unop) -> Result<()> {
        let a = self.pop_operand_stack().as_f64();
        let result = match f64_unop {
            F64Unop::Neg => WasmValue::F64(-a),
            F64Unop::Abs => WasmValue::F64(a.abs()),
            F64Unop::Ceil => WasmValue::F64(a.ceil()),
            F64Unop::Floor => WasmValue::F64(a.floor()),
            F64Unop::Trunc => WasmValue::F64(a.trunc()),
            F64Unop::Nearest => WasmValue::F64(a.round()),
            F64Unop::Sqrt => WasmValue::F64(a.sqrt()),
            F64Unop::I32TruncF64S => todo!(),
            F64Unop::I32TruncF64U => todo!(),
        };

        self.push_operand_stack(result);
        Ok(())
    }

    fn run_f64_binop(&mut self, f64_binop: &F64Binop) -> Result<()> {
        let b = self.pop_operand_stack().as_f64();
        let a = self.pop_operand_stack().as_f64();
        let result = match f64_binop {
            F64Binop::Eq => WasmValue::I32(i32::try_from(a == b)?),
            F64Binop::Ne => WasmValue::I32(i32::try_from(a != b)?),
            F64Binop::Lt => WasmValue::I32(i32::try_from(a < b)?),
            F64Binop::Gt => WasmValue::I32(i32::try_from(a > b)?),
            F64Binop::Le => WasmValue::I32(i32::try_from(a <= b)?),
            F64Binop::Ge => WasmValue::I32(i32::try_from(a >= b)?),
            F64Binop::Add => WasmValue::F64(a + b),
            F64Binop::Sub => WasmValue::F64(a - b),
            F64Binop::Mul => WasmValue::F64(a * b),
            F64Binop::Div => WasmValue::F64(a / b),
            F64Binop::Min => WasmValue::F64(a.min(b)),
            F64Binop::Max => WasmValue::F64(a.max(b)),
        };

        self.push_operand_stack(result);

        Ok(())
    }

    // control flow functions
    fn run_block(&mut self, insts: &[Instruction], block_type: BlockType) -> Result<()> {
        let mut expected_stack_height = self.operand_stack.len();
        expected_stack_height += self.stack_height_delta(block_type);

        let frame = BlockControlFlowFrame {
            control_type: BlockControlFlowType::Block,
            expected_stack_height,
            start_pc: self.pc,
            end_pc: Self::find_matching_end(insts, self.pc)?,
        };

        self.control_flow_frames.push_back(frame);

        Ok(())
    }

    /// Run the if instruction, return true if the condition is met, false otherwise
    fn run_if(&mut self, insts: &[Instruction], block_type: BlockType) -> Result<()> {
        let mut expected_stack_height = self.operand_stack.len();
        expected_stack_height += self.stack_height_delta(block_type);

        let cond = self.pop_operand_stack().as_i32();
        let else_pc = Self::find_closest_else(insts, self.pc);
        let frame = BlockControlFlowFrame {
            control_type: BlockControlFlowType::If {
                else_pc,
                condition_met: cond != 0,
            },
            expected_stack_height,
            start_pc: self.pc,
            end_pc: Self::find_matching_end(insts, self.pc)?,
        };

        self.control_flow_frames.push_back(frame);

        Ok(())
    }
}

impl<'a> WasmFunctionExecutorImpl<'a> {
    fn find_closest_else(insts: &[Instruction], start: Pc) -> Option<Pc> {
        let end_pc = Self::find_matching_end(insts, start).expect("no matching end for if block");
        let mut pc = start;
        while pc < insts.len() {
            let inst = &insts[pc];
            if inst == &Instruction::Else {
                if pc < end_pc {
                    return Some(pc);
                } else {
                    return None;
                }
            }
            pc += 1;
        }

        None
    }

    fn find_matching_end(insts: &[Instruction], start: Pc) -> Result<Pc> {
        let mut pc = start;
        let mut depth = 0;
        while pc < insts.len() {
            let inst = &insts[pc];
            if Instruction::is_control_block_start(inst) {
                depth += 1;
            } else if Instruction::is_control_block_end(inst) {
                depth -= 1;
            }

            if depth == 0 {
                return Ok(pc);
            }

            pc += 1;
        }

        Err(anyhow!("no matching end for block"))
    }

    fn should_skip(&self, pc: Pc) -> bool {
        let frame = self.control_flow_frames.back().unwrap();
        match frame.control_type {
            BlockControlFlowType::Block => false,
            BlockControlFlowType::Loop => false,
            BlockControlFlowType::If {
                else_pc,
                condition_met,
            } => {
                if let Some(else_pc) = else_pc {
                    if pc >= else_pc {
                        condition_met
                    } else {
                        !condition_met
                    }
                } else {
                    !condition_met
                }
            }
        }
    }

    fn stack_height_delta(&self, block_type: BlockType) -> usize {
        match block_type {
            BlockType::Empty => 0,
            BlockType::Type(_) => 1,
            BlockType::FuncType(f) => {
                let module = self.module.borrow();
                let func = module.get_func(f).expect("function not found");
                let nparams = func.get_sig().params().len();
                let nresults = func.get_sig().results().len();
                nresults - nparams
            }
        }
    }
}

fn encode_i32leb(v: i32) -> Vec<u8> {
    let mut buf = vec![];

    let mut val = v;
    let mut b: u8 = 0xFF;
    while b & 0x80 != 0 {
        b = (val & 0x7F) as u8;
        val >>= 7;
        if !(((val == 0) && !(b & 0x40 != 0)) || ((val == -1) && (b & 0x40 != 0))) {
            b |= 0x80;
        }
        buf.push(b);
    }

    buf
}

fn encode_f64(v: f64) -> Vec<u8> {
    let u64 = u64::from_le_bytes(v.to_le_bytes());
    u64.to_le_bytes().to_vec()
}
