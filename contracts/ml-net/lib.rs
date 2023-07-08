#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use ml_net::{MLNetError, MLNetRef};

#[ink::contract]
mod ml_net {
    use ink::storage::Mapping;
    use ink::prelude::{
        // string::String,
        vec::Vec
    };
    use job::Job;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum MLNetError {
        InvalidJobFormat,
        InvalidLayer,
        UserAlreadyExists,
        UserNotFound
    }

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

    #[ink(event)]
    pub struct LayerEvent {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        layer: Layer,
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
        model: Vec<Layer>,
        // User role in ml-net (0: sensory, 1: inter,
        // role: i8
    }

    impl UserCache {
        pub fn new() -> Self {
            Self {
                y_cache: Vec::new(),
                y_loc: Vec::new(),
                y_ind: 0,
                model: Vec::new(),
                // role
            }
        }
    }

    /// Framework for an ML training and execution platform
    #[ink(storage)]
    pub struct MLNet {
        author: AccountId,
        reward: Balance,
        /// Kind of ML task initiated (0: discrete bloom, 1: linked bloom, 2: cascade, 3: ensemble,
        /// 4: execute)
        kind: i8,
        /// Distribution of participation-based reward (singular: false or uniform: true)
        reward_distribution: bool,
        /// Status of job (open: true, closed: false)
        open: bool,
        /// Keeps track of user contributions
        user_cache: Mapping<AccountId, UserCache>,
        /// Specify neural network layer architeceture
        model_info: Vec<i64>,
        dim: Layer,
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
            model_info: Vec<i64>,
            dim: Layer
            // participation: Mapping::new(),
            // filters,
            // max_responses,
            // formatting_tips,
            // data: String (some way of direting the user to the data (via arweave or url)
        ) -> Self {
            Self {
                author,
                reward,
                reward_distribution,
                kind,
                open: true,
                user_cache: Mapping::new(),
                model_info,
                dim
            }
        }

        #[ink(message)]
        pub fn join(&mut self) -> Result<(), MLNetError> {
            let caller: AccountId = Self::env().caller();

            // Check that user hasn't joined already
            if !self.user_cache.contains(&caller) {
                self.user_cache.insert(&caller, &UserCache::new());
            } else {
                return Err(MLNetError::UserAlreadyExists);
            }

            Ok(())
        }


        #[ink(message)]
        pub fn submit_y(&mut self, y_ind: i64, y_pred: Layer) -> Result<(), MLNetError> {
            let caller: AccountId = Self::env().caller();

            self.assert_layer(&y_pred)?;

            if let Some(mut cache) = self.user_cache.get(&caller) {
                cache.y_cache.push(y_pred);
                cache.y_loc.push(y_ind);
                cache.y_ind += 1;
                self.user_cache.insert(&caller, &cache);
            } else {
                return Err(MLNetError::UserNotFound);
            }

            Ok(())
        }

        #[ink(message)]
        pub fn get_y(&self, user: AccountId, y_ind: i64) -> Result<Option<Layer>, MLNetError> {
            if let Some(cache) = self.user_cache.get(user) {
                let ind = y_ind as usize;
                return Ok(cache.y_cache.get(ind).cloned());
            } else {
                return Err(MLNetError::UserNotFound);
            }
        }

        #[ink(message)]
        pub fn submit_model(&mut self, model: Vec<Layer>) -> Result<(), MLNetError> {
            let caller: AccountId = Self::env().caller();

            for layer in &model {
                self.assert_layer(layer)?;
            }

            if let Some(mut cache) = self.user_cache.get(&caller) {
                cache.model = model;
                self.user_cache.insert(&caller, &cache);
            }

            Ok(())
        }

        fn assert_layer(&self, layer: &Layer) -> Result<(), MLNetError> {
            match (layer, &self.dim) {
                (Layer::DimOne(_), Layer::DimOne(_)) => Ok(()),
                (Layer::DimTwo(_), Layer::DimTwo(_)) => Ok(()),
                (Layer::DimThree(_), Layer::DimThree(_)) => Ok(()),
                _ => Err(MLNetError::InvalidLayer),
            }
        }

        fn assert_model(&self, model: Vec<Layer>) -> Result<(), MLNetError> {
            for layer in model {
                self.assert_layer(&layer)?;
            }

            Ok(())
        }

        // fn calculate_loss(&mut self, y_pred: Layer) {
        //     unimplemented!()
        // }
    }

    impl Job for MLNet {
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

        #[ink(message)]
        fn get_proof(&self) {
            unimplemented!()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{test, DefaultEnvironment};

        #[ink::test]
        pub fn ml_works() {
            let accounts = test::default_accounts::<DefaultEnvironment>();
            let addresses: Vec<AccountId> = vec![
                accounts.alice, accounts.bob, accounts.charlie,
                accounts.eve, accounts.django, accounts.frank
            ];
            let event_subscriber = test::recorded_events();

            // Main user defines ml-net
            let mut net: MLNet = MLNet::new(
                accounts.alice,
                0,
                true,
                0,
                vec![10, 32, 16, 2],
                Layer::DimTwo(vec![])
            );

            // Add all users to job
            for address in addresses {
                test::set_caller::<DefaultEnvironment>(address);
                net.join();
            }

            // Test vector to send
            let layer = Layer::DimTwo(
                vec![
                    vec![1_000, 100_014, 5_909, 22_311],
                    vec![200, 120_934, 423_897, 4_382],
                    vec![42_901, 71_203, 909_090, 22_222],
                ]
            );

            // Simulate 10 blocks of activity on the discrete-bloom network
            // for block in 0..10 {
            //     for address in &addresses {
            //         if address.clone() == net.author {
            //
            //         } else {
            //
            //         }
            //     }
            // }

            test::set_caller::<DefaultEnvironment>(accounts.alice);
            net.submit_y(0, layer);

            if let Some(y_vec) = net.get_y(accounts.alice, 0).unwrap() {
                match y_vec {
                    Layer::DimOne(values) => { unimplemented!() }
                    Layer::DimTwo(values) => {
                        println!("{:?}", values);
                        println!("Passed.")
                    }
                    Layer::DimThree(values) => { unimplemented!() }
                }
            } else {
                println!("Failed.");
            }

            // if let Layer::DimTwo(data) = layer {
            //     let mut output = <Sha2x256 as HashOutput>::Type::default();
            //     let hash = ink::env::hash_bytes::<Sha2x256>(&data, &mut output);
            //     println!("{}", hash);
            // }

            // let mut hash_output = <Sha2x256 as HashOutput>::Type::default();
            // hash_output.clone_from_slice(&ink_env::hash::hash::<Sha2x256, _>(&encoded_data));
            // hash_output
        }
    }
}
