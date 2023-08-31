#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod framework {
    use ink::{
        prelude::{
            string::String,
            vec::Vec
        },
        storage::Mapping,
    };
    // use network::Network;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo)
    )]
    pub enum FrameworkError {
        TaskOpen,
        FundingTooLow,
        UserAlreadyExists
    }

    #[derive(scale::Decode, scale::Encode, Debug, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(
            scale_info::TypeInfo,
            ink::storage::traits::StorageLayout
        )
    )]
    pub struct User {
        uid: i64,
        public_key: String,
        locked: Balance,
        reputation: i8,
    }

    #[derive(scale::Decode, scale::Encode, Debug, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(
            scale_info::TypeInfo,
            ink::storage::traits::StorageLayout
        )
    )]
    pub struct Connection {
        author: i64,
        funding: Balance,
        description: String,
        uids: Vec<i64>,
    }

    #[ink(storage)]
    pub struct Framework {
        id_counter: i64,
        connection_counter: i64,
        users: Mapping<AccountId, User>,
        // connections: Mapping<i64, Connection>,
        total_deposits: Balance,
    }

    impl Framework {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                id_counter: 0,
                connection_counter: 0,
                users: Mapping::new(),
                // connections: Mapping::default(),
                total_deposits: 0
            }
        }

        #[ink(message)]
        pub fn create_user(
            &mut self,
            public_key: String
        ) -> Result<(), FrameworkError> {
            let caller: AccountId = self.env().caller();

            if self.users.contains(caller) {
                return Err(FrameworkError::UserAlreadyExists);
            }

            let user: User = User {
                uid: self.id_counter,
                public_key,
                locked: 0,
                reputation: 1
            };

            self.users.insert(caller, &user);
            self.id_counter += 1;

            Ok(())
        }

        #[ink()]

        #[ink(message)]
        pub fn get_uid(&self, user_address: AccountId) -> i64 {
            let user = self.users.get(user_address).expect("User not found!");

            return user.uid;
        }

        #[ink(message, payable)]
        pub fn create_network(&mut self) {

        }
    }

    // impl Network for Framework {
    //     #[ink(message)]
    //     fn join(&mut self) {
    //         let caller: AccountId = Self::env().caller();
    //
    //         // Check that user hasn't joined already
    //         if self.user_cache.iter().all(|user| user.id != caller) {
    //             self.user_cache.push(UserCache::new(caller));
    //         }
    //     }
    //
    //     #[ink(message)]
    //     fn open(&mut self) {
    //         self.open = true;
    //         todo!("add reward mechanism")
    //     }
    //
    //     #[ink(message)]
    //     fn close(&mut self) {
    //         self.open = false;
    //         todo!("assert reward + dispute mechanisms")
    //     }
    //
    //     #[ink(message)]
    //     fn dispute(&mut self) {
    //         unimplemented!()
    //     }
    //
    //     #[ink(message)]
    //     fn get_proof(&self) {
    //         unimplemented!()
    //     }
    // }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{test, DefaultEnvironment};

        #[ink::test]
        fn test_contract() {
            let accounts = test::default_accounts::<DefaultEnvironment>();

            set_caller(accounts.alice);
            let mut framework: Framework = Framework::new();

            set_balance(accounts.alice, 1_000);
            set_caller(accounts.alice);
            let result = framework.create_user(
                String::from("abcdefghijklmnopqrstuvwxyz0123456789-=+_)(*&^%$#@!~[];',./{}|:<>?")
            );
        }

        fn set_caller(caller: AccountId) {
            test::set_caller::<DefaultEnvironment>(caller);
        }

        fn set_balance(caller: AccountId, new_balance: Balance) {
            test::set_account_balance::<DefaultEnvironment>(caller, new_balance);
        }
    }
}
