#![cfg_attr(not(feature = "std"), no_std, no_main)]

// pub use self::users::Users;

#[ink::contract]
mod users {
    use super::*;
    use ink::prelude::{string::String, vec::Vec};

    #[derive(scale::Decode, scale::Encode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct User {
        address: AccountId,
        username: String,
        experience: Vec<String>,
        skills: Vec<String>,
        // history: Vec<String> // potentially a hash of a completed tasknet
    }

    impl User {
        pub fn new(
            address: AccountId,
            username: String,
            experience: Vec<String>,
            skills: Vec<String>
        ) -> Self {
            Self {
                address,
                username,
                experience,
                skills
            }
        }
    }


        #[ink(message)]
        pub fn create_user(
            &mut self,
            username: String,
            experience: Vec<String>,
            skills: Vec<String>
        ) {
            let caller: AccountId = Self::env().caller();

            // Create users if address isn't linked to an account
            if !self.users.iter().any(|user| user.username == username) {
                let mut user = User::new(
                    caller,
                    username,
                    Vec::<String>::with_capacity(10),
                    Vec::<String>::with_capacity(10),
                );

                // Push experiences to users if specified
                for i in 0..experience.len().min(user.experience.capacity()) {
                    user.experience.push(experience.get(i)
                                    .cloned()
                                    .unwrap_or_else(|| String::new()));
                }

                // Push skills to users if specified
                for i in 0..skills.len().min(user.skills.capacity()) {
                    user.skills.push(skills.get(i)
                                    .cloned()
                                    .unwrap_or_else(|| String::new()));
                }

                // Add users to contract
                self.users.push(user);
            } else {
                // return Err(FrameworkError::UserAlreadyExists);
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn create_user() {
            // Arrange
            let mut contract = Users::new();
            let caller: AccountId = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>().alice;
            let username = String::from("Alice");
            let experience = vec![String::from("Software Developer"), String::from("Data Analyst")];
            let skills = vec![
                String::from("Rust"),
                String::from("Python"),
                String::from("SQL")];

            contract.create_user(username.clone(), experience.clone(), skills.clone());
            // let users = contract.get_user(caller);
            //
            // assert_eq!(users, Some(User { username, experience, skills }));
        }
    }
}
