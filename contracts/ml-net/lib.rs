#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use ml_net::MLNetRef;

#[ink::contract]
mod ml_net {
    use ink::storage::Mapping;
    use ink::prelude::{
        // string::String,
        vec::Vec
    };
    use task::Task;

    /// Allows for multi-dimensional layers
    #[derive(scale::Decode, scale::Encode, Debug, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub enum Layer {
        /// One dimensional layer
        DimOne(Vec<i64>),
        /// Two dimensional layer
        DimTwo(Vec<Vec<i64>>),
        /// Three dimensional layer
        DimThree(Vec<Vec<Vec<i64>>>),
    }

    /// To store y_pred sumissions and any other data relevant for proving and improving training
    #[derive(scale::Decode, scale::Encode, Debug, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct UserCache {
        /// Y submission vector
        y_cache: Vec<Layer>,
        /// Associated data index
        y_loc: Vec<i64>,
        /// Submission indexes
        y_ind: i64,
        // User role in ml-net (0: sensory, 1: inter,
        // role: i8
    }

    impl UserCache {
        pub fn new() -> Self {
            Self {
                y_cache: Vec::new(),
                y_loc: Vec::new(),
                y_ind: 0,
                // role
            }
        }
    }

    /// Framework for an ML training and execution platform
    #[ink(storage)]
    pub struct MLNet {
        author: AccountId,
        reward: Balance,
        /// Kind of ML task initiated (0: bloom, 1: cascade, 2: ensemble, 3: execute)
        kind: i8,
        /// Distribution of reward is singular (false) or uniform (true)
        reward_distribution: bool,
        open: bool,
        /// Keeps track of user contributions
        user_map: Mapping<AccountId, UserCache>,
        /// Specify neural network layer architeceture
        layer_dims: Vec<i64>,
        // max_responses: i8, // interchangable with max_block_len?
        // formatting_tips: String, // can be used to justify disputes
    }

    impl MLNet {
        #[ink(constructor)]
        pub fn new(
            author: AccountId,
            reward: Balance,
            reward_distribution: bool,
            kind: i8,
            layer_dims: Vec<i64>,
            // participation: Mapping::new(),
            // filters,
            // max_responses,
            // formatting_tips,
            // data: String (some way of direting the user to the data (via arweave or url)
        ) -> Self {
            let specified_layers: usize = layer_dims.capacity();

            // Enforce network structure depending on kind: 0: bloom, 1: cascade, 2: ensemble,
            // 3: exec
            if kind == 0 || kind == 2 {
                if !specified_layers == 2 {
                    ink::env::debug_println!("Invalid network format for given kind!");
                    Self::env().terminate_contract(author);
                }
            } else if kind == 1 {
                if specified_layers <= 2 {
                    ink::env::debug_println!("Invalid network format for given kind!");
                    Self::env().terminate_contract(author);
                }
            } else if !kind == 3 {
                if specified_layers < 2 {
                    ink::env::debug_println!("Invalid network format, must be greater than 1");
                    Self::env().terminate_contract(author);
                }
            }
            else {
                ink::env::debug_println!("Invalid network kind!");
                Self::env().terminate_contract(author);
            }

            Self {
                author,
                reward,
                reward_distribution,
                kind,
                open: true,
                user_map: Mapping::new(),
                layer_dims
            }
        }

        #[ink(message)]
        pub fn join_net(&mut self, user: AccountId) {
            // Check that user hasn't joined already
            if !self.user_map.contains(user) {
                let user_cache: UserCache = UserCache::new();
                self.user_map.insert(&user, &user_cache);
            } else {
                ink::env::debug_println!("User already joined network!");
                Self::env().terminate_contract(user);
            }
        }

        #[ink(message)]
        pub fn submit_y(&mut self, y_ind: i64, y_pred: Layer) {
            let caller: AccountId = Self::env().caller();

            if self.user_map.contains(&caller) {
                if let Some(mut user_cache) = self.user_map.get(&caller) {
                    user_cache.y_cache.push(y_pred);
                    user_cache.y_loc.push(y_ind);
                    user_cache.y_ind += 1;
                }
            } else {
                ink::env::debug_println!("User not found!");
                Self::env().terminate_contract(caller);
            }
        }

        #[ink(message)]
        pub fn submit_loss(&mut self, loss: Layer) {
            let caller: AccountId = Self::env().caller();
            //
            // if self.user_map.contains(&caller) {
            //     if let Some(mut user_cache) = self.user_map.get(&caller) {
            //         user_cache.y_cache.push(y_pred);
            //         user_cache.y_loc.push(y_ind);
            //         user_cache.y_ind += 1;
            //     } else {
            //         ink::env::debug_println!("Invalid User call!");
            //         Self::env().terminate_contract(caller);
            //     }
            // } else {
            //     ink::env::debug_println!("User not found!");
            //     Self::env().terminate_contract(caller);
            // }
        }

        // fn calculate_loss(&mut self, y_pred: Layer) {
        //     unimplemented!()
        // }
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
        use std::any::Any;
        use super::*;

        #[ink::test]
        pub fn ml_works() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();


            // Main user defines ml-net
            let mut net: MLNet = MLNet::new(
                accounts.alice,
                0,
                true,
                0,
                vec![10, 64, 64, 8, 2]
            );

            net.join_net(accounts.alice);

            let layer = Layer::DimTwo(
                vec![
                    vec![1000, 10000124, 123112973],
                    vec![0, 9120934, 14238974, 4382],
                    vec![1, 1203, 1, 2],
                    vec![200, 20, 12039, 9000],
                ]
            );

            net.submit_y(10, layer);

            if let Some(cache) = net.user_map.get(accounts.alice) {
                if let Some(y) = cache.y_cache.get(0) {
                    match y {
                        Layer::DimOne(values) => {
                            for value in values {
                                print!("{}", value)
                            }
                        }
                        Layer::DimTwo(values) => {
                            for x in values {
                                for y in x {
                                    print!("{}", y)
                                }
                                println!("l")
                            }
                        }
                        Layer::DimThree(values) => {}
                    }
                }
            }
        }
    }
}
