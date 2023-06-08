#![cfg_attr(not(feature = "std"), no_std, no_std)]

#[ink::contract]
mod tasknet {
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;
    use ink::prelude::string::String;
    use poll_market::PollMarket;

    #[ink(storage)]
    pub struct TaskNet {
        polls: PollMarket,
        next_poll_id: i64,
    }


    impl TaskNet {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                polls: PollMarket::new(),
                next_poll_id: 0
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        pub fn init() {
            let mut network = TaskNet::new();
        }
    }
}
