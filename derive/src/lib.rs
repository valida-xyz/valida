#![no_std]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use proc_macro::{TokenStream};
use quote::{quote};
use syn::{Data, Field, Fields, Ident};
use proc_macro2::{TokenStream as TokenStream2};

#[proc_macro_derive(Machine, attributes(instruction, chip))]
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
    impl_machine_given_instructions_and_chips(machine, &instructions, &chips)
}

fn impl_machine_given_instructions_and_chips(
    machine: &Ident,
    instructions: &[&Field],
    chips: &[&Field],
) -> TokenStream {
    let run = run_method(machine, instructions);
    let prove = prove_method();
    let verify = verify_method();
    let gen = quote! {
        impl Machine for #machine {
            type F = ::valida_machine::DefaultField;
            #run
            #prove
            #verify
        }
    };
    gen.into()
}

fn run_method(machine: &Ident, instructions: &[&Field]) -> TokenStream2 {
    let opcode_arms = instructions.iter().map(|inst| {
        let ident = &inst.ident;
        let ty = &inst.ty;
        quote! {
            <#ty as Instruction<#machine>>::OPCODE => {
                #ty::execute(self);
            }
        }
    }).collect::<TokenStream2>();
    quote! {
        fn run(&mut self) {
            loop {
                let opcode: u32 = 0u32; // TODO
                match opcode {
                    #opcode_arms
                    _ => todo!(),
                };
            }
        }
    }
}

fn prove_method() -> TokenStream2 {
    quote! { fn prove(&self) {} }
}

fn verify_method() -> TokenStream2 {
    quote! { fn verify() {} }
}
