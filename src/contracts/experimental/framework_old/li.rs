#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod framework {
    use ink::env::hash::Blake2x128;
    use ink::prelude::{
        string::String,
        vec::Vec
    };

    #[derive(scale::Decode, scale::Encode, Debug, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct User {
        address: AccountId,
        // public_key: String,
        // locked_balance: Balance,
        // uid: i64,
        // reputation: i8,
        // block_joined: u64
    }

    impl User {
        fn new(
            address: AccountId,
            // public_key: String,
            // locked: Balance,
            // uid: i64,
            // block_joined: u64
        ) -> Self {
            Self {
                address,
                // public_key,
                // locked_balance: locked,
                // uid,
                // reputation: 127,
                // block_joined
            }
        }
    }

    #[derive(Debug, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum FrameworkError {
        UserTaken,
        PublicKeyInvalid,
        LockFunds
    }

    #[ink(storage)]
    pub struct Framework {
        users: Vec<User>,
        next_user_id: i64,
    }

    impl Framework {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                users: Vec::new(),
                next_user_id: 0,
            }
        }

        // #[ink(message, payable)]
        // pub fn create_user(&mut self, pub_key: String, locked_amount: Balance) -> Result<(), FrameworkError> {
        //     let caller: AccountId = Self::env().caller();
        //
        //     if !self.users.iter().any(
        //         |user| user.address == caller // || user.public_key == pub_key
        //     ) {
        //         let creation_block = Self::env().block_timestamp();
        //
        //         match self.env().transfer(caller, locked_amount) {
        //             Ok(_) => {
        //                 let mut user = User::new(
        //                     caller,
        //                     // pub_key,
        //                     // locked_amount,
        //                     // self.next_user_id,
        //                     // creation_block
        //                 );
        //
        //                 self.next_user_id += 1;
        //                 self.users.push(user);
        //
        //                 Ok(())
        //             },
        //             Err(_) => Err(FrameworkError::LockFunds)
        //         }
        //     } else {
        //         Err(FrameworkError::UserTaken)
        //     }
        // }
        //
        // pub fn check_key(pub_key: String) -> bool {
        //
        // }
    }

    #[cfg(test)]
    mod tests {
         use super::*;
        use ink::env::{test, DefaultEnvironment};

        #[ink::test]
        fn contract_works() {
            let accounts = test::default_accounts::<DefaultEnvironment>();
            let addresses: Vec<AccountId> = vec![
                accounts.alice, accounts.bob, accounts.charlie,
                accounts.eve, accounts.django, accounts.frank
            ];
            let mut framework = Framework::new();

            set_caller(accounts.alice);
            framework.create_user(String::from("jumboslice"), 1)
        }

        fn set_caller(caller: AccountId) {
            test::set_caller::<DefaultEnvironment>(caller);
        }
    }
}
