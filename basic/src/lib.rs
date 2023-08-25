#![no_std]
#![allow(unused)]

extern crate alloc;

use valida_alu_u32::{
    add::{Add32Chip, Add32Instruction, MachineWithAdd32Chip},
    bitwise::{
        And32Instruction, Bitwise32Chip, MachineWithBitwise32Chip, Or32Instruction,
        Xor32Instruction,
    },
    div::{Div32Chip, Div32Instruction, MachineWithDiv32Chip},
    lt::{Lt32Chip, Lt32Instruction, MachineWithLt32Chip},
    mul::{MachineWithMul32Chip, Mul32Chip, Mul32Instruction},
    shift::{MachineWithShift32Chip, Shift32Chip, Shl32Instruction, Shr32Instruction},
    sub::{MachineWithSub32Chip, Sub32Chip, Sub32Instruction},
};
use valida_bus::{MachineWithGeneralBus, MachineWithMemBus, MachineWithRangeBus8};
use valida_cpu::{
    BeqInstruction, BneInstruction, Imm32Instruction, JalInstruction, JalvInstruction,
    Load32Instruction, ReadAdviceInstruction, StopInstruction, Store32Instruction,
};
use valida_cpu::{CpuChip, MachineWithCpuChip};
use valida_derive::Machine;
use valida_machine::{
    AbstractExtensionField, AbstractField, BusArgument, Chip, Instruction, Machine, ProgramROM,
};
use valida_memory::{MachineWithMemoryChip, MemoryChip};
use valida_output::{MachineWithOutputChip, OutputChip, WriteInstruction};
use valida_range::{MachineWithRangeChip, RangeCheckerChip};

use p3_maybe_rayon::*;

#[derive(Machine, Default)]
pub struct BasicMachine {
    // Core instructions
    #[instruction]
    load32: Load32Instruction,
    #[instruction]
    store32: Store32Instruction,
    #[instruction]
    jal: JalInstruction,
    #[instruction]
    jalv: JalvInstruction,
    #[instruction]
    beq: BeqInstruction,
    #[instruction]
    bne: BneInstruction,
    #[instruction]
    imm32: Imm32Instruction,
    #[instruction]
    stop: StopInstruction,

    // ALU instructions
    #[instruction(add_u32)]
    add32: Add32Instruction,
    #[instruction(add_u32)]
    sub32: Sub32Instruction,
    #[instruction(mul_u32)]
    mul32: Mul32Instruction,
    #[instruction(div_u32)]
    div32: Div32Instruction,
    #[instruction(shift_u32)]
    shl32: Shl32Instruction,
    #[instruction(shift_u32)]
    shr32: Shr32Instruction,
    #[instruction(lt_u32)]
    lt32: Lt32Instruction,
    #[instruction(bitwise_u32)]
    and32: And32Instruction,
    #[instruction(bitwise_u32)]
    or32: Or32Instruction,
    #[instruction(bitwise_u32)]
    xor32: Xor32Instruction,

    // Input/output instructions
    #[instruction]
    read: ReadAdviceInstruction,
    #[instruction(output)]
    write: WriteInstruction,

    #[chip]
    cpu: CpuChip,
    #[chip]
    mem: MemoryChip,
    #[chip]
    add_u32: Add32Chip,
    #[chip]
    sub_u32: Sub32Chip,
    #[chip]
    mul_u32: Mul32Chip,
    #[chip]
    div_u32: Div32Chip,
    #[chip]
    shift_u32: Shift32Chip,
    #[chip]
    lt_u32: Lt32Chip,
    #[chip]
    bitwise_u32: Bitwise32Chip,
    #[chip]
    output: OutputChip,
    #[chip]
    range: RangeCheckerChip, // TODO: Specify 8-bit RC chip
}

impl MachineWithGeneralBus for BasicMachine {
    fn general_bus(&self) -> BusArgument {
        BusArgument::Global(0)
    }
}

impl MachineWithMemBus for BasicMachine {
    fn mem_bus(&self) -> BusArgument {
        BusArgument::Global(1)
    }
}

impl MachineWithRangeBus8 for BasicMachine {
    fn range_bus(&self) -> BusArgument {
        BusArgument::Global(2)
    }
}

impl MachineWithCpuChip for BasicMachine {
    fn cpu(&self) -> &CpuChip {
        &self.cpu
    }

    fn cpu_mut(&mut self) -> &mut CpuChip {
        &mut self.cpu
    }
}

impl MachineWithMemoryChip for BasicMachine {
    fn mem(&self) -> &MemoryChip {
        &self.mem
    }

    fn mem_mut(&mut self) -> &mut MemoryChip {
        &mut self.mem
    }
}

impl MachineWithAdd32Chip for BasicMachine {
    fn add_u32(&self) -> &Add32Chip {
        &self.add_u32
    }

    fn add_u32_mut(&mut self) -> &mut Add32Chip {
        &mut self.add_u32
    }
}

impl MachineWithSub32Chip for BasicMachine {
    fn sub_u32(&self) -> &Sub32Chip {
        &self.sub_u32
    }

    fn sub_u32_mut(&mut self) -> &mut Sub32Chip {
        &mut self.sub_u32
    }
}

impl MachineWithMul32Chip for BasicMachine {
    fn mul_u32(&self) -> &Mul32Chip {
        &self.mul_u32
    }

    fn mul_u32_mut(&mut self) -> &mut Mul32Chip {
        &mut self.mul_u32
    }
}

impl MachineWithDiv32Chip for BasicMachine {
    fn div_u32(&self) -> &Div32Chip {
        &self.div_u32
    }

    fn div_u32_mut(&mut self) -> &mut Div32Chip {
        &mut self.div_u32
    }
}

impl MachineWithBitwise32Chip for BasicMachine {
    fn bitwise_u32(&self) -> &Bitwise32Chip {
        &self.bitwise_u32
    }

    fn bitwise_u32_mut(&mut self) -> &mut Bitwise32Chip {
        &mut self.bitwise_u32
    }
}

impl MachineWithLt32Chip for BasicMachine {
    fn lt_u32(&self) -> &Lt32Chip {
        &self.lt_u32
    }

    fn lt_u32_mut(&mut self) -> &mut Lt32Chip {
        &mut self.lt_u32
    }
}

impl MachineWithShift32Chip for BasicMachine {
    fn shift_u32(&self) -> &Shift32Chip {
        &self.shift_u32
    }

    fn shift_u32_mut(&mut self) -> &mut Shift32Chip {
        &mut self.shift_u32
    }
}

impl MachineWithOutputChip for BasicMachine {
    fn output(&self) -> &OutputChip {
        &self.output
    }

    fn output_mut(&mut self) -> &mut OutputChip {
        &mut self.output
    }
}

impl MachineWithRangeChip for BasicMachine {
    fn range(&self) -> &RangeCheckerChip {
        &self.range
    }

    fn range_mut(&mut self) -> &mut RangeCheckerChip {
        &mut self.range
    }
}
