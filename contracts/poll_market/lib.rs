#![cfg_attr(not(feature = "std"), no_std)]

pub use self::poll_market::PollMarketRef;

#[ink::contract]
mod poll_market {

    use ink::primitives::AccountId;
    use ink::prelude::collections::HashMap;
    use ink::storage::Mapping;
    use ink::prelude::{string::String, vec::Vec};

    #[ink(storage)]
    pub struct PollMarket {
        polls: Mapping<i64, Poll>,
        next_poll_id: i64,
    }

    pub struct Poll {
        title: String,
        description: String,
        responses: Vec<String>,
        reward: Balance,
        has_voted: HashMap<AccountId, ()>
    }

    impl PollMarket {
        #[ink(contract)]
        pub fn new() -> Self{
            Self {
                polls: Mapping::new(),
                next_poll_id: 1,
            }
        }

        #[ink(message)]
        pub fn create_poll(
            &mut self, title: String, description: String, responses: Vec<String>, reward: Balance
        ) {
            let poll = Poll {
                title,
                description,
                responses,
                reward,
                has_voted: HashMap::new()
            };

            self.polls.insert(&self.next_poll_id, &poll);
            self.next_poll_id += 1;
        }

        #[ink(message)]
        pub fn display_poll(&self, poll_id: i64) -> Option<Poll> {
            self.polls.get(&poll_id)
        }
    }
}
