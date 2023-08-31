#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::trait_definition]
pub trait Job {
    #[ink(message)]
    fn dispute(&mut self);

    #[ink(message)]
    fn join(&mut self);

    #[ink(message)]
    fn close(&mut self);

    #[ink(message)]
    fn open(&mut self);

    #[ink(message)]
    fn get_proof(&self);
}