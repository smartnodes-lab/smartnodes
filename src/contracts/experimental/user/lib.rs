#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use self::user::User;

#[ink::contract]
mod user {
    use super::*;
    use ink::prelude::{string::String, vec::Vec};

    #[ink(storage)]
    pub struct User {
        address: AccountId,
        username: String,
        experience: Vec<String>,
        skills: Vec<String>,
        // history: Vec<String> // potentially a hash of a completed tasknet-net
    }

    impl User {
        #[ink(constructor)]
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
            // let user = contract.get_user(caller);
            //
            // assert_eq!(user, Some(User { username, experience, skills }));
        }
    }
}
