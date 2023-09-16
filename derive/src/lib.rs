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
use syn::{spanned::Spanned, Data, Field, Fields, Ident, Token};

struct MachineFields {
    base_field: Ident,
    ext_field: Ident,
}

impl Parse for MachineFields {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        let base_field = content.parse()?;
        content.parse::<Token![,]>()?;
        let ext_field = content.parse()?;
        Ok(MachineFields {
            base_field,
            ext_field,
        })
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

        let name = &machine.ident;
        let run = run_method(machine, &instructions);
        let prove = prove_method(&chips);
        let verify = verify_method(&chips);

        let (impl_generics, ty_generics, where_clause) = machine.generics.split_for_impl();

        let machine_fields = machine
            .attrs
            .iter()
            .filter(|a| a.path.segments.len() == 1 && a.path.segments[0].ident == "machine_fields")
            .next()
            .expect("machine_fields attribute required to derive Machine");
        let machine_fields: MachineFields = syn::parse2(machine_fields.tokens.clone()).expect(
            "Invalid machine_fields attribute, expected #[machine_fields(<BaseField>, <ExtField>)]",
        );

        let base_field = &machine_fields.base_field;
        let ext_field = &machine_fields.ext_field;

        let stream = quote! {
            impl #impl_generics Machine for #name #ty_generics #where_clause {
                type F = #base_field;
                type EF = #ext_field;
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

fn run_method(machine: &syn::DeriveInput, instructions: &[&Field]) -> TokenStream2 {
    let name = &machine.ident;
    let (_, ty_generics, _) = machine.generics.split_for_impl();

    let opcode_arms = instructions
        .iter()
        .map(|inst| {
            let ty = &inst.ty;
            quote! {
                <#ty as Instruction<#name #ty_generics>>::OPCODE => {
                    #ty::execute(self, ops);
                }
            }
        })
        .collect::<TokenStream2>();

    quote! {
        fn run(&mut self, program: &ProgramROM<i32>) {
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
                if opcode == <StopInstruction as Instruction<Self>>::OPCODE {
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
    let push_chips = chips
        .iter()
        .map(|chip| {
            let chip_name = chip.ident.as_ref().unwrap();
            quote! {
                chips.push(alloc::boxed::Box::new(self.#chip_name()));
            }
        })
        .collect::<TokenStream2>();

    let prove_starks = chips
        .iter()
        .enumerate()
        .map(|(n, chip)| {
            let chip_name = chip.ident.as_ref().unwrap();
            quote! {
                #[cfg(debug_assertions)]
                check_constraints(
                    self,
                    self.#chip_name(),
                    &main_traces[#n],
                    &perm_traces[#n],
                    &perm_challenges,
                );

                chip_proofs.push(prove(self, config, self.#chip_name(), &mut challenger));
            }
        })
        .collect::<TokenStream2>();

    quote! {
        #[tracing::instrument(name = "prove machine execution", skip_all)]
        fn prove<SC>(&self, config: &SC) -> ::valida_machine::proof::MachineProof<SC>
        where
            SC: ::valida_machine::config::StarkConfig<Val = Self::F, Challenge = Self::EF>,
        {
            use ::valida_machine::__internal::*;
            use ::valida_machine::__internal::p3_challenger::FieldChallenger;
            use ::valida_machine::__internal::p3_commit::{Pcs, MultivariatePcs};
            use ::valida_machine::__internal::p3_matrix::dense::RowMajorMatrix;
            use ::valida_machine::chip::generate_permutation_trace;
            use ::valida_machine::proof::MachineProof;
            use alloc::vec;
            use alloc::vec::Vec;
            use alloc::boxed::Box;

            let mut chips: Vec<Box<&dyn Chip<Self>>> = Vec::new();
            #push_chips

            let mut challenger = config.challenger();

            let main_traces = tracing::info_span!("generate main traces")
                .in_scope(||
                    chips.par_iter().map(|chip| {
                        chip.generate_trace(self)
                    }).collect::<Vec<_>>()
                );

            //// TODO: Want to avoid cloning, but this leads to lifetime issues...
            //// let main_trace_views = main_traces.iter().map(|trace| trace.as_view()).collect();

            let (main_commit, main_data) = tracing::info_span!("commit to main traces")
                .in_scope(||
                    config.pcs().commit_batches(main_traces.clone())
                );
            //// TODO: Have challenger observe main_commit.

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

            //// TODO: Want to avoid cloning, but this leads to lifetime issues...
            //// let perm_trace_views = perm_traces.iter().map(|trace| trace.as_view()).collect();

            //let (perm_commit, perm_data) = config.pcs().commit_batches(perm_traces.clone());
            //// TODO: Have challenger observe perm_commit.

            //let opening_points = &[vec![Self::EF::TWO], vec![Self::EF::TWO]]; // TODO
            //let (openings, opening_proof) = config.pcs().open_multi_batches::<Self::EF, SC::Chal>(
            //    &[&main_data, &perm_data], opening_points, &mut challenger);

            let mut chip_proofs = vec![];
            #prove_starks

            #[cfg(debug_assertions)]
            check_cumulative_sums::<Self>(&perm_traces[..]);

            MachineProof {
                // opening_proof,
                chip_proofs,
                phantom: core::marker::PhantomData,
            }
        }
    }
}

fn verify_method(_chips: &[&Field]) -> TokenStream2 {
    quote! {
        fn verify<SC>(
            proof: &::valida_machine::proof::MachineProof<SC>,
        ) -> core::result::Result<(), ()>
        where
            SC: ::valida_machine::config::StarkConfig<Val = Self::F, Challenge = Self::EF>
        {
            Ok(()) // TODO
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
                // TODO: Double check if this is correct & consider making asserts debug-only.
                let (prefix, shorts, _suffix) = unsafe { self.align_to::<#name<T>>() };
                assert!(prefix.is_empty(), "Data was not aligned");
                assert_eq!(shorts.len(), 1);
                &shorts[0]
            }
        }

        impl<T> BorrowMut<#name<T>> for [T] {
            fn borrow_mut(&mut self) -> &mut #name<T> {
                // TODO: Double check if this is correct & consider making asserts debug-only.
                let (prefix, shorts, _suffix) = unsafe { self.align_to_mut::<#name<T>>() };
                assert!(prefix.is_empty(), "Data was not aligned");
                assert_eq!(shorts.len(), 1);
                &mut shorts[0]
            }
        }
    };
    methods.into()
}
