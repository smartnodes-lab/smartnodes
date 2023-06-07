#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod tasknet {

    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;
    use ink::prelude::string::String;
    use poll_market::PollMarket;

    #[ink(storage)]
    pub struct TaskNet {
        poll_market: PollMarket,
    }


    impl TaskNet {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                poll_market: PollMarket::new(),
            }
        }

        #[ink(message)]
        pub fn create_poll(
            &mut self, title: String, description: String, reward: Balance
        ) {
            self.poll_market.create_poll(title, description, reward);
        }

        // #[ink(message)]
        // pub fn display_poll(&self, poll_id: int) -> Option<Poll> {
        //
        // }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        pub fn create_poll_works() {
            let mut net = TaskNet::new();

            net.create_poll(String::from("First Poll"),
                            String::from("What is your favourite pokemon card?"),
                            1);
        }
    }
}
