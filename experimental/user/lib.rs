#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod user {

    #[ink(storage)]
    pub struct User {
        address: AccountId,
        username: String,
        descriptors: String,
    }

    impl User {
        #[ink(constructor)]
        // Change descriptors to Vec of some kind and add ways to check if username/account is already taken
        pub fn new(username: String, descriptors: String) -> Self {
            let caller: AccountId = Self::env().caller();

            Self {
                address: caller,
                username,
                descriptors
            }
        }

        #[ink(message)]
        pub fn add_descriptor(&mut self, descriptor: String) {
            self.descriptors.push_str(descriptor);
        }
    }
}
