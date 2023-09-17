#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use self::usernet::UserNet;

#[ink::contract]
mod usernet {

    use ink::{
        prelude::{
            vec::Vec,
            string::String
        },
        storage::Mapping
    };

    type UserId = u32;
    type ConnectionId = u32;
    type JobId = u32;

    const MAX_FILTER_WORDS: u8 = 5;

    #[derive(Clone, Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum UserNetError {
        UserExists,
        FundingTooLow
    }

    /// Holds key user connectivity info for data streams
    #[derive(scale::Decode, scale::Encode, Debug, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Connection {
        host: i64,
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

    /// Holds key individual data (f key, rep, keywords)
    #[derive(scale::Decode, scale::Encode, Debug, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct User {
        address: AccountId,
        locked: Balance,
        reputation: i8,
        filters: [[u8; 15]; 6],
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

    impl UserNet {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                users: Mapping::new(),
                user_list: Users::default(),
                connections: Connections::default()
            }
        }

        #[ink(message, payable)]
        pub fn create_user(
            &mut self,
            user: User
        ) -> Result<(), UserNetError> {
            let caller: AccountId = self.env().caller();

            if self.user_list.users.iter().all(|user_address| user_address != caller) {
                let user_id = self.user_list.next_id;
                self.user_list.next_id =
                    user_id.checked_add(1).expect("UserIds Exhausted!");
                self.users.insert(user_id, user);
                self.user_list.users.push(user_id);
                self.env().emit_event()
            }

            Ok(())
        }

        #[ink(message)]
        pub fn get_user(&self) -> Result<UserNet, UserNetError> {
            let caller = self.env().caller();
            let user = self.users.get(&user_id).ok_or(UserNetError::UserNotFound)?;

            ensure_caller_is_user(&user);
            Ok(User)
        }

        fn ensure_caller_is_user(&self, user: &User) {
            assert_eq!(self.env().caller(), user.address);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn constructor_works() {
            let contract = UserNet::new();
        }
    }
}
