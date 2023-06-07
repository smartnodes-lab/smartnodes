#![cfg_attr(not(feature = "std"), no_std)]

pub use self::poll_market::PollMarket;

#[ink::contract]
mod poll_market {

    use ink::storage::Mapping;
    use ink::prelude::{string::String, vec::Vec};

    pub struct Poll {
        creator: AccountId,
        title: String,
        description: String,
        reward: Balance,
        responses: Mapping<AccountId, String>
    }

    #[ink(storage)]
    pub struct PollMarket {
        polls: Vec<Poll>,
        next_poll_id: usize,
    }

    impl PollMarket {
        #[ink(constructor)]
        pub fn new() -> Self{
            Self {
                polls: Vec::new(),
                next_poll_id: 0,
            }
        }

        #[ink(message)]
        pub fn create_poll(
            &mut self, title: String, description: String, reward: Balance
        ) {
            let caller: AccountId = Self::env().caller();
            let poll = Poll {
                creator: caller,
                title,
                description,
                reward,
                responses: Mapping::new()
            };

            self.polls.insert(self.next_poll_id, poll);
            self.next_poll_id += 1;
        }

        #[ink(message)]
        pub fn display_poll(&self, poll_id: i64) -> Option<Poll> {
            self.polls.get(poll_id)
        }
    }
}
