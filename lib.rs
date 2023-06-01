#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod nexus {

    // use src::Job;
    // use ink::storage::Mapping;

    #[ink(storage)]
    pub struct Nexus {
        job_index: i64,
        // index_to_proposal: Mapping<u32, Job>,
        // user_to_amount_funded: Mapping<AccountId, u32>,
        // min_stake: u8
    }

    impl Nexus {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                job_index: 0,
            }
        }

        #[ink(message)]
        pub fn add_job(&mut self) {
            self.job_index += 1;
        }

        #[ink(message)]
        pub fn get_job_num(self) -> i64 {
            self.job_index
        }
    }
}
