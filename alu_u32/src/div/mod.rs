extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use columns::{Div32Cols, DIV_COL_MAP, NUM_DIV_COLS};
use core::mem::transmute;
use valida_bus::MachineWithGeneralBus;
use valida_cpu::MachineWithCpuChip;
use valida_machine::SDiv;
use valida_machine::StarkConfig;
use valida_machine::{instructions, Chip, Instruction, Interaction, Operands, Word};
use valida_opcodes::{DIV32, SDIV32, MUL32, SUB32, LT32};
use valida_range::MachineWithRangeChip;
use valida_util::pad_to_power_of_two;
use p3_air::VirtualPairCol;
use p3_field::{AbstractField, Field, PrimeField};
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::prelude::*;

pub mod columns;
pub mod stark;

#[derive(Clone)]
pub enum Operation {
    Div32(Word<u8>, Word<u8>, Word<u8>), // (quotient, dividend, divisor)
    SDiv32(Word<u8>, Word<u8>, Word<u8>), //signed
}

#[derive(Default)]
pub struct Div32Chip {
    pub operations: Vec<Operation>,
}

impl<M, SC> Chip<M, SC> for Div32Chip
where
    M: MachineWithGeneralBus<SC::Val>,
    SC: StarkConfig,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<SC::Val> {
        let rows = self
            .operations
            .par_iter()
            .map(|op| self.op_to_row(op))
            .collect::<Vec<_>>();

        let mut trace =
            RowMajorMatrix::new(rows.into_iter().flatten().collect::<Vec<_>>(), NUM_DIV_COLS);

        pad_to_power_of_two::<NUM_DIV_COLS, SC::Val>(&mut trace.values);

        trace
    }

    fn global_sends(&self, machine: &M) -> Vec<Interaction<SC::Val>>{


	let input_1:[VirtualPairCol<SC::Val>;4] = DIV_COL_MAP.input_1.0.map(VirtualPairCol::single_main);
	let input_2 = DIV_COL_MAP.input_2.0.map(VirtualPairCol::single_main);
	let output:[VirtualPairCol<SC::Val>;4] = DIV_COL_MAP.output.0.map(VirtualPairCol::single_main);
	let intermediate_output = DIV_COL_MAP.intermediate_output.0.map(VirtualPairCol::single_main);
	let q:[VirtualPairCol<SC::Val>;4] = DIV_COL_MAP.q.0.map(VirtualPairCol::single_main);

	//check for overflow in input_1 = input_2*output + q by checking that input_1 < q == 0
	let overflow_opcode = VirtualPairCol::constant(SC::Val::from_canonical_u32(LT32));
	let mut add_overflow_fields = vec![overflow_opcode];
	add_overflow_fields.extend(input_1);
	add_overflow_fields.extend(q);
	add_overflow_fields.push(VirtualPairCol::constant(SC::Val::from_canonical_u32(0u32)));	
	let is_real = VirtualPairCol::constant(SC::Val::from_canonical_u32(1u32));
	let overflow_interaction = Interaction {
	    fields: add_overflow_fields,
	    count: is_real,
	    argument_index: machine.general_bus()
	};

	//check for overflow in input_2*output = intermediate_product by checing that intermediate_product < input_2 == 0
	let overflow_opcode = VirtualPairCol::constant(SC::Val::from_canonical_u32(LT32));
	let mut mul_overflow_fields = vec![overflow_opcode];
	mul_overflow_fields.extend(intermediate_output);
	mul_overflow_fields.extend(input_2);
	mul_overflow_fields.push(VirtualPairCol::constant(SC::Val::from_canonical_u32(0u32)));	
	let is_real = VirtualPairCol::constant(SC::Val::from_canonical_u32(1u32));
	let mul_overflow_interaction = Interaction {
	    fields: mul_overflow_fields,
	    count: is_real,
	    argument_index: machine.general_bus()
	};		

	//intermediate_output = input_2*output
	//in the future implement this as mulhs to account for overflow
	let mul_opcode = VirtualPairCol::constant(SC::Val::from_canonical_u32(MUL32));
	let input_2 = DIV_COL_MAP.input_2.0.map(VirtualPairCol::single_main);
	let output = DIV_COL_MAP.output.0.map(VirtualPairCol::single_main);
	let intermediate_output = DIV_COL_MAP.intermediate_output.0.map(VirtualPairCol::single_main);	
	let mut mul_fields = vec![mul_opcode];
	mul_fields.extend(input_2);
	mul_fields.extend(output);
	mul_fields.extend(intermediate_output);
	let is_real = VirtualPairCol::constant(SC::Val::from_canonical_u32(1u32));
	let mul_interaction = Interaction {
	    fields: mul_fields,
	    count: is_real,
	    argument_index: machine.general_bus()
	};


	//input_1 - intermediate_output = q
	let sub_opcode = VirtualPairCol::constant(SC::Val::from_canonical_u32(SUB32));
	let intermediate_output = DIV_COL_MAP.intermediate_output.0.map(VirtualPairCol::single_main);
	let input_1:[VirtualPairCol<SC::Val>;4] = DIV_COL_MAP.input_1.0.map(VirtualPairCol::single_main);
	let q:[VirtualPairCol<SC::Val>;4] = DIV_COL_MAP.q.0.map(VirtualPairCol::single_main);		
	let is_real = VirtualPairCol::constant(SC::Val::from_canonical_u32(1u32));		
	let mut sub_fields = vec![sub_opcode];
	sub_fields.extend(input_1);
	sub_fields.extend(intermediate_output);
	sub_fields.extend(q);
	let sub_interaction= Interaction {
	    fields: sub_fields,
	    count: is_real,
	    argument_index: machine.general_bus()
	};

	//q < output == 1
	let lt_opcode = VirtualPairCol::constant(SC::Val::from_canonical_u32(LT32));
	let q: [VirtualPairCol<SC::Val>;4] = DIV_COL_MAP.q.0.map(VirtualPairCol::single_main);	
	let output = DIV_COL_MAP.output.0.map(VirtualPairCol::single_main);
	let is_real = VirtualPairCol::constant(SC::Val::from_canonical_u32(1u32));	
	let mut lt_fields = vec![lt_opcode];
	lt_fields.extend(q);
	lt_fields.extend(output);
	lt_fields.push(VirtualPairCol::constant(SC::Val::from_canonical_u32(1u32)));
	let lt_interaction= Interaction {
	    fields: lt_fields,
	    count: is_real,
	    argument_index: machine.general_bus()
	};
	vec![overflow_interaction, mul_overflow_interaction, mul_interaction, sub_interaction, lt_interaction]
    }

    fn global_receives(&self, machine: &M) -> Vec<Interaction<SC::Val>> {
        let opcode = VirtualPairCol::new_main(
            vec![
                (DIV_COL_MAP.is_div, SC::Val::from_canonical_u32(DIV32)),
                (DIV_COL_MAP.is_sdiv, SC::Val::from_canonical_u32(SDIV32)),
            ],
            SC::Val::zero(),
        );
        let input_1 = DIV_COL_MAP.input_1.0.map(VirtualPairCol::single_main);
        let input_2 = DIV_COL_MAP.input_2.0.map(VirtualPairCol::single_main);
        let output = DIV_COL_MAP.output.0.map(VirtualPairCol::single_main);

        let mut fields = vec![opcode];
        fields.extend(input_1);
        fields.extend(input_2);
        fields.extend(output);

        let is_real = VirtualPairCol::sum_main(vec![DIV_COL_MAP.is_div, DIV_COL_MAP.is_sdiv]);

        let receive = Interaction {
            fields,
            count: is_real,
            argument_index: machine.general_bus(),
        };
        vec![receive]
    }
}



impl Div32Chip {
    fn op_to_row<F>(&self, op: &Operation) -> [F; NUM_DIV_COLS]
    where
        F: PrimeField,
    {
        let mut row = [F::zero(); NUM_DIV_COLS];
        let cols: &mut Div32Cols<F> = unsafe { transmute(&mut row) };

        match op {
            Operation::Div32(_, _, _) => {
                cols.is_div = F::one();
            }
            Operation::SDiv32(_, _, _) => {
                cols.is_sdiv = F::one();
            }
        }

        // TODO: Fill in other columns.

        row
    }
}

pub trait MachineWithDiv32Chip<F: Field>: MachineWithCpuChip<F> {
    fn div_u32(&self) -> &Div32Chip;
    fn div_u32_mut(&mut self) -> &mut Div32Chip;
}

instructions!(Div32Instruction, SDiv32Instruction);

impl<M, F> Instruction<M, F> for Div32Instruction
where
    M: MachineWithDiv32Chip<F> + MachineWithRangeChip<F, 256>,
    F: Field,
{
    const OPCODE: u32 = DIV32;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let opcode = <Self as Instruction<M, F>>::OPCODE;
        let clk = state.cpu().clock;
        let pc = state.cpu().pc;
        let mut imm: Option<Word<u8>> = None;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let b = state
            .mem_mut()
            .read(clk, read_addr_1, true, pc, opcode, 0, "");
        let c = if ops.is_imm() == 1 {
            let c = (ops.c() as u32).into();
            imm = Some(c);
            c
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state
                .mem_mut()
                .read(clk, read_addr_2, true, pc, opcode, 1, "")
        };

        let a = b / c;
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .div_u32_mut()
            .operations
            .push(Operation::Div32(a, b, c));
        state.cpu_mut().push_bus_op(imm, opcode, ops);

        state.range_check(a);
    }
}

impl<M, F> Instruction<M, F> for SDiv32Instruction
where
    M: MachineWithDiv32Chip<F> + MachineWithRangeChip<F, 256>,
    F: Field,
{
    const OPCODE: u32 = SDIV32;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let opcode = <Self as Instruction<M, F>>::OPCODE;
        let clk = state.cpu().clock;
        let pc = state.cpu().pc;
        let mut imm: Option<Word<u8>> = None;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let b = state
            .mem_mut()
            .read(clk, read_addr_1, true, pc, opcode, 0, "");
        let c = if ops.is_imm() == 1 {
            let c = (ops.c() as u32).into();
            imm = Some(c);
            c
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state
                .mem_mut()
                .read(clk, read_addr_2, true, pc, opcode, 1, "")
        };

        let a = b.sdiv(c);
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .div_u32_mut()
            .operations
            .push(Operation::SDiv32(a, b, c));
        state.cpu_mut().push_bus_op(imm, opcode, ops);

        state.range_check(a);
    }
}
