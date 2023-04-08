use crate::cpu::columns::{CpuCols, NUM_CPU_COLUMNS};
use crate::framework::config::Config;
use crate::framework::constraint_consumer::ConstraintConsumer;
use crate::framework::stark::Stark;
use crate::framework::window::AirWindow;
use core::borrow::Borrow;
use p3_field::field::{Field, FieldExtension};
use p3_field::packed::PackedField;
use p3_field::trivial_extension::TrivialExtension;

pub struct CpuStark;

impl<C: Config> Stark<C> for CpuStark {
    fn columns(&self) -> usize {
        NUM_CPU_COLUMNS
    }

    fn eval_packed_base(
        &self,
        window: AirWindow<<C::F as Field>::Packing>,
        constraints: &mut ConstraintConsumer<C::F, C::FE, <C::F as Field>::Packing>,
    ) {
        eval::<C::F, C::FE, <C::F as Field>::Packing>(window, constraints)
    }

    fn eval_ext(
        &self,
        window: AirWindow<C::FE>,
        constraints: &mut ConstraintConsumer<C::FE, TrivialExtension<C::FE>, C::FE>,
    ) {
        eval::<C::FE, TrivialExtension<C::FE>, C::FE>(window, constraints)
    }
}

fn eval<F, FE, P>(window: AirWindow<P>, constraints: &mut ConstraintConsumer<F, FE, P>)
where
    F: Field,
    FE: FieldExtension<Base = F>,
    P: PackedField<Scalar = F>,
{
    let local: &CpuCols<P> = window.local_row.borrow();
    let next: &CpuCols<P> = window.next_row.borrow();

    // TODO: Move to own function.
    let local_opcode_flags = &local.opcode_flags;
    let increment_pc = local_opcode_flags.is_imm32 + local_opcode_flags.is_bus_op;
    constraints.transition(increment_pc * (local.pc + P::ONES - next.pc));
}
