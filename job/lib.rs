#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod job {

    use ink::prelude::string::String;
    use ink::primitives::Hash;
    // use ink::prelude::vec::Vec;

    #[ink(storage)]
    pub struct Job {
        owner: Address,
        title: String,
        description: String,
        reward: u32,
        reward_dist: bool,
        // n_responses: u32,
        // filters: str
    }

    #[ink(event)]
    pub struct SentLoss {

    }

    impl Job {
        #[ink(constructor)]
        pub fn new(
            _title: String,
            _description: String,
            _reward: u32,
            _reward_dist: bool,
        ) -> Self {
            let caller = Self::env().caller();

            Self {
                owner: caller,
                title: _title,
                description: _description,
                reward: _reward,
                reward_dist: _reward_dist,
            }
        }

        #[ink(message)]
        pub fn send_loss(&mut self, loss: Hash) {
            let from = self.env().caller();


            self.env().emit_event(SentLoss {
                from
            });
        }

        pub fn
    }
}