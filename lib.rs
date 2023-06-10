#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod tasknet {
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;
    use ink::prelude::string::String;

    #[derive(scale::Decode, scale::Encode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Poll {
        author: AccountId,
        title: String,
        description: String,
        reward: Balance,
        responses: Vec<String>,
        participants: Vec<AccountId>
    }

    #[derive(scale::Decode, scale::Encode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct User {
        address: AccountId,
        username: String,
        descriptors: Vec<String>,
    }

    #[derive(Debug, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum TaskNetError {
        UserAlreadyExists,
        UserAlreadyResponded,
        TaskRewardTooLow
    }

    #[ink(storage)]
    pub struct TaskNet {
        polls: Mapping<i64, Poll>,
        next_poll_id: i64,
        users: Vec<User>
    }

    impl TaskNet {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                polls: Mapping::new(),
                next_poll_id: 0,
                users: Vec::new()

            }
        }

        #[ink(message)]
        pub fn create_poll(
            &mut self,
            title: String,
            description: String,
            reward: Balance
        ) {
            let author: AccountId = Self::env().caller();
            let poll: Poll = Poll {
                author,
                title,
                description,
                reward,
                responses: Vec::new(),
                participants: Vec::new()
            };

            self.polls.insert(self.next_poll_id, &poll);
            self.next_poll_id += 1;
        }

        #[ink(message)]
        pub fn close_poll(&self, poll_id: i64) {
            let caller: AccountId = Self::env().caller();

            if let Some(poll) = self.polls.get(poll_id) {
                if poll.author == caller {
                    self.polls.remove(poll_id);
                }
            }
        }

        #[ink(message)]
        pub fn display_poll(&self, poll_id: i64) -> Option<Poll> {
            return self.polls.get(poll_id);
        }

        #[ink(message)]
        pub fn create_user(&mut self, username: String) {
            let caller = Self::env().caller();
            let user = User {
                address: caller,
                username,
                descriptors: Vec::new()
            };

            self.users.push(user);
        }

        #[ink(message)]
        pub fn respond_to_poll(&mut self, poll_id: i64, response:String) {
            let caller = Self::env().caller();

            if let Some(mut poll) = self.polls.get(&poll_id) {
                if !poll.participants.contains(&caller) {
                    poll.participants.push(caller);
                    poll.responses.push(response);
                }
            }
        }

        // #[ink(message)]
        // pub fn get_user_polls(&self) -> Vec<Poll> {
        //     let caller = Self::env().caller();
        //     let mut user_polls = Vec::new();
        //     for poll in self.polls {
        //         if poll.author == caller {
        //             user_polls.push(poll.clone());
        //         }
        //     }
        //     return user_polls;
        // }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        pub fn tasknet_works() {
            let mut net: TaskNet = TaskNet::new();

            net.create_poll(
                String::from("Flagged Comment: I like pokeman cards."),
                String::from("Is this post harmful? (y/n)"),
                1
            );
            net.create_poll(
                String::from("Flagged Comment: Your mom is a whore"),
                String::from("Is this post harmful? (y/n)"),
                1
            );

            let poll = net.polls.get(1).unwrap();

            println!("Poll Title: {}", poll.title);
            println!("Description: {}", poll.description);
            println!("Reward: {} AZERO", poll.reward);
        }
    }
}
