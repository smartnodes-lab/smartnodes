#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::trait_definition]
pub trait Task {
    #[ink(message)]
    fn respond(&mut self);

    #[ink(message)]
    fn dispute(&mut self);

    #[ink(message)]
    fn close(&mut self);

    #[ink(message)]
    fn open(&mut self);
}