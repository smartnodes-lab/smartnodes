#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod framework {
    use ink::storage::Mapping;

    #[derive(scale::Decode, scale::Encode, Debug, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct User {
        address: AccountId,
        public_key: String,
        locked_balance: Balance,
        uid: i64,
        reputation: f32,
        block_joined: u64
    }

    impl User {
        fn new(
            address: AccountId,
            public_key: String,
            locked: Balance,
            uid: i64,
            block_joined: u64
        ) -> Self {
            Self {
                address,
                public_key,
                locked_balance: locked,
                uid,
                reputation: 1.0,
                block_joined: u64
            }
        }
    }

    #[derive(Debug, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum FrameworkError {
        UserTaken,
        PublicKeyInvalid,
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

        #[ink(message, payable)]
        pub fn create_user(&mut self, pub_key: String) -> Result<(), FrameworkError> {
            let caller: AccountId = Self::env().caller();

            if !self.users.iter().any(
                |user| user.public_key == pub_key || user.address == caller
            ) {
                let creation_block = Self::env().block_timestamp();

                let mut user = User::new(
                    caller,
                    username,
                    Vec::<String>::with_capacity(10),
                    self.next_user_id,
                    creation_block
                );

                self.env().transfer(caller, user.locked_balance);

                self.next_user_id += 1;
                self.users.push(user);

                Ok(())
            } else {
                Err(FrameworkError::UserTaken)
            }
        }
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
            let framework = Framework::default();

            set_caller(accounts.alice);
            framework.create_user(String::from("jumboslice"))
        }

        fn set_caller(caller: AccountId) {
            test::set_caller::<DefaultEnvironment>(caller);
        }
    }
}
