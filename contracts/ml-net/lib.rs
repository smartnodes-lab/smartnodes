#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use ml_net::{MLNetError, MLNetRef};

#[ink::contract]
mod ml_net {
    use ink::{
        prelude::vec::Vec,
        reflect::ContractEventBase,
        storage::Mapping,
    };
    use job::Job;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum MLNetError {
        InvalidFormat,
        InvalidProof,
        UserAlreadyExists,
        UserNotFound
    }

    type Event = <MLNet as ContractEventBase>::Type;

    #[derive(scale::Decode, scale::Encode, Debug, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub enum Vector {
        /// One dimensional layer
        DimOne(Vec<i64>),
        /// Two dimensional layer
        DimTwo(Vec<Vec<i64>>),
        /// Three dimensional layer
        DimThree(Vec<Vec<Vec<i64>>> ),
    }

    #[derive(scale::Decode, scale::Encode, Debug, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Layer {
        weights: Vector,
        biases: Vector
    }

    #[ink(event)]
    pub struct Proof {
        // #[ink(topic)]
        // y_vec: Layer,
        #[ink(topic)]
        y_loc: i64,
    }

    /// To store y_pred sumissions and any other data relevant for proving and improving training
    #[derive(scale::Decode, scale::Encode, Debug, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct UserCache {
        id: AccountId,
        y_cache: Vec<Vector>,
        y_loc: Vec<i64>,
        model: Vec<Layer>,
        // role: i8
    }

    impl UserCache {
        pub fn new(id: AccountId) -> Self {
            Self {
                id,
                y_cache: Vec::new(),
                y_loc: Vec::new(),
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
        reward_distribution: bool,
        kind: i8,
        open: bool,
        // The good stuff
        model_info: Vec<i64>,
        dim: Vector,
        user_cache: Vec<UserCache>,
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
            dim: Vector
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
                user_cache: Vec::new(),
                model_info,
                dim
            }
        }

        #[ink(message)]
        pub fn submit_proof(&mut self, y_ind: i64, y_pred: Vector) -> Result<(), MLNetError> {
            let caller: AccountId = Self::env().caller();

            // Check the input matches that of the network
            self.assert_layer(&y_pred)?;

            // Check user exists and is the caller
            if let Some(cache_ind) = self.user_cache
                .iter()
                .position(|user| user.id == caller)
            {
                let Some(mut cache) = self.user_cache.get(cache_ind);
                if let Some(proof_ind) = cache.y_loc
                    .iter()
                    .position(|ind| ind == &y_ind)
                {
                    if cache.y_cache.len() == proof_ind {
                        cache.y_cache.push(y_pred);
                        self.user_cache.insert(cache_ind, cache.clone());
                    } else {
                        return Err(MLNetError::InvalidProof);
                    }
                }
            } else {
                return Err(MLNetError::UserNotFound);
            }

            Ok(())
        }

        #[ink(message)]
        pub fn request_proof(&self, y_loc: i64) {
            let caller: AccountId = Self::env().caller();

            if caller == self.author {
                self.env().emit_event(Proof { y_loc });

                for user_cache in self.user_cache {

                }
            }
        }

        #[ink(message)]
        pub fn get_y(&self, user_id: AccountId, y_ind: i64) -> Result<Option<Vector>, MLNetError> {
            if let Some(cache) = self.user_cache
                .iter()
                .find(|user| user.id == user_id)
            {
                let ind = y_ind as usize;

                if let Some(vec) = cache.y_cache.get(ind) {
                    return Ok(Some(vec.clone()));
                } else {
                    return Err(MLNetError::InvalidProof);
                }
            } else {
                return Err(MLNetError::UserNotFound);
            }
        }

        // #[ink(message)]
        // pub fn submit_model(&mut self, model: Vec<Layer>) -> Result<(), MLNetError> {
        //     let caller: AccountId = Self::env().caller();
        //
        //     for layer in &model {
        //         self.assert_layer(layer)?;
        //     }
        //
        //     if let Some(mut cache) = self.user_cache.get(&caller) {
        //         cache.model = model;
        //         self.user_cache.insert(&caller, &cache);
        //     }
        //
        //     Ok(())
        // }

        #[ink(message)]
        pub fn join(&mut self) -> Result<(), MLNetError> {
            let caller: AccountId = Self::env().caller();

            // Check that user hasn't joined already
            if self.user_cache.iter().all(|user| user.id != caller) {
                self.user_cache.push(UserCache::new(caller));
            } else {
                return Err(MLNetError::UserAlreadyExists);
            }

            Ok(())
        }

        fn assert_layer(&self, layer: &Vector) -> Result<(), MLNetError> {
            match (layer, &self.dim) {
                (Vector::DimOne(_), Vector::DimOne(_)) => Ok(()),
                (Vector::DimTwo(_), Vector::DimTwo(_)) => Ok(()),
                (Vector::DimThree(_), Vector::DimThree(_)) => Ok(()),
                _ => Err(MLNetError::InvalidFormat),
            }
        }

        // fn assert_model(&self, model: Vec<Layer>) -> Result<(), MLNetError> {
        //     for layer in model {
        //         self.assert_layer(&layer)?;
        //     }
        //
        //     Ok(())
        // }

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
                Vector::DimTwo(vec![])
            );

            // Add all user to job
            for address in addresses {
                test::set_caller::<DefaultEnvironment>(address);
                net.join();
            }

            // Test vector to send
            let layer = Vector::DimTwo(
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
            net.submit_proof(0, layer);

            if let Some(y_vec) = net.get_y(accounts.alice, 0).unwrap() {
                match y_vec {
                    Vector::DimOne(values) => { unimplemented!() }
                    Vector::DimTwo(values) => {
                        println!("{:?}", values);
                        println!("Passed.")
                    }
                    Vector::DimThree(values) => { unimplemented!() }
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
