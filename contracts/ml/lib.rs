#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod ml_task {
    use ink::storage::Mapping;
    use ink::prelude::{
        // string::String,
        vec::Vec
    };
    use task::Task;

    /// Allows vari-dimensional layers
    #[derive(scale::Decode, scale::Encode, Debug, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub enum Layer {
        DimOne(Vec<i64>),
        DimTwo(Vec<Vec<i64>>)
    }

    #[derive(scale::Decode, scale::Encode, Debug, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct YMap {
        y_map: Vec<Layer>
    }

    impl YMap {
        pub fn new() -> Self {
            Self {
                y_map: Vec::new()
            }
        }
    }

    #[ink(storage)]
    pub struct MLTask {
        author: AccountId,
        reward: Balance,
        kind: i8,
        reward_distribution: bool,
        open: bool,
        y_map: Mapping<AccountId, YMap>
        // max_responses: i8, // interchangable with max_block_len?
        // formatting_tips: String, // can be used to justify disputes
    }

    impl MLTask {
        #[ink(constructor)]
        pub fn new(
            reward: Balance,
            reward_distribution: bool,
            kind: i8,
            // filters: Vec<String>,
            // max_responses: i8,
            // formatting_tips: String
        ) -> Self {
            Self {
                author: Self::env().caller(),
                reward,
                reward_distribution,
                kind,
                open: true,
                y_map: Mapping::new()
                // participation: Mapping::new(),
                // filters,
                // max_responses,
                // formatting_tips,
            }
        }

        #[ink(message)]
        pub fn submit_y(&mut self, y_pred: Layer) {
            unimplemented!()
        }

        fn calculate_loss(&mut self, y_pred: Layer) {
            unimplemented!()
        }
    }

    impl Task for MLTask {
        #[ink(message)]
        fn respond(&mut self) {
            unimplemented!()
        }

        #[ink(message)]
        fn dispute(&mut self) {
            unimplemented!()
        }

        #[ink(message)]
        fn close(&mut self) {
            unimplemented!()
        }

        #[ink(message)]
        fn open(&mut self) {
            unimplemented!()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        pub fn ml_works() {
            let mut task: MLTask = MLTask::new(
                0,
                true,
                0
            );
        }
    }

}
