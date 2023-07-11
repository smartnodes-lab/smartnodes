#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use ml_net::{MLNetError, MLNetRef};

#[ink::contract]
mod ml_net {
    use ink::{
        prelude::vec::Vec,
        reflect::ContractEventBase,
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
                author: Self::env().caller(),
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
            if let Some(mut cache) = self.get_cache(caller) {
                if let Some(proof_ind) = cache.y_loc
                    .iter()
                    .position(|ind| ind == &y_ind)
                {
                    // Add proof
                    if cache.y_cache.len() == proof_ind {
                        cache.y_cache.push(y_pred);
                        self.update_cache(cache);
                    } else {
                        return Err(MLNetError::InvalidProof);
                    }
                }
            }

            Ok(())
        }

        #[ink(message)]
        pub fn request_proof(&mut self, y_loc: i64) {
            let caller: AccountId = Self::env().caller();

            if caller == self.author {
                self.env().emit_event(Proof { y_loc });

                for i in 0..self.user_cache.len() {
                    if let Some(cache) = self.user_cache.get(i) {
                        let mut cache = cache.clone();
                        cache.y_loc.push(y_loc);
                        self.user_cache.insert(i, cache);
                    }
                }
            }
        }

        #[ink(message)]
        pub fn get_y(&self, user_id: AccountId, y_ind: i64) -> Result<Option<Vector>, MLNetError> {
            if let Some(cache) = self.get_cache(user_id) {
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

        #[ink(message)]
        pub fn submit_model(&mut self, model: Vec<Layer>) -> Result<(), MLNetError> {
            let caller: AccountId = Self::env().caller();

            self.assert_model(&model);

            if let Some(mut cache) = self.get_cache(caller) {
                cache.model = model;
                self.update_cache(cache);
            }

            Ok(())
        }

        fn get_cache(&self, address: AccountId) -> Option<UserCache> {
            if let Some(cache) = self.user_cache
                .iter()
                .find(|user| user.id == address)
            {
                return Some(cache.clone());
            } else {
                return None;
            }
        }

        fn update_cache(&mut self, new_cache: UserCache) {
            let caller: AccountId = Self::env().caller();

            if let Some(cache_ind) = self.user_cache
                .iter()
                .position(|user| user.id == caller)
            {
                self.user_cache.insert(cache_ind, new_cache);
            }
        }

        fn assert_layer(&self, layer: &Vector) -> Result<(), MLNetError> {
            match (layer, &self.dim) {
                (Vector::DimOne(_), Vector::DimOne(_)) => Ok(()),
                (Vector::DimTwo(_), Vector::DimTwo(_)) => Ok(()),
                (Vector::DimThree(_), Vector::DimThree(_)) => Ok(()),
                _ => Err(MLNetError::InvalidFormat),
            }
        }

        fn assert_model(&self, model: &Vec<Layer>) -> Result<(), MLNetError> {
            unimplemented!()
        }

        fn calculate_loss(&mut self, y_pred: Layer) {
            unimplemented!()
        }
    }

    impl Job for MLNet {
        #[ink(message)]
        fn join(&mut self) {
            let caller: AccountId = Self::env().caller();

            // Check that user hasn't joined already
            if self.user_cache.iter().all(|user| user.id != caller) {
                self.user_cache.push(UserCache::new(caller));
            }
        }

        #[ink(message)]
        fn open(&mut self) {
            self.open = true;
            todo!("add reward mechanism")
        }

        #[ink(message)]
        fn close(&mut self) {
            self.open = false;
            todo!("assert reward + dispute mechanisms")
        }

        #[ink(message)]
        fn dispute(&mut self) {
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

            // Main user defines ml-net
            set_caller(accounts.alice);
            let mut net: MLNet = MLNet::new(
                0,
                true,
                0,
                vec![10, 32, 16, 2],
                Vector::DimTwo(vec![])
            );

            // Add all user to job
            for address in &addresses {
                set_caller(address.clone());
                net.join();
            }

            // Contract author requests miners submit validation data / proof
            set_caller(accounts.alice);
            net.request_proof(0);

            // Test vector to send
            let test_vec = Vector::DimTwo(
                vec![
                    vec![1_000, 100_014, 5_909, 22_311],
                    vec![200, 120_934, 423_897, 4_382],
                    vec![901, 71_203, 909_090, 22_222],
                ]
            );

            // Simulate 10 requests for proof
            for i in 0..10 {
                // Contract author requests miners submit validation data / proof
                set_caller(accounts.alice);
                net.request_proof(i);

                for address in &addresses {
                    set_caller(address.clone());
                    net.submit_proof(i, test_vec.clone());
                }
            }

            let mut proofs_sent = 0;
            let mut participation = 0;
            let proofs_requested = test::recorded_events().count();

            for user in net.user_cache {
                proofs_sent += user.y_cache.capacity();
                if user.y_cache.len() > 0 { participation += 1; }
            }

            println!(
                "Users: {}, proofs requested: {}, proofs submitted: {}.", participation,
                proofs_requested, proofs_sent
            );

            // if let Layer::DimTwo(data) = layer {
            //     let mut output = <Sha2x256 as HashOutput>::Type::default();
            //     let hash = ink::env::hash_bytes::<Sha2x256>(&data, &mut output);
            //     println!("{}", hash);
            // }
            // let mut hash_output = <Sha2x256 as HashOutput>::Type::default();
            // hash_output.clone_from_slice(&ink_env::hash::hash::<Sha2x256, _>(&encoded_data));
            // hash_output
        }

        fn set_caller(caller: AccountId) {
            test::set_caller::<DefaultEnvironment>(caller);
        }

        fn get_latest_event() -> Option<test::EmittedEvent> {
            let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            emitted_events.last().cloned()
        }
    }
}
