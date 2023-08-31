#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod usernet {

    use ink::{
        prelude::{
            vec::Vec
        },
        storage::Mapping
    };

    type UserId = u32;
    type ConnectionId = u32;
    type JobId = u32;

    const MAX_FILTER_WORDS: u8 = 5;

    /// Holds key user connectivity info,
    #[derive(scale::Decode, scale::Encode, Debug, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Connection {
        author: i64,
        funding: Balance,
        description: String,
        uids: Vec<i64>,
    }

    #[derive(Default, scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(
            Debug,
            PartialEq,
            Eq,
            scale_info::TypeInfo,
            ink::storage::traits::StorageLayout
        )
    )]
    pub struct Connections {
        connections: Vec<ConnectionId>,
        next_id: ConnectionId
    }

    /// Holds key individual data (pub key, rep, keywords)
    #[derive(scale::Decode, scale::Encode, Debug, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct User {
        public_key: [u8; 392],
        locked: Balance,
        reputation: i8,
        filters: [String; 6],
    }

    #[derive(Default, scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(
            Debug,
            PartialEq,
            Eq,
            scale_info::TypeInfo,
            ink::storage::traits::StorageLayout
        )
    )]
    pub struct Users {
        users: Vec<UserId>,
        next_id: UserId
    }

    #[ink(storage)]
    pub struct UserNet {
        users: Mapping<UserId, User>,
        user_list: Users,
        connections: Connections
    }
}