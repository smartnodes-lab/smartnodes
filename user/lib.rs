#![cfg_attr(not(feature = "std"), no_std)]

pub use self::user::User;

#[ink::contract]
mod user {
    use ink::prelude::vec::Vec;

    #[ink(storage)]
    pub struct User {
        address: AccountId,
        username: String,
        experience: Vec<String>,
        skills: Vec<String>,
        // history: Vec<String> // potentially a hash of a completed task
    }

    impl User {
        #[ink(constructor)]
        // Change descriptors to Vec of some kind and
        // add ways to check if username/account is already taken
        pub fn new(
            username: String,
            experience: Option<Vec<String>>,
            skills: Option<Vec<String>>,
        ) -> Self {
            let caller: AccountId = Self::env().caller();

            let mut user = Self {
                address: caller,
                username,
                experience: Vec::new(),
                skills: Vec::new()
            };

            // Store experience if specified
            if let Some(experience) = experience {
                for description in experience {
                    user.experience.push(description);
                }
            };

            // Store skills if specified
            if let Some(skills) = skills {
                for skill in skills {
                    user.skills.push(skill);
                }
            }

            return user;
        }

        #[ink(message)]
        pub fn add_skills(&mut self, skills: Vec<String>) {
            let caller: AccountId = Self::env().caller();

            if self.address == caller {
                for skill in skills {
                    if !self.skills.contains(&skill) {
                        self.skills.push(skill);
                    }
                }
            }
        }

        #[ink(message)]
        pub fn remove_skills(&mut self, skills: Vec<String>) {
            let caller: AccountId = Self::env().caller();

            if self.address == caller {
                for skill in skills {
                    self.skills.retain(|item| item.eq(&skill));
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        pub fn user_works() {
            let mut _user: User = User::new(
                String::from("jumbomeats"),
                Some(vec![
                    String::from("created minecraft server with plugins")
                ]),
                Some(vec![
                    String::from("servers")
                ])
            );

            println!("{}", {_user.skills.get(0).unwrap()});

            _user.add_skills(
                vec![
                    String::from("Rust"),
                    String::from("ink!")
                ]
            );

            for (ind, skill) in _user.skills.iter().enumerate() {
                println!("{}", {_user.skills.get(ind).unwrap()});
                println!("{}", { ind })
            }

            _user.remove_skills(
                vec![
                    String::from("Rust")
                ]
            );

            for (ind, skill) in _user.skills.iter().enumerate() {
                println!("{}", {_user.skills.get(ind).unwrap()});
                println!("{}", { ind })
            }

        }
    }
}
