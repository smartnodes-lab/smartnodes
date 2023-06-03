#![cfg_attr(not(feature = "std"), no_std)]

pub use self::poll::PollRef;

#[ink::contract]
mod poll {

    use ink::prelude::string::String;
    use ink::storage::Mapping;

    #[ink(storage)]
    #[derive(scale::Encode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub struct Poll {
        title: String,
        description: String,
        responses: Mapping<AccountId, String>,
        reward: Balance
    }

    impl scale::EncodeLike<String> for Poll {}

    impl Poll {
        #[ink(constructor)]
        pub fn new(
            title: String, description: String, reward: Balance
        ) -> Self {
            let caller = Self::env().caller();

            Self {
                title,
                description,
                responses: Mapping::new(),
                reward,
            }
        }

        #[ink(message)]
        pub fn answer(&mut self, answer: String) {
            let caller = self.env().caller();

            // assert_eq!(self.answers.get(caller), );

            self.responses.insert(&answer);

            // Send event of loss submission after submission of loss
            // self.env().emit_event(SentLoss {});
        }
    }
}