#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod job {
    #[ink(storage)]
    pub struct Job {
        title: String,
        description: String,
        reward: u32,
        reward_distribution: bool, // false: final distribution to best algo, true: equal + gradual
                                   //   distributions for lower loss
        n_responses: u32,
        filters: String
    }

    impl Job {
        #[ink(constructor)]
        pub fn new() -> Self { Self {} }
    }
}