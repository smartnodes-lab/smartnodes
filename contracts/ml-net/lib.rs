#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ml_net::MLNetRef;

#[ink::contract]
mod ml_net {
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
    pub struct YVec {
        y_vec: Vec<Layer>
    }

    impl YVec {
        pub fn new() -> Self {
            Self {
                y_vec: Vec::new()
            }
        }
    }

    #[derive(Debug, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum MLNetError {
        UserTaken,
        UserAlreadyResponded,
        TaskRewardTooLow
    }

    #[ink(storage)]
    pub struct MLNet {
        author: AccountId,
        reward: Balance,
        kind: i8,
        reward_distribution: bool,
        open: bool,
        network_map: Mapping<AccountId, YVec>,
        output_dim: i8,
        // max_responses: i8, // interchangable with max_block_len?
        // formatting_tips: String, // can be used to justify disputes
    }

    impl MLNet {
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
                network_map: Mapping::new()
                // participation: Mapping::new(),
                // filters,
                // max_responses,
                // formatting_tips,
            }
        }

        #[ink(message)]
        pub fn submit_y(&mut self, y_pred: Layer) {
            let caller: AccountId = Self::env().caller();

            if self.network_map.contains(&caller) {
                if let Some(mut y_vec) = self.network_map.get(&caller) {
                    y_vec.y_vec.push(y_pred);
                }
            }


        }

        fn calculate_loss(&mut self, y_pred: Layer) {
            unimplemented!()
        }
    }

    impl Task for MLNet {
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
            let mut net: MLNet = MLNet::new(
                0,
                true,
                0
            );

            let layer = Layer::DimOne(Vec::with_capacity(10));
            net.submit_y(layer);
        }
    }

}
