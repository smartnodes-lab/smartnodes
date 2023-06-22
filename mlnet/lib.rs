#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod ainet {
    use ink::prelude::vec::Vec;

    #[derive(scale::Decode, scale::Encode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Layer {
        weights: Vec<u64>,
        biases: Vec<u64>,
    }

    #[derive(scale::Decode, scale::Encode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Model {
        layers: Vec<Layer>,
        activation: String
    }

    impl Model {
        pub fn forward(x) {}
    }

    #[ink(storage)]
    pub struct AINet {
        models: Mapping<i64, Model>,
    }

    impl Ml {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn flip(&mut self) {
            self.value = !self.value;
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its tasknet.
        #[ink::test]
        fn default_works() {
            let ml = Ml::default();
            assert_eq!(ml.get(), false);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut ml = Ml::new(false);
            assert_eq!(ml.get(), false);
            ml.flip();
            assert_eq!(ml.get(), true);
        }
    }
}
