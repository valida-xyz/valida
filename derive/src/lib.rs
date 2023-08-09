#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{spanned::Spanned, Data, Field, Fields, Ident};

#[proc_macro_derive(Machine, attributes(bus, chip, instruction))]
pub fn machine_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_machine(&ast)
}

fn impl_machine(ast: &syn::DeriveInput) -> TokenStream {
    match &ast.data {
        Data::Struct(struct_) => {
            let fields = match &struct_.fields {
                Fields::Named(named) => named.named.iter().collect(),
                Fields::Unnamed(unnamed) => unnamed.unnamed.iter().collect(),
                Fields::Unit => vec![],
            };
            impl_machine_given_fields(&ast.ident, &fields)
        }
        _ => panic!("Machine derive only supports structs"),
    }
}

fn impl_machine_given_fields(machine: &Ident, fields: &[&Field]) -> TokenStream {
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
    impl_machine_given_instructions_and_chips(machine, &instructions, &chips).into()
}

#[deprecated] // Planning manual impls for now.
#[allow(dead_code)]
fn impl_machine_chip_impl_given_chips(machine: &Ident, chips: &[&Field]) -> TokenStream2 {
    let chip_impls = chips.iter().map(|chip| {
        let chip_ty = &chip.ty;
        let tokens = quote!(#chip_ty);
        let chip_impl_name = Ident::new(&format!("MachineWith{}", tokens.to_string()), chip.span());
        let chip_methods = chip_methods(machine, chip);
        quote! {
            impl #chip_impl_name for #machine {
                #chip_methods
            }
        }
    });
    quote! {
        #(#chip_impls)*
    }
}

fn impl_machine_given_instructions_and_chips(
    machine: &Ident,
    instructions: &[&Field],
    chips: &[&Field],
) -> TokenStream2 {
    let run = run_method(machine, instructions);
    let prove = prove_method(chips);
    let verify = verify_method(chips);
    quote! {
        impl Machine for #machine {
            type F = ::valida_machine::__internal::DefaultField;
            type EF = ::valida_machine::__internal::DefaultExtensionField; // FIXME
            #run
            #prove
            #verify
        }
    }
}

#[allow(dead_code)]
fn chip_methods(_machine: &Ident, chip: &Field) -> TokenStream2 {
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

fn run_method(machine: &Ident, instructions: &[&Field]) -> TokenStream2 {
    let opcode_arms = instructions
        .iter()
        .map(|inst| {
            let ty = &inst.ty;
            quote! {
                <#ty as Instruction<#machine>>::OPCODE => {
                    #ty::execute(self, ops);
                }
            }
        })
        .collect::<TokenStream2>();

    quote! {
        fn run(&mut self, program: ProgramROM<i32>) {
            loop {
                // Fetch
                let pc = self.cpu().pc;
                let instruction = program.get_instruction(pc);
                let opcode = instruction.opcode;
                let ops = instruction.operands;

                // A STOP instruction signals the end of the program
                if opcode == <StopInstruction as Instruction<Self>>::OPCODE {
                    break;
                }

                // Execute
                match opcode {
                    #opcode_arms
                    _ => panic!("Unrecognized opcode: {}", opcode),
                };
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
                    self, self.#chip_name(), &main_traces[#n], &perm_traces[#n], &perm_challenges);

                chip_proofs.push(prove(self, config, self.#chip_name(), &mut challenger));
            }
        })
        .collect::<TokenStream2>();

    quote! {
        fn prove<SC>(&self, config: &SC) -> ::valida_machine::proof::MachineProof<SC>
        where
            SC: ::valida_machine::config::StarkConfig<Val = Self::F, Challenge = Self::EF>,
        {
            use ::valida_machine::__internal::*;
            use ::valida_machine::__internal::p3_challenger::Challenger;
            use ::valida_machine::__internal::p3_commit::{PCS, MultivariatePCS};
            use ::valida_machine::__internal::p3_matrix::dense::RowMajorMatrix;
            use ::valida_machine::chip::generate_permutation_trace;
            use ::valida_machine::proof::MachineProof;
            use alloc::vec;
            use alloc::vec::Vec;
            use alloc::boxed::Box;

            let mut chips: Vec<Box<&dyn Chip<Self>>> = Vec::new();
            #push_chips

            let mut challenger = config.challenger();

            let main_traces = chips.par_iter().map(|chip| {
                chip.generate_trace(self)
            }).collect::<Vec<_>>();

            //// TODO: Want to avoid cloning, but this leads to lifetime issues...
            //// let main_trace_views = main_traces.iter().map(|trace| trace.as_view()).collect();

            //let (main_commit, main_data) = config.pcs().commit_batches(main_traces.clone());
            //// TODO: Have challenger observe main_commit.

            let mut perm_challenges = Vec::new();
            for _ in 0..3 {
                perm_challenges.push(challenger.random_ext_element());
            }

            let perm_traces = chips.into_par_iter().enumerate().map(|(i, chip)| {
                generate_permutation_trace(self, *chip, &main_traces[i], perm_challenges.clone())
            }).collect::<Vec<_>>();

            //// TODO: Want to avoid cloning, but this leads to lifetime issues...
            //// let perm_trace_views = perm_traces.iter().map(|trace| trace.as_view()).collect();

            //let (perm_commit, perm_data) = config.pcs().commit_batches(perm_traces.clone());
            //// TODO: Have challenger observe perm_commit.

            //let opening_points = &[vec![Self::EF::TWO], vec![Self::EF::TWO]]; // TODO
            //let (openings, opening_proof) = config.pcs().open_multi_batches::<Self::EF, SC::Chal>(
            //    &[&main_data, &perm_data], opening_points, &mut challenger);

            let mut chip_proofs = vec![];
            #prove_starks
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
