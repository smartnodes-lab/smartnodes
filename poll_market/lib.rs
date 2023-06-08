#![cfg_attr(not(feature = "std"), no_std)]

pub use self::poll_market::PollMarket;

#[ink::contract]
mod poll_market {

    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;
    use ink::prelude::string::String;

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Poll {
        creator: AccountId,
        title: String,
        description: String,
        reward: Balance,
        responses: Vec<String>,
        participants: Vec<AccountId>
    }

    #[ink(storage)]
    pub struct PollMarket {
        polls: Mapping<i64, Poll>,
        next_poll_id: i64,
    }

    impl PollMarket {
        #[ink(constructor)]
        pub fn new() -> Self{
            Self {
                polls: Mapping::new(),
                next_poll_id: 0,
            }
        }

        #[ink(message)]
        pub fn create_poll(
            &mut self, title: String, description: String, reward: Balance, n_responses: i16
        ) {
            let creator: AccountId = Self::env().caller();
            let poll = Poll {
                creator,
                title,
                description,
                reward,
                responses: Vec::with_capacity(n_responses),
                participants: Vec::with_capacity(n_responses)
            };

            self.polls.insert(self.next_poll_id, &poll);
            self.next_poll_id += 1;
        }

        #[ink(message)]
        pub fn display_poll(&self, poll_id: i64) -> Option<Poll> {
            self.polls.get(poll_id)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        pub fn poll_market_works() {
            let mut market: PollMarket = PollMarket::new();

            market.create_poll(
                String::from("Flagged Comment: I like apples."),
                String::from("Is this post harmful? (y/n)"),
                1,
                10
            );
            market.create_poll(
                String::from("Flagged Comment: Your mom is a whore"),
                String::from("Is this post harmful? (y/n)"),
                1,
                10
            );

            let poll = market.polls.get(1).unwrap();

            println!("Poll Title: {}", poll.title);
            println!("Description: {}", poll.description);
            println!("Reward: {} AZERO", poll.reward);
        }
    }
}
