use core::marker::PhantomData;
use std::mem::transmute;

use p3_air::{Air, BaseAir, VirtualPairCol};
use p3_matrix::{
    dense::{RowMajorMatrix, RowMajorMatrixView},
    Matrix, MatrixRowSlices, MatrixRows,
};

use valida_machine::{
    BusArgument, Chip, Interaction, Machine, StarkConfig, ValidaAirBuilder, ValidaPublicValues,
    __internal::p3_field::AbstractField,
};

use crate::columns::{LookupCols, LOOKUP_COL_MAP, NUM_LOOKUP_COLS};

pub mod columns;
pub mod stark;
pub trait LookupTable<F>
where
    F: AbstractField,
{
    type M<'a>: MatrixRowSlices<F>
    where
        Self: 'a;

    fn lookup_type(&self) -> LookupType;
    fn table(&self) -> Self::M<'_>;
}

#[derive(Clone, Copy)]
pub enum LookupType {
    Public,
    Preprocessed,
    Private,
}

pub struct LookupChip<L, F> {
    table: L,
    pub counts: Vec<usize>,
    pub bus: BusArgument,
    pub _phantom: PhantomData<F>,
}

impl<L, F> LookupChip<L, F>
where
    L: LookupTable<F>,
    F: AbstractField,
{
    pub fn new(table: L, bus: BusArgument) -> Self {
        Self {
            table,
            counts: vec![],
            bus,
            _phantom: PhantomData,
        }
    }
    pub fn lookup_type(&self) -> LookupType {
        self.table.lookup_type()
    }
    pub fn table(&self) -> L::M<'_> {
        self.table.table()
    }
}

impl<M, SC, L> Chip<M, SC> for LookupChip<L, SC::Val>
where
    M: Machine<SC::Val>,
    SC: StarkConfig,
    L: LookupTable<SC::Val> + Sync,
{
    type Public = ValidaPublicValues<SC::Val>;

    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<SC::Val> {
        let height = self.table().height();
        debug_assert_eq!(self.counts.len(), height);
        match self.lookup_type() {
            LookupType::Private => {
                let table_width = self.table().width();
                let width = NUM_LOOKUP_COLS + table_width;
                // let mut rows = Vec::with_capacity(height);
                let mut rows = self
                    .counts
                    .iter()
                    .enumerate()
                    .map(|(n, count)| {
                        let mut arg_row = [SC::Val::zero(); NUM_LOOKUP_COLS];
                        let cols: &mut LookupCols<SC::Val> = unsafe { transmute(&mut arg_row) };
                        cols.mult = SC::Val::from_canonical_usize(*count);
                        let row: Vec<_> = arg_row.into_iter().chain(self.table().row(n)).collect();
                        row
                    })
                    .flatten()
                    .collect::<Vec<_>>();

                rows.resize(rows.len().next_power_of_two() * width, SC::Val::zero());
                RowMajorMatrix::new(rows, width)
            }
            _ => {
                let mut rows = self
                    .counts
                    .iter()
                    .map(|count| {
                        let mut row = [SC::Val::zero(); NUM_LOOKUP_COLS];
                        let cols: &mut LookupCols<SC::Val> = unsafe { transmute(&mut row) };
                        cols.mult = SC::Val::from_canonical_usize(*count);
                        row
                    })
                    .flatten()
                    .collect::<Vec<_>>();
                rows.resize(
                    rows.len().next_power_of_two() * NUM_LOOKUP_COLS,
                    SC::Val::zero(),
                );
                RowMajorMatrix::new(rows, NUM_LOOKUP_COLS)
            }
        }
    }

    fn generate_public_values(&self) -> Option<Self::Public> {
        match self.lookup_type() {
            LookupType::Public => {
                let public_trace = self.table().to_row_major_matrix();
                Some(ValidaPublicValues::PublicTrace(public_trace))
            }
            _ => None,
        }
    }

    fn global_receives(&self, _machine: &M) -> Vec<Interaction<SC::Val>> {
        let make_column = |i| match self.lookup_type() {
            LookupType::Preprocessed => VirtualPairCol::single_preprocessed(i),
            LookupType::Private => VirtualPairCol::single_main(i + NUM_LOOKUP_COLS),
            LookupType::Public => VirtualPairCol::single_public(i),
        };

        let fields = (0..self.table().width()).map(make_column).collect();
        let receives = Interaction {
            fields,
            count: VirtualPairCol::single_main(LOOKUP_COL_MAP.mult),
            argument_index: self.bus,
        };
        vec![receives]
        //let pc = VirtualPairCol::single_preprocessed(PREPROCESSED_COL_MAP.pc);
        // let opcode = VirtualPairCol::single_preprocessed(PREPROCESSED_COL_MAP.opcode);
        // let mut fields = vec![pc, opcode];
        // fields.extend(
        //     PREPROCESSED_COL_MAP
        //         .operands
        //         .0
        //         .iter()
        //         .map(|op| VirtualPairCol::single_preprocessed(*op)),
        // );
        // let receives = Interaction {
        //     fields,
        //     count: VirtualPairCol::single_main(COL_MAP.multiplicity),
        //     argument_index: machine.program_bus(),
        // };
        // vec![receives]
        // vec![]
    }
}
