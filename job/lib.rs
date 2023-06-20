#![cfg_attr(not(feature = "std"), no_std)]

pub use self::job::Job;

#[ink::contract]
mod job {

    use ink::storage::Mapping;
    use ink::prelude::string::String;

    #[ink(storage)]
    pub struct Job {
        author: AccountId,
        title: String,
        description: String,
        reward: Balance,
        reward_distribution: bool,
        responses: Mapping<AccountId, String>,
        filters: Option<Mapping<i8, String>>,
        max_responses: Option<i32>, // interchangable with max_block_len?
        required_format: Option<String>, // can be used to justify disputes
        open: bool
    }

    impl Job {
        #[ink(constructor)]
        pub fn new(
            _title: String,
            _description: String,
            _reward: u32,
            _reward_dist: bool,
            _filters: Option<Vec<String>>,
            _max_responses: Option<i16>,
            _required_format: Option<String>,
        ) -> Self {
            let caller = Self::env().caller();

            let mut job = Self {
                author: caller,
                title: _title,
                description: _description,
                reward: _reward,
                reward_distribution: _reward_dist,
                responses: Mapping::default(),
                filters: Vec::new(),
                max_responses: -1,
                required_format: String::from(""),
                open: true
            };

            if let Some(_filters) = _filters {
                for filter in _filters {
                    job.filters.push(filter);
                }
            }

            if let Some(_max_responses) = _max_responses {
                if _max_responses > 0 {
                    job.max_responses = _max_responses;
                }
            }

            if let Some(_required_format) = _required_format {
                job.required_format = _required_format;
            }
        }
    }
}