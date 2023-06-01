#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod job {

    use ink::storage::Mapping;
    use ink::prelude::{string::String, vec::Vec};
    use ink::primitives::Hash;

    #[ink(storage)]
    pub struct Job {
        owner: AccoundId,
        // Title of job (keep it short)
        title: String,
        // Description of job
        description: String,
        reward: u32,
        reward_dist: bool,
        // contributions: Mapping<AccoundId, SentLoss>
        // n_responses: u32,
        // filters: str
    }

    // Event for miner's loss submission, keeps track of progress and contributions
    // #[ink(event)]
    // pub struct SentLoss {
    //     from: AccoundId
    // }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        // Returned if not enough balance to fulfill a request is available.
        InsufficientBalance,
        // Returned if not enough allowance to fulfill a request is available.
        InsufficientAllowance,
    }

    impl Job {
        #[ink(constructor)]
        pub fn new(
            _title: String, _description: String, _reward: u32, _reward_dist: bool
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

        // #[ink(message)]
        // pub fn send_loss(&mut self, loss_hash: Hash) {
        //     let from = self.env().caller();
        //
        //     if from != self.owner {
        //         ;
        //     }
        //
        //     // Send event of loss submission after submission of loss
        //     // self.env().emit_event(SentLoss {});
        // }

        // #[ink(message)]
        // pub fn
    }
}

pub use self::job::Job;