#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod framework {
    use ink::storage::Mapping;
    use ink::prelude::{
        string::String,
        vec::Vec,
        collections::HashMap,
        boxed::Box
    };
    use ml_net::MLNetRef;
    use job::Job;

    #[derive(Debug, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum FrameworkError {
        UserTaken(String),
        UserAlreadyResponded,
        TaskRewardTooLow
    }

    #[derive(scale::Decode, scale::Encode, Debug, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct User {
        address: AccountId,
        username: String,
        skills: Vec<String>,
        history: Vec<String>, // potentially a hash of a completed tasknet-net
        locked_balance: Balance,
    }

    impl User {
        pub fn new(
            address: AccountId,
            username: String,
            skills: Vec<String>,
        ) -> Self {
            Self {
                address,
                username,
                skills,
                history: Vec::new(),
                locked_balance: 0
            }
        }
    }

    #[ink(storage)]
    pub struct Framework {
        users: Vec<User>,
        next_user_id: i64,
        ml_net: Mapping<User, MLNetRef>
    }

    impl Framework {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                users: Vec::new(),
                next_user_id: 0,
                ml_net: Mapping::new()
            }
        }

        // Payment locking functionality likely broken
        #[ink(message, payable)]
        pub fn create_user(
            &mut self, username: String, skills: Vec<String>
        ) -> Result<(), FrameworkError> {
            let caller: AccountId = Self::env().caller();

            // Create user if address isn't linked to an account
            if !self.users.iter().any(|user| user.username == username) {
                let mut user = User::new(
                    caller,
                    username,
                    Vec::<String>::with_capacity(10),
                );

                // Push skills to user if specified
                for i in 0..skills.len().min(user.skills.capacity()) {
                    user.skills.push(skills.get(i)
                        .cloned()
                        .unwrap_or_else(|| String::new())
                    );
                }

                // Add value of attached balance
                self.env().transfer(caller, user.locked_balance);

                // Add user to contract
                self.users.push(user);

                Ok(())
            } else {
                Err(FrameworkError::UserTaken(username))
            }
        }

        #[ink(message)]
        pub fn create_job(
            &mut self, reward: Balance, distribution: bool, filters: Vec<String>, formatting: String
        ) {
            unimplemented!()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{test, DefaultEnvironment};

        #[ink::test]
        pub fn framework_test() {
            let accounts = test::default_accounts::<DefaultEnvironment>();
            let mut main_contract: Framework = Framework::new();

            // Test user creation
            main_contract.create_user(
                String::from("jumbomeats"),
                Vec::new()
            );
        }
    }
}
