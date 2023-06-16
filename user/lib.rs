#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod user {
    use ink::prelude::vec::Vec;

    #[ink(storage)]
    pub struct User {
        #[ink(topic)]
        address: AccountId,
        username: String,
        experience: Vec<String>,
        skills: Vec<String>,
        // history: Vec<String> // potentially a hash of a completed job
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
        pub fn remove_skill(&mut self, skill_ind: usize) {
            let caller: AccountId = Self::env().caller();

            if self.address == caller {
                for skill_ind in skill_inds {
                    self.skills.remove(skill_ind);
                }
            }
        }
    }
}
