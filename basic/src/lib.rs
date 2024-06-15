#![allow(unused)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use core::marker::PhantomData;
use p3_air::Air;
use p3_commit::{Pcs, UnivariatePcs, UnivariatePcsWithLde};
use p3_field::{extension::BinomialExtensionField, TwoAdicField};
use p3_field::{AbstractExtensionField, AbstractField, PrimeField32};
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::{Dimensions, Matrix, MatrixRowSlices, MatrixRows};
use p3_maybe_rayon::*;
use p3_util::{log2_ceil_usize, log2_strict_usize};
use valida_alu_u32::{
    add::{Add32Chip, Add32Instruction, MachineWithAdd32Chip},
    bitwise::{
        And32Instruction, Bitwise32Chip, MachineWithBitwise32Chip, Or32Instruction,
        Xor32Instruction,
    },
    com::{Com32Chip, Eq32Instruction, MachineWithCom32Chip, Ne32Instruction},
    div::{Div32Chip, Div32Instruction, MachineWithDiv32Chip, SDiv32Instruction},
    lt::{
        Lt32Chip, Lt32Instruction, Lte32Instruction, MachineWithLt32Chip, Sle32Instruction,
        Slt32Instruction,
    },
    mul::{
        MachineWithMul32Chip, Mul32Chip, Mul32Instruction, Mulhs32Instruction, Mulhu32Instruction,
    },
    shift::{
        MachineWithShift32Chip, Shift32Chip, Shl32Instruction, Shr32Instruction, Sra32Instruction,
    },
    sub::{MachineWithSub32Chip, Sub32Chip, Sub32Instruction},
};
use valida_bus::{
    MachineWithGeneralBus, MachineWithMemBus, MachineWithProgramBus, MachineWithRangeBus8,
};
use valida_cpu::{
    BeqInstruction, BneInstruction, Imm32Instruction, JalInstruction, JalvInstruction,
    Load32Instruction, LoadFpInstruction, LoadS8Instruction, LoadU8Instruction,
    ReadAdviceInstruction, StopInstruction, Store32Instruction, StoreU8Instruction,
};
use valida_cpu::{CpuChip, MachineWithCpuChip};
use valida_machine::PublicRow;
use valida_machine::PublicValues;
use valida_machine::ValidaPublicValues;
use valida_machine::__internal::p3_challenger::{CanObserve, FieldChallenger};
use valida_machine::__internal::{
    check_constraints, check_cumulative_sums, get_log_quotient_degree, quotient,
};
use valida_machine::{
    generate_permutation_trace, verify_constraints, AdviceProvider, BusArgument, Chip, ChipProof,
    Commitments, Instruction, Machine, MachineProof, OpenedValues, ProgramROM, StoppingFlag,
    ValidaAirBuilder,
};
use valida_memory::{MachineWithMemoryChip, MemoryChip};
use valida_output::{MachineWithOutputChip, OutputChip, WriteInstruction};
use valida_program::{MachineWithProgramChip, ProgramChip, ProgramChipTrait};
use valida_range::{MachineWithRangeChip, RangeCheckerChip};
use valida_static_data::{MachineWithStaticDataChip, StaticDataChip};

use p3_maybe_rayon::prelude::*;
use valida_machine::StarkConfig;

#[derive(Default)]
pub struct BasicMachine<F: PrimeField32 + TwoAdicField> {
    // Core instructions
    load32: Load32Instruction,
    loadu8: LoadU8Instruction,
    loads8: LoadS8Instruction,
    store32: Store32Instruction,
    storeu8: StoreU8Instruction,

    jal: JalInstruction,
    jalv: JalvInstruction,
    beq: BeqInstruction,
    bne: BneInstruction,
    imm32: Imm32Instruction,
    stop: StopInstruction,
    loadfp: LoadFpInstruction,

    // ALU instructions
    add32: Add32Instruction,
    sub32: Sub32Instruction,
    mul32: Mul32Instruction,
    mulhs32: Mulhs32Instruction,
    mulhu32: Mulhu32Instruction,
    div32: Div32Instruction,
    sdiv32: SDiv32Instruction,
    shl32: Shl32Instruction,
    shr32: Shr32Instruction,
    sra32: Sra32Instruction,
    lt32: Lt32Instruction,
    lte32: Lte32Instruction,
    and32: And32Instruction,
    or32: Or32Instruction,
    xor32: Xor32Instruction,
    ne32: Ne32Instruction,
    eq32: Eq32Instruction,

    // Input/output instructions
    read: ReadAdviceInstruction,
    write: WriteInstruction,

    // Chips
    cpu: CpuChip,
    program: ProgramChip<F>,
    mem: MemoryChip,
    add_u32: Add32Chip,
    sub_u32: Sub32Chip,
    mul_u32: Mul32Chip,
    div_u32: Div32Chip,
    shift_u32: Shift32Chip,
    lt_u32: Lt32Chip,
    com_u32: Com32Chip,
    bitwise_u32: Bitwise32Chip,
    output: OutputChip,
    range: RangeCheckerChip<256>,
    static_data: StaticDataChip,

    _phantom_sc: PhantomData<fn() -> F>,
}

const NUM_CHIPS: usize = 14;

impl<F: PrimeField32 + TwoAdicField> Machine<F> for BasicMachine<F> {
    fn run<Adv>(&mut self, _program: &ProgramROM<i32>, advice: &mut Adv)
    where
        Adv: AdviceProvider,
    {
        self.initialize_memory();

        loop {
            let step_did_stop = self.step(advice);
            if step_did_stop == StoppingFlag::DidStop {
                break;
            }
        }

        // Record padded STOP instructions
        let n = self.cpu().clock.next_power_of_two() - self.cpu().clock;
        for _ in 0..n {
            self.read_word(self.cpu().pc as usize);
        }
    }

    fn prove<SC>(&self, config: &SC) -> MachineProof<SC>
    where
        SC: StarkConfig<Val = F>,
    {
        let mut chips: [Box<&dyn Chip<Self, SC, Public = ValidaPublicValues<SC::Val>>>; NUM_CHIPS] = [
            Box::new(self.cpu()),
            Box::new(self.program()),
            Box::new(self.mem()),
            Box::new(self.add_u32()),
            Box::new(self.sub_u32()),
            Box::new(self.mul_u32()),
            Box::new(self.div_u32()),
            Box::new(self.shift_u32()),
            Box::new(self.lt_u32()),
            Box::new(self.com_u32()),
            Box::new(self.bitwise_u32()),
            Box::new(self.output()),
            Box::new(self.range()),
            Box::new(self.static_data()),
        ];
        let log_quotient_degrees: [usize; NUM_CHIPS] = [
            get_log_quotient_degree::<Self, SC, _>(self, self.cpu()),
            get_log_quotient_degree::<Self, SC, _>(self, self.program()),
            get_log_quotient_degree::<Self, SC, _>(self, self.mem()),
            get_log_quotient_degree::<Self, SC, _>(self, self.add_u32()),
            get_log_quotient_degree::<Self, SC, _>(self, self.sub_u32()),
            get_log_quotient_degree::<Self, SC, _>(self, self.mul_u32()),
            get_log_quotient_degree::<Self, SC, _>(self, self.div_u32()),
            get_log_quotient_degree::<Self, SC, _>(self, self.shift_u32()),
            get_log_quotient_degree::<Self, SC, _>(self, self.lt_u32()),
            get_log_quotient_degree::<Self, SC, _>(self, self.com_u32()),
            get_log_quotient_degree::<Self, SC, _>(self, self.bitwise_u32()),
            get_log_quotient_degree::<Self, SC, _>(self, self.output()),
            get_log_quotient_degree::<Self, SC, _>(self, self.range()),
            get_log_quotient_degree::<Self, SC, _>(self, self.static_data()),
        ];

        let mut challenger = config.challenger();
        // TODO: Seed challenger with digest of all constraints & trace lengths.
        let pcs = config.pcs();

        let preprocessed_traces: [Option<RowMajorMatrix<SC::Val>>; NUM_CHIPS] =
            tracing::info_span!("generate preprocessed traces").in_scope(|| {
                chips
                    .par_iter()
                    .map(|chip| chip.preprocessed_trace())
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap()
            });

        let has_preprocessed_traces: [bool; NUM_CHIPS] = preprocessed_traces
            .iter()
            .map(Option::is_some)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let (preprocessed_commit, preprocessed_data) =
            tracing::info_span!("commit to preprocessed traces").in_scope(|| {
                pcs.commit_batches(preprocessed_traces.into_iter().flatten().collect())
            });
        challenger.observe(preprocessed_commit.clone());
        let mut preprocessed_trace_ldes_real = pcs.get_ldes(&preprocessed_data).into_iter();

        // add the None's back in so we can iterate through this as we do with the other lde arrays.
        let mut preprocessed_trace_ldes: Vec<_> = has_preprocessed_traces
            .iter()
            .map(|&has_trace| {
                if has_trace {
                    Some(preprocessed_trace_ldes_real.next().unwrap())
                } else {
                    None
                }
            })
            .collect();

        // preprocessed_traces.iter().map(|trace|)

        let mut public_traces: [Option<ValidaPublicValues<SC::Val>>; NUM_CHIPS] =
            tracing::info_span!("generate public values vector").in_scope(|| {
                chips
                    .par_iter()
                    .map(|chip| chip.generate_public_values())
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap()
            });

        let mut public_trace_ldes: Vec<_> = public_traces
            .iter()
            .map(|opt| opt.as_ref().map(|trace| trace.get_ldes(config)))
            .collect();

        let main_traces: [RowMajorMatrix<SC::Val>; NUM_CHIPS] =
            tracing::info_span!("generate main trace").in_scope(|| {
                chips
                    .par_iter()
                    .map(|chip| chip.generate_trace(self))
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap()
            });

        let degrees: [usize; NUM_CHIPS] = main_traces
            .iter()
            .map(|trace| trace.height())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let log_degrees = degrees.map(log2_strict_usize);
        let g_subgroups = log_degrees.map(|log_deg| SC::Val::two_adic_generator(log_deg));

        let (main_commit, main_data) = tracing::info_span!("commit to main traces")
            .in_scope(|| pcs.commit_batches(main_traces.to_vec()));
        challenger.observe(main_commit.clone());
        let mut main_trace_ldes = pcs.get_ldes(&main_data);

        let mut perm_challenges = Vec::new();
        for _ in 0..3 {
            perm_challenges.push(challenger.sample_ext_element());
        }

        let perm_traces: [RowMajorMatrix<SC::Challenge>; NUM_CHIPS] =
            tracing::info_span!("generate permutation traces").in_scope(|| {
                chips
                    .into_par_iter()
                    .enumerate()
                    .map(|(i, chip)| {
                        generate_permutation_trace(
                            self,
                            *chip,
                            &main_traces[i],
                            perm_challenges.clone(),
                        )
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap()
            });

        let cumulative_sums = perm_traces
            .iter()
            .map(|trace| *trace.row_slice(trace.height() - 1).last().unwrap())
            .collect::<Vec<_>>();

        let (perm_commit, perm_data) = tracing::info_span!("commit to permutation traces")
            .in_scope(|| {
                let flattened_perm_traces = perm_traces
                    .iter()
                    .map(|trace| trace.flatten_to_base())
                    .collect::<Vec<_>>();
                pcs.commit_batches(flattened_perm_traces)
            });
        challenger.observe(perm_commit.clone());
        let mut perm_trace_ldes = pcs.get_ldes(&perm_data);

        let alpha: SC::Challenge = challenger.sample_ext_element();

        let mut quotients: Vec<RowMajorMatrix<SC::Val>> = vec![];

        let mut i: usize = 0;

        println!("hi");

        let chip = self.cpu();
        #[cfg(debug_assertions)]
        check_constraints::<Self, _, SC>(
            self,
            chip,
            &main_traces[i],
            &perm_traces[i],
            &perm_challenges,
            &public_traces[i],
        );
        println!("hi2");
        quotients.push(quotient(
            self,
            config,
            chip,
            log_degrees[i],
            preprocessed_trace_ldes.remove(0),
            main_trace_ldes.remove(0),
            perm_trace_ldes.remove(0),
            public_trace_ldes.remove(0),
            cumulative_sums[i],
            &perm_challenges,
            alpha,
        ));
        i += 1;

        println!("main: {:?}", &main_traces[i].height());
        println!("perm: {:?}", &perm_traces[i].height());
        println!("public: {:?}", &public_traces[i].as_ref().unwrap().height());

        let chip = self.program();
        #[cfg(debug_assertions)]
        check_constraints::<Self, _, SC>(
            self,
            chip,
            &main_traces[i],
            &perm_traces[i],
            &perm_challenges,
            &public_traces[i],
        );
        quotients.push(quotient(
            self,
            config,
            chip,
            log_degrees[i],
            preprocessed_trace_ldes.remove(0),
            main_trace_ldes.remove(0),
            perm_trace_ldes.remove(0),
            public_trace_ldes.remove(0),
            cumulative_sums[i],
            &perm_challenges,
            alpha,
        ));
        i += 1;

        let chip = self.mem();
        #[cfg(debug_assertions)]
        check_constraints::<Self, _, SC>(
            self,
            chip,
            &main_traces[i],
            &perm_traces[i],
            &perm_challenges,
            &public_traces[i],
        );
        quotients.push(quotient(
            self,
            config,
            chip,
            log_degrees[i],
            preprocessed_trace_ldes.remove(0),
            main_trace_ldes.remove(0),
            perm_trace_ldes.remove(0),
            public_trace_ldes.remove(0),
            cumulative_sums[i],
            &perm_challenges,
            alpha,
        ));
        i += 1;

        let chip = self.add_u32();
        #[cfg(debug_assertions)]
        check_constraints::<Self, _, SC>(
            self,
            chip,
            &main_traces[i],
            &perm_traces[i],
            &perm_challenges,
            &public_traces[i],
        );
        quotients.push(quotient(
            self,
            config,
            chip,
            log_degrees[i],
            preprocessed_trace_ldes.remove(0),
            main_trace_ldes.remove(0),
            perm_trace_ldes.remove(0),
            public_trace_ldes.remove(0),
            cumulative_sums[i],
            &perm_challenges,
            alpha,
        ));
        i += 1;

        let chip = self.sub_u32();
        #[cfg(debug_assertions)]
        check_constraints::<Self, _, SC>(
            self,
            chip,
            &main_traces[i],
            &perm_traces[i],
            &perm_challenges,
            &public_traces[i],
        );
        quotients.push(quotient(
            self,
            config,
            chip,
            log_degrees[i],
            preprocessed_trace_ldes.remove(0),
            main_trace_ldes.remove(0),
            perm_trace_ldes.remove(0),
            public_trace_ldes.remove(0),
            cumulative_sums[i],
            &perm_challenges,
            alpha,
        ));
        i += 1;

        let chip = self.mul_u32();
        #[cfg(debug_assertions)]
        check_constraints::<Self, _, SC>(
            self,
            chip,
            &main_traces[i],
            &perm_traces[i],
            &perm_challenges,
            &public_traces[i],
        );
        quotients.push(quotient(
            self,
            config,
            chip,
            log_degrees[i],
            preprocessed_trace_ldes.remove(0),
            main_trace_ldes.remove(0),
            perm_trace_ldes.remove(0),
            public_trace_ldes.remove(0),
            cumulative_sums[i],
            &perm_challenges,
            alpha,
        ));
        i += 1;

        let chip = self.div_u32();
        #[cfg(debug_assertions)]
        check_constraints::<Self, _, SC>(
            self,
            chip,
            &main_traces[i],
            &perm_traces[i],
            &perm_challenges,
            &public_traces[i],
        );
        quotients.push(quotient(
            self,
            config,
            chip,
            log_degrees[i],
            preprocessed_trace_ldes.remove(0),
            main_trace_ldes.remove(0),
            perm_trace_ldes.remove(0),
            public_trace_ldes.remove(0),
            cumulative_sums[i],
            &perm_challenges,
            alpha,
        ));
        i += 1;

        let chip = self.shift_u32();
        #[cfg(debug_assertions)]
        check_constraints::<Self, _, SC>(
            self,
            chip,
            &main_traces[i],
            &perm_traces[i],
            &perm_challenges,
            &public_traces[i],
        );
        quotients.push(quotient(
            self,
            config,
            chip,
            log_degrees[i],
            preprocessed_trace_ldes.remove(0),
            main_trace_ldes.remove(0),
            perm_trace_ldes.remove(0),
            public_trace_ldes.remove(0),
            cumulative_sums[i],
            &perm_challenges,
            alpha,
        ));
        i += 1;

        let chip = self.lt_u32();
        #[cfg(debug_assertions)]
        check_constraints::<Self, _, SC>(
            self,
            chip,
            &main_traces[i],
            &perm_traces[i],
            &perm_challenges,
            &public_traces[i],
        );
        quotients.push(quotient(
            self,
            config,
            chip,
            log_degrees[i],
            preprocessed_trace_ldes.remove(0),
            main_trace_ldes.remove(0),
            perm_trace_ldes.remove(0),
            public_trace_ldes.remove(0),
            cumulative_sums[i],
            &perm_challenges,
            alpha,
        ));
        i += 1;

        let chip = self.com_u32();
        #[cfg(debug_assertions)]
        check_constraints::<Self, _, SC>(
            self,
            chip,
            &main_traces[i],
            &perm_traces[i],
            &perm_challenges,
            &public_traces[i],
        );
        quotients.push(quotient(
            self,
            config,
            chip,
            log_degrees[i],
            preprocessed_trace_ldes.remove(0),
            main_trace_ldes.remove(0),
            perm_trace_ldes.remove(0),
            public_trace_ldes.remove(0),
            cumulative_sums[i],
            &perm_challenges,
            alpha,
        ));
        i += 1;

        let chip = self.bitwise_u32();
        #[cfg(debug_assertions)]
        check_constraints::<Self, _, SC>(
            self,
            chip,
            &main_traces[i],
            &perm_traces[i],
            &perm_challenges,
            &public_traces[i],
        );
        quotients.push(quotient(
            self,
            config,
            chip,
            log_degrees[i],
            preprocessed_trace_ldes.remove(0),
            main_trace_ldes.remove(0),
            perm_trace_ldes.remove(0),
            public_trace_ldes.remove(0),
            cumulative_sums[i],
            &perm_challenges,
            alpha,
        ));
        i += 1;

        let chip = self.output();
        #[cfg(debug_assertions)]
        check_constraints::<Self, _, SC>(
            self,
            chip,
            &main_traces[i],
            &perm_traces[i],
            &perm_challenges,
            &public_traces[i],
        );
        quotients.push(quotient(
            self,
            config,
            chip,
            log_degrees[i],
            preprocessed_trace_ldes.remove(0),
            main_trace_ldes.remove(0),
            perm_trace_ldes.remove(0),
            public_trace_ldes.remove(0),
            cumulative_sums[i],
            &perm_challenges,
            alpha,
        ));
        i += 1;

        let chip = self.range();
        #[cfg(debug_assertions)]
        check_constraints::<Self, _, SC>(
            self,
            chip,
            &main_traces[i],
            &perm_traces[i],
            &perm_challenges,
            &public_traces[i],
        );
        quotients.push(quotient(
            self,
            config,
            chip,
            log_degrees[i],
            preprocessed_trace_ldes.remove(0),
            main_trace_ldes.remove(0),
            perm_trace_ldes.remove(0),
            public_trace_ldes.remove(0),
            cumulative_sums[i],
            &perm_challenges,
            alpha,
        ));
        i += 1;

        let chip = self.static_data();
        #[cfg(debug_assertions)]
        check_constraints::<Self, _, SC>(
            self,
            chip,
            &main_traces[i],
            &perm_traces[i],
            &perm_challenges,
            &public_traces[i],
        );
        quotients.push(quotient(
            self,
            config,
            chip,
            log_degrees[i],
            preprocessed_trace_ldes.remove(0),
            main_trace_ldes.remove(0),
            perm_trace_ldes.remove(0),
            public_trace_ldes.remove(0),
            cumulative_sums[i],
            &perm_challenges,
            alpha,
        ));
        i += 1;

        assert_eq!(quotients.len(), NUM_CHIPS);
        assert_eq!(log_quotient_degrees.len(), NUM_CHIPS);
        let coset_shifts = tracing::debug_span!("coset shift").in_scope(|| {
            let pcs_coset_shift = pcs.coset_shift();
            log_quotient_degrees.map(|log_d| pcs_coset_shift.exp_power_of_2(log_d))
        });
        assert_eq!(coset_shifts.len(), NUM_CHIPS);
        let (quotient_commit, quotient_data) = tracing::info_span!("commit to quotient chunks")
            .in_scope(|| pcs.commit_shifted_batches(quotients.to_vec(), &coset_shifts));

        challenger.observe(quotient_commit.clone());

        #[cfg(debug_assertions)]
        check_cumulative_sums(&perm_traces[..]);

        let zeta: SC::Challenge = challenger.sample_ext_element();
        let zeta_and_next: [Vec<SC::Challenge>; NUM_CHIPS] =
            g_subgroups.map(|g| vec![zeta, zeta * g]);
        let zeta_exp_quotient_degree: [Vec<SC::Challenge>; NUM_CHIPS] =
            log_quotient_degrees.map(|log_deg| vec![zeta.exp_power_of_2(log_deg)]);
        let prover_data_and_points = [
            // TODO: Causes some errors, probably related to the fact that not all chips have preprocessed traces?
            // (&preprocessed_data, zeta_and_next.as_slice()),
            (&main_data, zeta_and_next.as_slice()),
            (&perm_data, zeta_and_next.as_slice()),
            (&quotient_data, zeta_exp_quotient_degree.as_slice()),
        ];
        let (openings, opening_proof) =
            pcs.open_multi_batches(&prover_data_and_points, &mut challenger);

        // TODO: add preprocessed openings
        let [main_openings, perm_openings, quotient_openings] = openings
            .try_into()
            .expect("Should have 3 rounds of openings");

        let commitments = Commitments {
            main_trace: main_commit,
            perm_trace: perm_commit,
            quotient_chunks: quotient_commit,
        };

        // TODO: add preprocessed openings
        let chip_proofs = log_degrees
            .iter()
            .zip(main_openings)
            .zip(perm_openings)
            .zip(quotient_openings)
            .zip(perm_traces)
            .map(|((((log_degree, main), perm), quotient), perm_trace)| {
                // TODO: add preprocessed openings
                let [preprocessed_local, preprocessed_next] = [vec![], vec![]];

                let [main_local, main_next] = main.try_into().expect("Should have 2 openings");
                let [perm_local, perm_next] = perm.try_into().expect("Should have 2 openings");
                let [quotient_chunks] = quotient.try_into().expect("Should have 1 opening");

                let opened_values = OpenedValues {
                    preprocessed_local,
                    preprocessed_next,
                    trace_local: main_local,
                    trace_next: main_next,
                    permutation_local: perm_local,
                    permutation_next: perm_next,
                    quotient_chunks,
                };

                let cumulative_sum = perm_trace
                    .row_slice(perm_trace.height() - 1)
                    .last()
                    .unwrap()
                    .clone();
                ChipProof {
                    log_degree: *log_degree,
                    opened_values,
                    cumulative_sum,
                }
            })
            .collect::<Vec<_>>();

        MachineProof {
            commitments,
            opening_proof,
            chip_proofs,
        }
    }

    fn verify<SC>(&self, config: &SC, proof: &MachineProof<SC>) -> Result<(), ()>
    where
        SC: StarkConfig<Val = F>,
    {
        let mut chips: [Box<&dyn Chip<Self, SC, Public = _>>; NUM_CHIPS] = [
            Box::new(self.cpu()),
            Box::new(self.program()),
            Box::new(self.mem()),
            Box::new(self.add_u32()),
            Box::new(self.sub_u32()),
            Box::new(self.mul_u32()),
            Box::new(self.div_u32()),
            Box::new(self.shift_u32()),
            Box::new(self.lt_u32()),
            Box::new(self.com_u32()),
            Box::new(self.bitwise_u32()),
            Box::new(self.output()),
            Box::new(self.range()),
            Box::new(self.static_data()),
        ];

        let log_quotient_degrees: [usize; NUM_CHIPS] = [
            get_log_quotient_degree::<Self, SC, _>(self, self.cpu()),
            get_log_quotient_degree::<Self, SC, _>(self, self.program()),
            get_log_quotient_degree::<Self, SC, _>(self, self.mem()),
            get_log_quotient_degree::<Self, SC, _>(self, self.add_u32()),
            get_log_quotient_degree::<Self, SC, _>(self, self.sub_u32()),
            get_log_quotient_degree::<Self, SC, _>(self, self.mul_u32()),
            get_log_quotient_degree::<Self, SC, _>(self, self.div_u32()),
            get_log_quotient_degree::<Self, SC, _>(self, self.shift_u32()),
            get_log_quotient_degree::<Self, SC, _>(self, self.lt_u32()),
            get_log_quotient_degree::<Self, SC, _>(self, self.com_u32()),
            get_log_quotient_degree::<Self, SC, _>(self, self.bitwise_u32()),
            get_log_quotient_degree::<Self, SC, _>(self, self.output()),
            get_log_quotient_degree::<Self, SC, _>(self, self.range()),
            get_log_quotient_degree::<Self, SC, _>(self, self.static_data()),
        ];

        let mut challenger = config.challenger();
        // TODO: Seed challenger with digest of all constraints & trace lengths.
        let pcs = config.pcs();

        let chips_interactions = chips
            .iter()
            .map(|chip| chip.all_interactions(self))
            .collect::<Vec<_>>();

        let dims = &[
            chips
                .iter()
                .zip(proof.chip_proofs.iter())
                .map(|(chip, chip_proof)| Dimensions {
                    width: chip.trace_width(),
                    height: 1 << chip_proof.log_degree,
                })
                .collect::<Vec<_>>(),
            chips_interactions
                .iter()
                .zip(proof.chip_proofs.iter())
                .map(|(interactions, chip_proof)| Dimensions {
                    width: (interactions.len() + 1) * SC::Challenge::D,
                    height: 1 << chip_proof.log_degree,
                })
                .collect::<Vec<_>>(),
            proof
                .chip_proofs
                .iter()
                .zip(log_quotient_degrees)
                .map(|(chip_proof, log_quotient_deg)| Dimensions {
                    width: log_quotient_deg << SC::Challenge::D,
                    height: 1 << chip_proof.log_degree,
                })
                .collect::<Vec<_>>(),
        ];

        // Get the generators of the trace subgroups for each chip.
        let g_subgroups: [SC::Val; NUM_CHIPS] = proof
            .chip_proofs
            .iter()
            .map(|chip_proof| SC::Val::two_adic_generator(chip_proof.log_degree))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        // TODO: maybe avoid cloning opened values (not sure if possible)
        let mut main_values = vec![];
        let mut perm_values = vec![];
        let mut quotient_values = vec![];

        for chip_proof in proof.chip_proofs.iter() {
            let OpenedValues {
                preprocessed_local,
                preprocessed_next,
                trace_local,
                trace_next,
                permutation_local,
                permutation_next,
                quotient_chunks,
            } = &chip_proof.opened_values;

            main_values.push(vec![trace_local.clone(), trace_next.clone()]);
            perm_values.push(vec![permutation_local.clone(), permutation_next.clone()]);
            quotient_values.push(vec![quotient_chunks.clone()]);
        }

        let chips_opening_values = vec![main_values, perm_values, quotient_values];

        // Observe commitments and get challenges.
        let Commitments {
            main_trace,
            perm_trace,
            quotient_chunks,
        } = &proof.commitments;

        // Compute the commitments to preprocessed traces (TODO: avoid in the future)
        let preprocessed_traces: Vec<RowMajorMatrix<SC::Val>> =
            tracing::info_span!("generate preprocessed traces").in_scope(|| {
                chips
                    .par_iter()
                    .flat_map(|chip| chip.preprocessed_trace())
                    .collect::<Vec<_>>()
            });

        let (preprocessed_commit, preprocessed_data) =
            tracing::info_span!("commit to preprocessed traces")
                .in_scope(|| pcs.commit_batches(preprocessed_traces.to_vec()));

        challenger.observe(preprocessed_commit.clone());

        challenger.observe(main_trace.clone());

        let mut perm_challenges = Vec::new();
        for _ in 0..3 {
            perm_challenges.push(challenger.sample_ext_element::<SC::Challenge>());
        }

        challenger.observe(perm_trace.clone());

        let alpha = challenger.sample_ext_element::<SC::Challenge>();

        challenger.observe(quotient_chunks.clone());

        // Verify the opening proof.
        let zeta: SC::Challenge = challenger.sample_ext_element();
        let zeta_and_next: [Vec<SC::Challenge>; NUM_CHIPS] =
            g_subgroups.map(|g| vec![zeta, zeta * g]);
        let zeta_exp_quotient_degree: [Vec<SC::Challenge>; NUM_CHIPS] =
            log_quotient_degrees.map(|log_deg| vec![zeta.exp_power_of_2(log_deg)]);
        pcs.verify_multi_batches(
            &[
                // TODO: add preprocessed trace
                (main_trace.clone(), zeta_and_next.as_slice()),
                (perm_trace.clone(), zeta_and_next.as_slice()),
                (quotient_chunks.clone(), zeta_exp_quotient_degree.as_slice()),
            ],
            dims,
            chips_opening_values,
            &proof.opening_proof,
            &mut challenger,
        )
        .map_err(|_| ())?;

        // Verify the constraints.
        let mut i = 0;

        let chip = self.cpu();
        let public_values = <CpuChip as Chip<Self, SC>>::generate_public_values(&chip);
        verify_constraints::<Self, _, SC>(
            self,
            chip,
            &proof.chip_proofs[i].opened_values,
            &public_values,
            proof.chip_proofs[i].cumulative_sum,
            proof.chip_proofs[i].log_degree,
            g_subgroups[i],
            zeta,
            alpha,
            &perm_challenges,
        )
        .expect(&format!("Failed to verify constraints on chip {}", i));
        i += 1;

        let chip = self.program();
        let public_values = <ProgramChip<F> as Chip<Self, SC>>::generate_public_values(&chip);
        verify_constraints::<Self, _, SC>(
            self,
            chip,
            &proof.chip_proofs[i].opened_values,
            &public_values,
            proof.chip_proofs[i].cumulative_sum,
            proof.chip_proofs[i].log_degree,
            g_subgroups[i],
            zeta,
            alpha,
            &perm_challenges,
        )
        .expect(&format!("Failed to verify constraints on chip {}", i));
        i += 1;

        let chip = self.mem();
        let public_values = <MemoryChip as Chip<Self, SC>>::generate_public_values(&chip);
        verify_constraints::<Self, _, SC>(
            self,
            chip,
            &proof.chip_proofs[i].opened_values,
            &public_values,
            proof.chip_proofs[i].cumulative_sum,
            proof.chip_proofs[i].log_degree,
            g_subgroups[i],
            zeta,
            alpha,
            &perm_challenges,
        )
        .expect(&format!("Failed to verify constraints on chip {}", i));
        i += 1;

        let chip = self.add_u32();
        let public_values = <Add32Chip as Chip<Self, SC>>::generate_public_values(&chip);
        verify_constraints::<Self, _, SC>(
            self,
            chip,
            &proof.chip_proofs[i].opened_values,
            &public_values,
            proof.chip_proofs[i].cumulative_sum,
            proof.chip_proofs[i].log_degree,
            g_subgroups[i],
            zeta,
            alpha,
            &perm_challenges,
        )
        .expect(&format!("Failed to verify constraints on chip {}", i));
        i += 1;

        let chip = self.sub_u32();
        let public_values = <Sub32Chip as Chip<Self, SC>>::generate_public_values(&chip);
        verify_constraints::<Self, _, SC>(
            self,
            chip,
            &proof.chip_proofs[i].opened_values,
            &public_values,
            proof.chip_proofs[i].cumulative_sum,
            proof.chip_proofs[i].log_degree,
            g_subgroups[i],
            zeta,
            alpha,
            &perm_challenges,
        )
        .expect(&format!("Failed to verify constraints on chip {}", i));
        i += 1;

        let chip = self.mul_u32();
        let public_values = <Mul32Chip as Chip<Self, SC>>::generate_public_values(&chip);
        verify_constraints::<Self, _, SC>(
            self,
            chip,
            &proof.chip_proofs[i].opened_values,
            &public_values,
            proof.chip_proofs[i].cumulative_sum,
            proof.chip_proofs[i].log_degree,
            g_subgroups[i],
            zeta,
            alpha,
            &perm_challenges,
        )
        .expect(&format!("Failed to verify constraints on chip {}", i));
        i += 1;

        let chip = self.div_u32();
        let public_values = <Div32Chip as Chip<Self, SC>>::generate_public_values(&chip);
        verify_constraints::<Self, _, SC>(
            self,
            chip,
            &proof.chip_proofs[i].opened_values,
            &public_values,
            proof.chip_proofs[i].cumulative_sum,
            proof.chip_proofs[i].log_degree,
            g_subgroups[i],
            zeta,
            alpha,
            &perm_challenges,
        )
        .expect(&format!("Failed to verify constraints on chip {}", i));
        i += 1;

        let chip = self.shift_u32();
        let public_values = <Shift32Chip as Chip<Self, SC>>::generate_public_values(&chip);
        verify_constraints::<Self, _, SC>(
            self,
            chip,
            &proof.chip_proofs[i].opened_values,
            &public_values,
            proof.chip_proofs[i].cumulative_sum,
            proof.chip_proofs[i].log_degree,
            g_subgroups[i],
            zeta,
            alpha,
            &perm_challenges,
        )
        .expect(&format!("Failed to verify constraints on chip {}", i));
        i += 1;

        let chip = self.lt_u32();
        let public_values = <Lt32Chip as Chip<Self, SC>>::generate_public_values(&chip);
        verify_constraints::<Self, _, SC>(
            self,
            chip,
            &proof.chip_proofs[i].opened_values,
            &public_values,
            proof.chip_proofs[i].cumulative_sum,
            proof.chip_proofs[i].log_degree,
            g_subgroups[i],
            zeta,
            alpha,
            &perm_challenges,
        )
        .expect(&format!("Failed to verify constraints on chip {}", i));
        i += 1;

        let chip = self.com_u32();
        let public_values = <Com32Chip as Chip<Self, SC>>::generate_public_values(&chip);
        verify_constraints::<Self, _, SC>(
            self,
            chip,
            &proof.chip_proofs[i].opened_values,
            &public_values,
            proof.chip_proofs[i].cumulative_sum,
            proof.chip_proofs[i].log_degree,
            g_subgroups[i],
            zeta,
            alpha,
            &perm_challenges,
        )
        .expect(&format!("Failed to verify constraints on chip {}", i));
        i += 1;

        let chip = self.bitwise_u32();
        let public_values = <Bitwise32Chip as Chip<Self, SC>>::generate_public_values(&chip);
        verify_constraints::<Self, _, SC>(
            self,
            chip,
            &proof.chip_proofs[i].opened_values,
            &public_values,
            proof.chip_proofs[i].cumulative_sum,
            proof.chip_proofs[i].log_degree,
            g_subgroups[i],
            zeta,
            alpha,
            &perm_challenges,
        )
        .expect(&format!("Failed to verify constraints on chip {}", i));
        i += 1;

        let chip = self.output();
        let public_values = <OutputChip as Chip<Self, SC>>::generate_public_values(&chip);
        verify_constraints::<Self, _, SC>(
            self,
            chip,
            &proof.chip_proofs[i].opened_values,
            &public_values,
            proof.chip_proofs[i].cumulative_sum,
            proof.chip_proofs[i].log_degree,
            g_subgroups[i],
            zeta,
            alpha,
            &perm_challenges,
        )
        .expect(&format!("Failed to verify constraints on chip {}", i));
        i += 1;

        let chip = self.range();
        let public_values =
            <RangeCheckerChip<256> as Chip<Self, SC>>::generate_public_values(&chip);
        verify_constraints::<Self, _, SC>(
            self,
            chip,
            &proof.chip_proofs[i].opened_values,
            &public_values,
            proof.chip_proofs[i].cumulative_sum,
            proof.chip_proofs[i].log_degree,
            g_subgroups[i],
            zeta,
            alpha,
            &perm_challenges,
        )
        .expect(&format!("Failed to verify constraints on chip {}", i));
        i += 1;

        let chip = self.static_data();
        let public_values = <StaticDataChip as Chip<Self, SC>>::generate_public_values(&chip);
        verify_constraints::<Self, _, SC>(
            self,
            chip,
            &proof.chip_proofs[i].opened_values,
            &public_values,
            proof.chip_proofs[i].cumulative_sum,
            proof.chip_proofs[i].log_degree,
            g_subgroups[i],
            zeta,
            alpha,
            &perm_challenges,
        )
        .expect(&format!("Failed to verify constraints on chip {}", i));
        i += 1;

        // Verify that the cumulative_sum sums add up to zero.
        let sum: SC::Challenge = proof
            .chip_proofs
            .iter()
            .map(|chip_proof| chip_proof.cumulative_sum)
            .sum();

        if sum != SC::Challenge::zero() {
            return Err(());
        }

        Ok(())
    }

    fn step<Adv>(&mut self, advice: &mut Adv) -> StoppingFlag
    where
        Adv: AdviceProvider,
    {
        // Fetch
        let pc = self.cpu().pc;
        let instruction = self.program.program_rom().get_instruction(pc);
        let opcode = instruction.opcode;
        let ops = instruction.operands;

        // Execute
        match opcode {
            <Load32Instruction as Instruction<Self, F>>::OPCODE => {
                Load32Instruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <LoadU8Instruction as Instruction<Self, F>>::OPCODE => {
                LoadU8Instruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <LoadS8Instruction as Instruction<Self, F>>::OPCODE => {
                LoadS8Instruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <Store32Instruction as Instruction<Self, F>>::OPCODE => {
                Store32Instruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <StoreU8Instruction as Instruction<Self, F>>::OPCODE => {
                StoreU8Instruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <JalInstruction as Instruction<Self, F>>::OPCODE => {
                JalInstruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <JalvInstruction as Instruction<Self, F>>::OPCODE => {
                JalvInstruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <BeqInstruction as Instruction<Self, F>>::OPCODE => {
                BeqInstruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <BneInstruction as Instruction<Self, F>>::OPCODE => {
                BneInstruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <Imm32Instruction as Instruction<Self, F>>::OPCODE => {
                Imm32Instruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <StopInstruction as Instruction<Self, F>>::OPCODE => {
                StopInstruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <LoadFpInstruction as Instruction<Self, F>>::OPCODE => {
                LoadFpInstruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <Add32Instruction as Instruction<Self, F>>::OPCODE => {
                Add32Instruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <Sub32Instruction as Instruction<Self, F>>::OPCODE => {
                Sub32Instruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <Mul32Instruction as Instruction<Self, F>>::OPCODE => {
                Mul32Instruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <Mulhs32Instruction as Instruction<Self, F>>::OPCODE => {
                Mulhs32Instruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <Mulhu32Instruction as Instruction<Self, F>>::OPCODE => {
                Mulhu32Instruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <Div32Instruction as Instruction<Self, F>>::OPCODE => {
                Div32Instruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <SDiv32Instruction as Instruction<Self, F>>::OPCODE => {
                SDiv32Instruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <Shl32Instruction as Instruction<Self, F>>::OPCODE => {
                Shl32Instruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <Shr32Instruction as Instruction<Self, F>>::OPCODE => {
                Shr32Instruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <Sra32Instruction as Instruction<Self, F>>::OPCODE => {
                Sra32Instruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <Lt32Instruction as Instruction<Self, F>>::OPCODE => {
                Lt32Instruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <Lte32Instruction as Instruction<Self, F>>::OPCODE => {
                Lte32Instruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <Slt32Instruction as Instruction<Self, F>>::OPCODE => {
                Slt32Instruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <Sle32Instruction as Instruction<Self, F>>::OPCODE => {
                Sle32Instruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <And32Instruction as Instruction<Self, F>>::OPCODE => {
                And32Instruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <Or32Instruction as Instruction<Self, F>>::OPCODE => {
                Or32Instruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <Xor32Instruction as Instruction<Self, F>>::OPCODE => {
                Xor32Instruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <Ne32Instruction as Instruction<Self, F>>::OPCODE => {
                Ne32Instruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <Eq32Instruction as Instruction<Self, F>>::OPCODE => {
                Eq32Instruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <ReadAdviceInstruction as Instruction<Self, F>>::OPCODE => {
                ReadAdviceInstruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            <WriteInstruction as Instruction<Self, F>>::OPCODE => {
                WriteInstruction::execute_with_advice::<Adv>(self, ops, advice)
            }
            _ => panic!("Unrecognized opcode: {}, pc = {}", opcode, pc),
        };
        self.read_word(pc as usize);

        // A STOP instruction signals the end of the program
        if opcode == <StopInstruction as Instruction<Self, F>>::OPCODE {
            StoppingFlag::DidStop
        } else {
            StoppingFlag::DidNotStop
        }
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithGeneralBus<F> for BasicMachine<F> {
    fn general_bus(&self) -> BusArgument {
        BusArgument::Global(0)
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithProgramBus<F> for BasicMachine<F> {
    fn program_bus(&self) -> BusArgument {
        BusArgument::Global(1)
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithMemBus<F> for BasicMachine<F> {
    fn mem_bus(&self) -> BusArgument {
        BusArgument::Global(2)
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithRangeBus8<F> for BasicMachine<F> {
    fn range_bus(&self) -> BusArgument {
        BusArgument::Global(3)
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithCpuChip<F> for BasicMachine<F> {
    fn cpu(&self) -> &CpuChip {
        &self.cpu
    }

    fn cpu_mut(&mut self) -> &mut CpuChip {
        &mut self.cpu
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithProgramChip<F> for BasicMachine<F> {
    fn program(&self) -> &ProgramChip<F> {
        &self.program
    }

    fn program_mut(&mut self) -> &mut ProgramChip<F> {
        &mut self.program
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithMemoryChip<F> for BasicMachine<F> {
    fn mem(&self) -> &MemoryChip {
        &self.mem
    }

    fn mem_mut(&mut self) -> &mut MemoryChip {
        &mut self.mem
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithAdd32Chip<F> for BasicMachine<F> {
    fn add_u32(&self) -> &Add32Chip {
        &self.add_u32
    }

    fn add_u32_mut(&mut self) -> &mut Add32Chip {
        &mut self.add_u32
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithSub32Chip<F> for BasicMachine<F> {
    fn sub_u32(&self) -> &Sub32Chip {
        &self.sub_u32
    }

    fn sub_u32_mut(&mut self) -> &mut Sub32Chip {
        &mut self.sub_u32
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithMul32Chip<F> for BasicMachine<F> {
    fn mul_u32(&self) -> &Mul32Chip {
        &self.mul_u32
    }

    fn mul_u32_mut(&mut self) -> &mut Mul32Chip {
        &mut self.mul_u32
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithDiv32Chip<F> for BasicMachine<F> {
    fn div_u32(&self) -> &Div32Chip {
        &self.div_u32
    }

    fn div_u32_mut(&mut self) -> &mut Div32Chip {
        &mut self.div_u32
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithBitwise32Chip<F> for BasicMachine<F> {
    fn bitwise_u32(&self) -> &Bitwise32Chip {
        &self.bitwise_u32
    }

    fn bitwise_u32_mut(&mut self) -> &mut Bitwise32Chip {
        &mut self.bitwise_u32
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithLt32Chip<F> for BasicMachine<F> {
    fn lt_u32(&self) -> &Lt32Chip {
        &self.lt_u32
    }

    fn lt_u32_mut(&mut self) -> &mut Lt32Chip {
        &mut self.lt_u32
    }
}
impl<F: PrimeField32 + TwoAdicField> MachineWithCom32Chip<F> for BasicMachine<F> {
    fn com_u32(&self) -> &Com32Chip {
        &self.com_u32
    }

    fn com_u32_mut(&mut self) -> &mut Com32Chip {
        &mut self.com_u32
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithShift32Chip<F> for BasicMachine<F> {
    fn shift_u32(&self) -> &Shift32Chip {
        &self.shift_u32
    }

    fn shift_u32_mut(&mut self) -> &mut Shift32Chip {
        &mut self.shift_u32
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithOutputChip<F> for BasicMachine<F> {
    fn output(&self) -> &OutputChip {
        &self.output
    }

    fn output_mut(&mut self) -> &mut OutputChip {
        &mut self.output
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithRangeChip<F, 256> for BasicMachine<F> {
    fn range(&self) -> &RangeCheckerChip<256> {
        &self.range
    }

    fn range_mut(&mut self) -> &mut RangeCheckerChip<256> {
        &mut self.range
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithStaticDataChip<F> for BasicMachine<F> {
    fn static_data(&self) -> &StaticDataChip {
        &self.static_data
    }

    fn static_data_mut(&mut self) -> &mut StaticDataChip {
        &mut self.static_data
    }
}
