#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod tasknet {

    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;
    use ink::prelude::string::String;
    use contracts::poll::PollRef as Poll;

    #[ink(storage)]
    pub struct TaskNet {
        polls: Mapping<i64, String>,
        next_poll_id: i64,
    }


    impl TaskNet {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                polls: Mapping::new(),
                next_poll_id: 1,
            }
        }

        #[ink(message)]
        pub fn create_poll(
            &mut self, title: String, description: String, reward: Balance
        ) {
            let poll = Poll {
                title,
                description,
                responses: Vec::new(),
                reward
            };

            self.polls.insert(&self.next_poll_id, &poll);
            self.next_poll_id += 1;
        }

        #[ink(message)]
        pub fn display_poll(&self, poll_id: i64) -> Option<String> {
            self.polls.get(&poll_id)
        }
    }
}
