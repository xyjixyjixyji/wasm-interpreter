use anyhow::{anyhow, Result};
use debug_cell::RefCell;
use log::debug;
use wasmparser::{BinaryReader, ConstExpr, ValType, WasmFeatures};

use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use super::{interpreter::LinearMemory, WasmFunctionExecutor};
use crate::module::{
    components::FuncDecl,
    insts::{F64Binop, F64Unop, I32Binop, I32Unop, Instructions, MemArg},
    value_type::WasmValue,
    wasm_module::WasmModule,
    wasmops::{WASM_OP_END, WASM_OP_F64_CONST, WASM_OP_I32_CONST},
};

struct Pc(usize);

pub(super) struct ControlFlowTable {
    /// jump table, the key is the branch instruction pc, the value is the target pc.
    /// the table can be used to quickly set the pc to target pc.
    jump_table: HashMap<Pc, Pc>,
    /// stack height table, the key is the branch instruction pc, the value is
    /// the stack height when entering it. This is used when we take a branch,
    /// we need to unwind the stack to this height.
    stack_height_table: HashMap<Pc, usize>,
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
    /// The control flow table
    control_flow_table: ControlFlowTable,
    /// The reference to the linear memory for the Wasm VM instance.
    mem: Rc<RefCell<LinearMemory>>,
    /// The reference to the Wasm module for the Wasm VM instance.
    module: Rc<RefCell<WasmModule<'a>>>,
}

impl<'a> WasmFunctionExecutor for WasmFunctionExecutorImpl<'a> {
    fn execute(&mut self) -> Result<WasmValue> {
        let mut done_exec = false;
        while !done_exec && self.pc.0 < self.func.get_insts().len() {
            let inst = self.func.get_inst(self.pc.0).clone();
            match inst {
                Instructions::Return => {
                    done_exec = true;
                }
                Instructions::Unreachable => {
                    Err(anyhow!("unreachable instruction"))?;
                }
                Instructions::Nop => {
                    self.inc_pc();
                }
                Instructions::Block { ty } => todo!(),
                Instructions::Loop { ty } => todo!(),
                Instructions::If { ty } => todo!(),
                Instructions::Else => todo!(),
                Instructions::End => {
                    self.inc_pc();
                }
                Instructions::Br { rel_depth } => todo!(),
                Instructions::BrIf { rel_depth } => todo!(),
                Instructions::BrTable { table } => todo!(),
                Instructions::Call { func_idx } => todo!(),
                Instructions::CallIndirect {
                    type_index,
                    table_index,
                } => todo!(),
                Instructions::Drop => {
                    self.pop_operand_stack();
                    self.inc_pc();
                }
                Instructions::Select => todo!(),
                Instructions::LocalGet { local_idx } => {
                    let local = self.locals[local_idx as usize];
                    self.push_operand_stack(local);
                    self.inc_pc();
                }
                Instructions::LocalSet { local_idx } => {
                    let value = self.pop_operand_stack();
                    self.locals[local_idx as usize] = value;
                    self.inc_pc();
                }
                Instructions::LocalTee { local_idx } => {
                    let value = self.pop_operand_stack();
                    self.locals[local_idx as usize] = value;
                    self.push_operand_stack(value);
                    self.inc_pc();
                }
                Instructions::GlobalGet { global_idx } => {
                    self.run_global_get(global_idx)?;
                    self.inc_pc();
                }
                Instructions::GlobalSet { global_idx } => {
                    self.run_global_set(global_idx)?;
                    self.inc_pc();
                }
                Instructions::I32Load { memarg } => {
                    let v = self.run_i32_load(&memarg, 4)?;
                    self.push_operand_stack(v);
                    self.inc_pc();
                }
                Instructions::F64Load { memarg } => {
                    let v = self.run_f64_load(&memarg)?;
                    self.push_operand_stack(v);
                    self.inc_pc();
                }
                Instructions::I32Load8S { memarg } => {
                    let v = self.run_i32_load(&memarg, 1)?.as_i32();
                    let v = ((v & 0xFF) as i8) as i32;
                    self.push_operand_stack(WasmValue::I32(v));
                    self.inc_pc();
                }
                Instructions::I32Load8U { memarg } => {
                    let v = self.run_i32_load(&memarg, 1)?.as_i32();
                    let v = v & 0xFF;
                    self.push_operand_stack(WasmValue::I32(v));
                    self.inc_pc();
                }
                Instructions::I32Load16S { memarg } => {
                    let v = self.run_i32_load(&memarg, 2)?.as_i32();
                    let v = ((v & 0xFFFF) as i16) as i32;
                    self.push_operand_stack(WasmValue::I32(v));
                    self.inc_pc();
                }
                Instructions::I32Load16U { memarg } => {
                    let v = self.run_i32_load(&memarg, 2)?.as_i32();
                    let v = v & 0xFFFF;
                    self.push_operand_stack(WasmValue::I32(v));
                    self.inc_pc();
                }
                Instructions::I32Store { memarg } => todo!(),
                Instructions::F64Store { memarg } => todo!(),
                Instructions::I32Store8 { memarg } => todo!(),
                Instructions::I32Store16 { memarg } => todo!(),
                Instructions::MemorySize { mem } => {
                    self.run_memory_size(mem)?;
                    self.inc_pc();
                }
                Instructions::MemoryGrow { mem } => {
                    self.run_memory_grow(mem)?;
                    self.inc_pc();
                }
                Instructions::I32Const { value } => {
                    self.push_operand_stack(WasmValue::I32(value));
                    self.inc_pc();
                }
                Instructions::F64Const { value } => {
                    self.push_operand_stack(WasmValue::F64(value));
                    self.inc_pc();
                }
                Instructions::I32Unop(i32_unop) => {
                    self.run_i32_unop(&i32_unop)?;
                    self.inc_pc();
                }
                Instructions::I32Binp(i32_binop) => {
                    self.run_i32_binop(&i32_binop)?;
                    self.inc_pc();
                }
                Instructions::F64Unop(f64_unop) => {
                    self.run_f64_unop(&f64_unop)?;
                    self.inc_pc();
                }
                Instructions::F64Binop(f64_binop) => {
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
        main_locals: Option<Vec<WasmValue>>,
    ) -> Self {
        let control_flow_table = Self::analyze_control_flow_table(&func, Rc::clone(&module));
        let mut locals = if let Some(locals) = main_locals {
            locals
        } else {
            vec![]
        };

        locals.extend(
            func.get_pure_locals()
                .iter()
                .map(|(_, ty)| WasmValue::default_value(ty)),
        );

        Self {
            func,
            pc: Pc(0),
            mem,
            module,
            locals,
            control_flow_table,
            operand_stack: VecDeque::new(),
        }
    }

    pub fn inc_pc(&mut self) {
        self.pc.0 += 1;
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

    pub fn call_func(&self, func: FuncDecl) -> WasmValue {
        let mut executor = WasmFunctionExecutorImpl::new(
            func,
            Rc::clone(&self.module),
            Rc::clone(&self.mem),
            None,
        );

        executor.execute().unwrap()
    }

    // TODO
    fn analyze_control_flow_table(
        func: &FuncDecl,
        module: Rc<RefCell<WasmModule<'a>>>,
    ) -> ControlFlowTable {
        let jump_table = HashMap::new();
        let stack_height_table = HashMap::new();

        ControlFlowTable {
            jump_table,
            stack_height_table,
        }
    }
}

/// Instruction execution
impl<'a> WasmFunctionExecutorImpl<'a> {
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
        let effective_addr = align(base + memarg.offset, memarg.align);

        let mem = self.mem.borrow();
        let mem_size = mem.size();

        if effective_addr + width > mem_size as u32 {
            return Err(anyhow!("out of bounds memory access"));
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
        let effective_addr = align(base + memarg.offset, memarg.align);
        unimplemented!();
        Ok(())
    }

    fn run_f64_load(&mut self, memarg: &MemArg) -> Result<WasmValue> {
        let base = u32::try_from(self.pop_operand_stack().as_i32())?;
        let effective_addr = align(base + memarg.offset, memarg.align);

        let mem = self.mem.borrow();
        let mem_size = mem.size();

        if effective_addr + 8 > mem_size as u32 {
            return Err(anyhow!("out of bounds memory access"));
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
            I32Binop::Add => i32::try_from(a + b)?,
            I32Binop::Sub => i32::try_from(a - b)?,
            I32Binop::Mul => i32::try_from(a * b)?,
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

fn align(addr: u32, align: u32) -> u32 {
    (addr + (align - 1)) & !(align - 1)
}
