#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod nexus {

    use src::Job;
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct Nexus {
        job_index: i64,
        index_to_proposal: Mapping<u32, Job>,
        // user_to_amount_funded: Mapping<AccountId, u32>,
        // min_stake: u8
    }

    impl Nexus {
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self {
                job_index: 0,
            }

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

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let nexus = Nexus::default();
            assert_eq!(nexus.get(), false);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut nexus = Nexus::new(false);
            assert_eq!(nexus.get(), false);
            nexus.flip();
            assert_eq!(nexus.get(), true);
        }
    }
}
