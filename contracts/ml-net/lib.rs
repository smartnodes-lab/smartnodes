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

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum MLNetError {
        InvalidNetFormat,
        InvalidNetKind,
        UserAlreadyExists,
        UserNotFound,
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
        from: Option<AccountId>,
        layer: Layer
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
        input_dim: u8,
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
            input_dim: u8
            // participation: Mapping::new(),
            // filters,
            // max_responses,
            // formatting_tips,
            // data: String (some way of direting the user to the data (via arweave or url)
        ) -> Self {
            let specified_dims: usize = model_info.capacity();

            // Enforce network structure depending on kind
            if kind == 0 {
                // Discrete bloom network must contain specified structure (dimensions)
                if specified_dims < 2 {
                    ink::env::debug_println!("Model must have at least two layers (in, out)");
                    Self::env().terminate_contract(author);
                }
            } else if kind == 1 {
                if specified_dims <= 2 {
                    ink::env::debug_println!("Invalid network format for given kind!");
                    Self::env().terminate_contract(author);
                }
            } else if kind == 2 {
                if !specified_dims == 2 {
                    ink::env::debug_println!("Invalid network format for kind!");
                    Self::env().terminate_contract(author);
                }
            } else if kind == 3 {
                if specified_dims < 2 {
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
                user_cache: Mapping::new(),
                model_info,
                input_dim
            }
        }

        #[ink(message)]
        pub fn join_net(&mut self) -> Result<(), MLNetError> {
            let caller: AccountId = Self::env().caller();

            // Check that user hasn't joined already
            if !self.user_cache.contains(caller) {
                self.user_cache.insert(caller, &UserCache::new());
            } else {
                return Err(MLNetError::UserAlreadyExists);
            }

            Ok(())
        }

        #[ink(message)]
        pub fn submit_y(&mut self, y_ind: i64, y_pred: Layer) -> Result<(), MLNetError> {
            let caller: AccountId = Self::env().caller();

            if let Some(mut cache) = self.user_cache.get(caller) {
                cache.y_cache.push(y_pred);
                cache.y_loc.push(y_ind);
                cache.y_ind += 1;
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
        pub fn submit_loss(&mut self, loss: Layer) {
            let caller: AccountId = Self::env().caller();

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
        use super::*;
        use ink::env::{test, DefaultEnvironment};

        #[ink::test]
        pub fn ml_works() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            // Main user defines ml-net
            let mut net: MLNet = MLNet::new(
                accounts.alice,
                0,
                true,
                0,
                vec![10, 32, 16, 2],
                2
            );

            test::set_caller::<DefaultEnvironment>(accounts.alice);
            net.join_net();

            let layer = Layer::DimTwo(
                vec![
                    vec![1000, 10000124, 2, 123112973],
                    vec![0, 9120934, 14238974, 4382],
                    vec![1, 1203, 1, 2],
                ]
            );

            test::set_caller::<DefaultEnvironment>(accounts.alice);
            net.submit_y(10, layer);
            test::advance_block::<DefaultEnvironment>;

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
        }
    }
}
