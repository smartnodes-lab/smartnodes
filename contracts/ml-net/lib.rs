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
    pub struct LayerVec {
        layer_vec: Vec<Layer>
    }

    impl LayerVec {
        pub fn new() -> Self {
            Self {
                layer_vec: Vec::new()
            }
        }
    }

    #[ink(storage)]
    pub struct MLNet {
        author: AccountId,
        reward: Balance,
        kind: i8,
        reward_distribution: bool,
        open: bool,
        network_map: Mapping<AccountId, LayerVec>,
        layer_dims: Vec<i64>,
        // max_responses: i8, // interchangable with max_block_len?
        // formatting_tips: String, // can be used to justify disputes
    }

    impl MLNet {
        #[ink(constructor)]
        pub fn new(
            reward: Balance,
            reward_distribution: bool,
            _kind: i8,
            layer_dims: Vec<i64>,
            // max_responses: i8,
            // formatting_tips: String
        ) -> Self {
            let specified_layers: usize = layer_dims.capacity();
            let caller: AccountId = Self::env().caller();

            // Enforce network structure depending on kind: 0: bloom, 1: cascade, 2: ensemble
            if _kind == 0 || _kind == 2 {
                if !specified_layers == 2 {
                    ink::env::debug_println!("Invalid network format for given kind!");
                    Self::env().terminate_contract(caller);
                }
            } else if _kind == 1 {
                if specified_layers <= 2 {
                    ink::env::debug_println!("Invalid network format for given kind!");
                    Self::env().terminate_contract(caller);
                }
            } else {
                ink::env::debug_println!("Invalid network kind!");
                Self::env().terminate_contract(caller);
            }

            Self {
                author: Self::env().caller(),
                reward,
                reward_distribution,
                kind: _kind,
                open: true,
                network_map: Mapping::new(),
                layer_dims
                // participation: Mapping::new(),
                // filters,
                // max_responses,
                // formatting_tips,
            }
        }

        #[ink(message)]
        pub fn submit_y(&mut self, y_ind: i64, y_pred: Layer) {
            let caller: AccountId = Self::env().caller();

            if self.network_map.contains(&caller) {
                if let Some(mut layer_vec) = self.network_map.get(&caller) {
                    layer_vec.layer_vec.push(y_pred);
                }
            } else {
                ink::env::debug_println!("User not found in network!");
                Self::env().terminate_contract(caller);
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
                0,
                Vec::new()
            );

            let layer = Layer::DimOne(Vec::with_capacity(10));
            net.submit_y(10, layer);
        }
    }
}
