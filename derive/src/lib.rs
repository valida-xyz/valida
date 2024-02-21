#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{spanned::Spanned, Data, Field, Fields, Ident};

// TODO: now trivial with a single field
struct MachineFields {
    val: Ident,
}

impl Parse for MachineFields {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        let val = content.parse()?;
        Ok(MachineFields { val })
    }
}

#[proc_macro_derive(Machine, attributes(machine_fields, bus, chip, instruction))]
pub fn machine_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_machine(&ast)
}

fn impl_machine(machine: &syn::DeriveInput) -> TokenStream {
    if let Data::Struct(struct_) = &machine.data {
        let fields = match &struct_.fields {
            Fields::Named(named) => named.named.iter().collect(),
            Fields::Unnamed(unnamed) => unnamed.unnamed.iter().collect(),
            Fields::Unit => vec![],
        };

        let instructions = fields
            .iter()
            .filter(|f| f.attrs.iter().any(|a| a.path.is_ident("instruction")))
            .copied()
            .collect::<Vec<_>>();
        let chips = fields
            .iter()
            .filter(|f| f.attrs.iter().any(|a| a.path.is_ident("chip")))
            .copied()
            .collect::<Vec<_>>();

        let machine_fields = machine
            .attrs
            .iter()
            .filter(|a| a.path.segments.len() == 1 && a.path.segments[0].ident == "machine_fields")
            .next()
            .expect("machine_fields attribute required to derive Machine");
        let machine_fields: MachineFields = syn::parse2(machine_fields.tokens.clone())
            .expect("Invalid machine_fields attribute, expected #[machine_fields(<Val>)]");
        let val = &machine_fields.val;

        let name = &machine.ident;
        let run = run_method(machine, &instructions, &val);
        let prove = prove_method(&chips);
        let verify = verify_method(&chips);

        let (impl_generics, ty_generics, where_clause) = machine.generics.split_for_impl();

        let stream = quote! {
            impl #impl_generics Machine<#val> for #name #ty_generics #where_clause {
                #run
                #prove
                #verify
            }
        };

        stream.into()
    } else {
        panic!("Machine derive only supports structs");
    }
}

#[deprecated] // Planning manual impls for now.
#[allow(dead_code)]
fn impl_machine_chip_impl_given_chips(
    machine: &syn::DeriveInput,
    chips: &[&Field],
) -> TokenStream2 {
    let chip_impls = chips.iter().map(|chip| {
        let chip_ty = &chip.ty;
        let tokens = quote!(#chip_ty);
        let chip_impl_name = Ident::new(&format!("MachineWith{}", tokens.to_string()), chip.span());
        let chip_methods = chip_methods(chip);

        let name = &machine.ident;
        let (impl_generics, ty_generics, where_clause) = machine.generics.split_for_impl();

        quote! {
            impl #impl_generics #chip_impl_name for #name #ty_generics #where_clause {
                #chip_methods
            }
        }
    });
    quote! {
        #(#chip_impls)*
    }
}

#[allow(dead_code)]
fn chip_methods(chip: &Field) -> TokenStream2 {
    let mut methods = vec![];
    let chip_name = chip.ident.as_ref().unwrap();
    let chip_name_mut = Ident::new(&format!("{}_mut", chip_name), chip_name.span());
    let chip_type = &chip.ty;
    methods.push(quote! {
        fn #chip_name(&self) -> &#chip_type {
            &self.#chip_name
        }
        fn #chip_name_mut(&mut self) -> &mut #chip_type {
            &mut self.#chip_name
        }
    });
    quote! {
        #(#methods)*
    }
}

fn run_method(machine: &syn::DeriveInput, instructions: &[&Field], val: &Ident) -> TokenStream2 {
    let name = &machine.ident;
    let (_, ty_generics, _) = machine.generics.split_for_impl();

    let opcode_arms = instructions
        .iter()
        .map(|inst| {
            let ty = &inst.ty;
            quote! {
                // TODO: Self instead of #name #ty_generics?
                <#ty as Instruction<#name #ty_generics, #val>>::OPCODE =>
                    #ty::execute_with_advice::<Adv>(self, ops, advice),
            }
        })
        .collect::<TokenStream2>();

    quote! {
        fn run<Adv: ::valida_machine::AdviceProvider>(&mut self, program: &ProgramROM<i32>, advice: &mut Adv) {
            loop {
                // Fetch
                let pc = self.cpu().pc;
                let instruction = program.get_instruction(pc);
                let opcode = instruction.opcode;
                let ops = instruction.operands;

                // Execute
                match opcode {
                    #opcode_arms
                    _ => panic!("Unrecognized opcode: {}", opcode),
                };
                self.read_word(pc as usize);

                // A STOP instruction signals the end of the program
                if opcode == <StopInstruction as Instruction<Self, #val>>::OPCODE {
                    break;
                }
            }

            // Record padded STOP instructions
            let n = self.cpu().clock.next_power_of_two() - self.cpu().clock;
            for _ in 0..n {
                self.read_word(self.cpu().pc as usize);
            }
        }
    }
}

fn prove_method(chips: &[&Field]) -> TokenStream2 {
    let num_chips = chips.len();
    let chip_list = chips
        .iter()
        .map(|chip| {
            let chip_name = chip.ident.as_ref().unwrap();
            quote! {
                alloc::boxed::Box::new(self.#chip_name()),
            }
        })
        .collect::<TokenStream2>();

    let quotient_degree_calls = chips
        .iter()
        .map(|chip| {
            let chip_name = chip.ident.as_ref().unwrap();
            quote! {
                get_log_quotient_degree::<Self, SC, _>(self, self.#chip_name()),
            }
        })
        .collect::<TokenStream2>();

    let compute_quotients = chips
        .iter()
        .enumerate()
        .map(|(i, chip)| {
            let chip_name = chip.ident.as_ref().unwrap();
            quote! {
                #[cfg(debug_assertions)]
                check_constraints::<Self, _, SC>(
                    self,
                    self.#chip_name(),
                    &main_traces[#i],
                    &perm_traces[#i],
                    &perm_challenges,
                );

                // TODO: Needlessly regenerating preprocessed_trace()
                let ppt: Option<RowMajorMatrix<SC::Val>> = self.#chip_name().preprocessed_trace();
                let preprocessed_trace_lde = ppt.map(|trace| preprocessed_trace_ldes.remove(0));

                quotients.push(quotient(
                    self,
                    config,
                    self.#chip_name(),
                    log_degrees[#i],
                    preprocessed_trace_lde,
                    main_trace_ldes.remove(0),
                    perm_trace_ldes.remove(0),
                    cummulative_sums[#i],
                    &perm_challenges,
                    alpha,
                ));
            }
        })
        .collect::<TokenStream2>();

    quote! {
        #[tracing::instrument(name = "prove machine execution", skip_all)]
        fn prove<SC: StarkConfig<Val = F>>(&self, config: &SC) -> ::valida_machine::MachineProof<SC>
        {
            use ::valida_machine::__internal::*;
            use ::valida_machine::__internal::p3_air::{BaseAir};
            use ::valida_machine::__internal::p3_field::{AbstractField};
            use ::valida_machine::__internal::p3_challenger::{CanObserve, FieldChallenger};
            use ::valida_machine::__internal::p3_commit::{Pcs, UnivariatePcs, UnivariatePcsWithLde};
            use ::valida_machine::__internal::p3_matrix::{Matrix, MatrixRowSlices, dense::RowMajorMatrix};
            use ::valida_machine::__internal::p3_util::log2_strict_usize;
            use ::valida_machine::{generate_permutation_trace, MachineProof, ChipProof, Commitments};
            use ::valida_machine::OpenedValues;
            use alloc::vec;
            use alloc::vec::Vec;
            use alloc::boxed::Box;

            let mut chips: [Box<&dyn Chip<Self, SC>>; #num_chips] = [ #chip_list ];
            let log_quotient_degrees: [usize; #num_chips] = [ #quotient_degree_calls ];

            let mut challenger = config.challenger();
            // TODO: Seed challenger with digest of all constraints & trace lengths.
            let pcs = config.pcs();

            let preprocessed_traces: Vec<RowMajorMatrix<SC::Val>> =
                tracing::info_span!("generate preprocessed traces")
                    .in_scope(||
                        chips.par_iter()
                            .flat_map(|chip| chip.preprocessed_trace())
                            .collect::<Vec<_>>()
                    );

            let (preprocessed_commit, preprocessed_data) =
                tracing::info_span!("commit to preprocessed traces")
                    .in_scope(|| pcs.commit_batches(preprocessed_traces.to_vec()));
            challenger.observe(preprocessed_commit.clone());
            let mut preprocessed_trace_ldes = pcs.get_ldes(&preprocessed_data);

            let main_traces: [RowMajorMatrix<SC::Val>; #num_chips] =
                tracing::info_span!("generate main traces")
                    .in_scope(||
                        chips.par_iter()
                            .map(|chip| chip.generate_trace(self))
                            .collect::<Vec<_>>()
                            .try_into().unwrap()
                    );

            let degrees: [usize; #num_chips] = main_traces.iter()
                .map(|trace| trace.height())
                .collect::<Vec<_>>()
                .try_into().unwrap();
            let log_degrees = degrees.map(|d| log2_strict_usize(d));
            let g_subgroups = log_degrees.map(|log_deg| SC::Val::two_adic_generator(log_deg));

            let (main_commit, main_data) = tracing::info_span!("commit to main traces")
                .in_scope(|| pcs.commit_batches(main_traces.to_vec()));
            challenger.observe(main_commit.clone());
            let mut main_trace_ldes = pcs.get_ldes(&main_data);

            let mut perm_challenges = Vec::new();
            for _ in 0..3 {
                perm_challenges.push(challenger.sample_ext_element());
            }

            let perm_traces = tracing::info_span!("generate permutation traces")
                .in_scope(||
                    chips.into_par_iter().enumerate().map(|(i, chip)| {
                        generate_permutation_trace(self, *chip, &main_traces[i], perm_challenges.clone())
                    }).collect::<Vec<_>>()
                );

            let cummulative_sums = perm_traces.iter()
                .map(|trace| trace.row_slice(trace.height() - 1).last().unwrap().clone())
                .collect::<Vec<_>>();

            let (perm_commit, perm_data) = tracing::info_span!("commit to permutation traces")
                .in_scope(|| {
                    let flattened_perm_traces = perm_traces.iter()
                        .map(|trace| trace.flatten_to_base())
                        .collect::<Vec<_>>();
                    pcs.commit_batches(flattened_perm_traces)
                });
            challenger.observe(perm_commit.clone());
            let mut perm_trace_ldes = pcs.get_ldes(&perm_data);

            let alpha: SC::Challenge = challenger.sample_ext_element();

            let mut quotients: Vec<RowMajorMatrix<SC::Val>> = vec![];
            #compute_quotients
            assert_eq!(quotients.len(), #num_chips);
            assert_eq!(log_quotient_degrees.len(), #num_chips);
            let coset_shifts = tracing::debug_span!("coset shift").in_scope(|| {
                let pcs_coset_shift = pcs.coset_shift();
                log_quotient_degrees.map(|log_d| pcs_coset_shift.exp_power_of_2(log_d))
            });
            assert_eq!(coset_shifts.len(), #num_chips);
            let (quotient_commit, quotient_data) = tracing::info_span!("commit to quotient chunks")
                .in_scope(|| pcs.commit_shifted_batches(quotients.to_vec(), &coset_shifts));

            challenger.observe(quotient_commit.clone());

            #[cfg(debug_assertions)]
            check_cumulative_sums(&perm_traces[..]);

            let zeta: SC::Challenge = challenger.sample_ext_element();
            let zeta_and_next: [Vec<SC::Challenge>; #num_chips] =
                g_subgroups.map(|g| vec![zeta, zeta * g]);
            let zeta_exp_quotient_degree: [Vec<SC::Challenge>; #num_chips] =
                log_quotient_degrees.map(|log_deg| vec![zeta.exp_power_of_2(log_deg)]);
            let prover_data_and_points = [
                // TODO: Causes some errors, probably related to the fact that not all chips have preprocessed traces?
                // (&preprocessed_data, zeta_and_next.as_slice()),
                (&main_data, zeta_and_next.as_slice()),
                (&perm_data, zeta_and_next.as_slice()),
                (&quotient_data, zeta_exp_quotient_degree.as_slice()),
            ];
            let (openings, opening_proof) = pcs.open_multi_batches(
               &prover_data_and_points, &mut challenger);

            // TODO: add preprocessed openings
            let [main_openings, perm_openings, quotient_openings] =
                openings.try_into().expect("Should have 3 rounds of openings");

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
                .map(|((((log_degree,  main), perm), quotient), perm_trace)| {
                    // TODO: add preprocessed openings
                    let [preprocessed_local, preprocessed_next] =
                        [vec![], vec![]];

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

                    let cumulative_sum = perm_trace.row_slice(perm_trace.height() - 1).last().unwrap().clone();
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
    }
}

fn verify_method(chips: &[&Field]) -> TokenStream2 {
    let num_chips = chips.len();
    let chip_list = chips
        .iter()
        .map(|chip| {
            let chip_name = chip.ident.as_ref().unwrap();
            quote! {
                alloc::boxed::Box::new(self.#chip_name()),
            }
        })
        .collect::<TokenStream2>();

    let quotient_degree_calls = chips
        .iter()
        .map(|chip| {
            let chip_name = chip.ident.as_ref().unwrap();
            quote! {
                get_log_quotient_degree::<Self, SC, _>(self, self.#chip_name()),
            }
        })
        .collect::<TokenStream2>();

    let verify_constraints = chips
        .iter()
        .enumerate()
        .map(|(i, chip)| {
            let chip_name = chip.ident.as_ref().unwrap();
            quote! {
                verify_constraints::<Self, _, SC>(
                    self,
                    self.#chip_name(),
                    &proof.chip_proofs[#i].opened_values,
                    proof.chip_proofs[#i].cumulative_sum,
                    proof.chip_proofs[#i].log_degree,
                    g_subgroups[#i],
                    zeta,
                    alpha,
                    &perm_challenges
                ).expect(&alloc::format!("Failed to verify constraints on chip {}", #i));
            }
        })
        .collect::<TokenStream2>();

    quote! {
        fn verify<SC: StarkConfig<Val = F>>(
            &self,
            config: &SC,
            proof: &::valida_machine::MachineProof<SC>,
        ) -> core::result::Result<(), ()>
        {
            use ::valida_machine::__internal::*;
            use ::valida_machine::__internal::p3_air::{BaseAir};
            use ::valida_machine::__internal::p3_field::{AbstractField, AbstractExtensionField};
            use ::valida_machine::__internal::p3_challenger::{CanObserve, FieldChallenger};
            use ::valida_machine::__internal::p3_commit::{Pcs, UnivariatePcs, UnivariatePcsWithLde};
            use ::valida_machine::__internal::p3_matrix::Dimensions;
            use ::valida_machine::__internal::p3_util::log2_strict_usize;
            use ::valida_machine::{verify_constraints, MachineProof, ChipProof, Commitments};
            use ::valida_machine::OpenedValues;
            use ::valida_machine::{VerificationError, ProofShapeError, OodEvaluationMismatch};
            use alloc::vec;
            use alloc::vec::Vec;
            use alloc::boxed::Box;


            let mut chips: [Box<&dyn Chip<Self, SC>>; #num_chips] = [ #chip_list ];
            let log_quotient_degrees: [usize; #num_chips] = [ #quotient_degree_calls ];
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
                chips_interactions.iter()
                  .zip(proof.chip_proofs.iter())
                    .map(|(interactions, chip_proof)| Dimensions {
                        width: (interactions.len() + 1) * SC::Challenge::D,
                        height: 1 << chip_proof.log_degree,
                    })
                    .collect::<Vec<_>>(),
                proof.chip_proofs.iter()
                    .zip(log_quotient_degrees)
                    .map(|(chip_proof, log_quotient_deg)| Dimensions {
                        width: log_quotient_deg << SC::Challenge::D,
                        height: 1 << chip_proof.log_degree,
                    })
                    .collect::<Vec<_>>(),
            ];

            // Get the generators of the trace subgroups for each chip.
            let g_subgroups :[SC::Val ; #num_chips] = proof.chip_proofs
                .iter()
                .map(|chip_proof| SC::Val::two_adic_generator(chip_proof.log_degree))
                .collect::<Vec<_>>().try_into().unwrap();

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
            tracing::info_span!("generate preprocessed traces")
                .in_scope(||
                    chips.par_iter()
                        .flat_map(|chip| chip.preprocessed_trace())
                        .collect::<Vec<_>>()
                );

            let (preprocessed_commit, preprocessed_data) =
                tracing::info_span!("commit to preprocessed traces")
                    .in_scope(|| pcs.commit_batches(preprocessed_traces.to_vec()));

            challenger.observe(preprocessed_commit.clone());

            // challenger.observe(preprocessed_commit.clone());
            challenger.observe(main_trace.clone());

            let mut perm_challenges = Vec::new();
            for _ in 0..3 {
                perm_challenges.push(challenger.sample_ext_element::<SC::Challenge>());
            }

            challenger.observe(perm_trace.clone());

            let alpha = challenger.sample_ext_element::<SC::Challenge>();

            challenger.observe(quotient_chunks.clone());

             // Verify the openning proof.
             let zeta: SC::Challenge = challenger.sample_ext_element();
             let zeta_and_next: [Vec<SC::Challenge>; #num_chips] =
                 g_subgroups.map(|g| vec![zeta, zeta * g]);
             let zeta_exp_quotient_degree: [Vec<SC::Challenge>; #num_chips] =
                 log_quotient_degrees.map(|log_deg| vec![zeta.exp_power_of_2(log_deg)]);
            pcs
                .verify_multi_batches(
                    &[
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
            #verify_constraints
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
    }
}

#[proc_macro_derive(AlignedBorrow)]
pub fn aligned_borrow_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    // Get struct name from ast
    let name = &ast.ident;
    let methods = quote! {
        impl<T> Borrow<#name<T>> for [T] {
            fn borrow(&self) -> &#name<T> {
                debug_assert_eq!(self.len(), size_of::<#name<u8>>());
                let (prefix, shorts, _suffix) = unsafe { self.align_to::<#name<T>>() };
                debug_assert!(prefix.is_empty(), "Alignment should match");
                debug_assert_eq!(shorts.len(), 1);
                &shorts[0]
            }
        }

        impl<T> BorrowMut<#name<T>> for [T] {
            fn borrow_mut(&mut self) -> &mut #name<T> {
                debug_assert_eq!(self.len(), size_of::<#name<u8>>());
                let (prefix, shorts, _suffix) = unsafe { self.align_to_mut::<#name<T>>() };
                debug_assert!(prefix.is_empty(), "Alignment should match");
                debug_assert_eq!(shorts.len(), 1);
                &mut shorts[0]
            }
        }
    };
    methods.into()
}
