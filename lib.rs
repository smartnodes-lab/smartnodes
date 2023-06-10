#![cfg_attr(not(feature = "std"), no_std, no_main)]

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
        participants: Vec<AccountId>,
        open: bool,

        // true: distributed among voters, false: random single distribution
        reward_distribution: bool,

        // max_votes: Option<u32>,
        // cost_per_vote: Balance,
        // recommended_format: Option<String>,
    }

    #[derive(scale::Decode, scale::Encode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct User {
        address: AccountId,
        username: String,
        descriptors: Vec<String>,
        cv: Vec<String>
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
        // Declaring TaskNet environment (storage variables for contract)
        polls: Mapping<i64, Poll>,
        next_poll_id: i64,
        users: Mapping<AccountId, User>
    }

    impl TaskNet {
        #[ink(constructor)]
        pub fn new() -> Self {
            // Instantiate TaskNet environment
            Self {
                polls: Mapping::new(),
                next_poll_id: 0,
                users: Mapping::new()
            }
        }

        #[ink(message)]
        pub fn create_poll(
            &mut self,
            title: String,
            description: String,
            reward: Balance,
            reward_distribution: bool
        ) {
            let author: AccountId = Self::env().caller();

            // Create Poll if user is signed on the network
            if self.users.contains(author) {
                let poll: Poll = Poll {
                    author,
                    title,
                    description,
                    reward,
                    responses: Vec::new(),
                    participants: Vec::new(),
                    open: true,
                    reward_distribution
                };

                // Insert poll to contract and update poll_id
                self.polls.insert(self.next_poll_id, &poll);
                self.next_poll_id += 1;
            }
        }

        fn close_poll(&self, poll_id: i64) {
            let caller: AccountId = Self::env().caller();

            // Close poll if caller is the poll author
            // For future reference, only allow poll to close if the reward-type was specified or a
            // a problem occurs (rewards should be locked and sent to participants)
            if self.users.contains(caller) {
                if let Some(mut poll) = self.get_poll(poll_id) {
                    if poll.author == caller {
                        poll.open = false;
                    }
                }
            }
        }

        #[ink(message)]
        pub fn create_user(
            &mut self,
            username: String,
            descriptors: Option<Vec<String>>,
            cv: Option<Vec<String>>
        ) {
            let caller = Self::env().caller();

            // Create user if address isn't linked to an account
            if !self.users.contains(caller) {
                let mut user = User {
                    address: caller,
                    username,
                    descriptors: Vec::new(),
                    cv: Vec::new()
                };

                // If descriptors were specified, add to descriptors
                if let Some(descriptors) = descriptors {
                    for descriptor in descriptors {
                        user.descriptors.push(descriptor);
                    }
                }

                if let Some(cv) = cv {
                    // do some cv processing here (one String per job experience? skills?)
                }

                // Add user to contract
                self.users.insert(caller, &user);
            }
        }

        #[ink(message)]
        pub fn respond_to_poll(&mut self, poll_id: i64, response:String) {
            let caller = Self::env().caller();

            // If user is not the author and the poll exists, allow response
            // Maybe add limits for participant # and user filters
            if self.users.contains(caller) {
                if let Some(mut poll) = self.get_poll(poll_id) {
                    if !poll.participants.contains(&caller) {
                        poll.participants.push(caller);
                        poll.responses.push(response);
                    }
                }
            }
        }

        #[ink(message)]
        pub fn get_poll(&self, poll_id: i64) -> Option<Poll> {
            return self.polls.get(poll_id);
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
        //
        // // Module for selecting key, value pairs denoted by String("Key: Value")
        // let key_value: Vec<String> = descriptor.split(": ")
        //     .map(|s| s.to_string())
        //     .collect();
        // let _key = &key_value[0];
        // let _value = &key_value[1];
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        pub fn tasknet_works() {
            let mut net: TaskNet = TaskNet::new();

            net.create_user(
                String::from("jumbomeats"),
                Some(vec![
                    String::from("Gender: Male"),
                    String::from("Age: 21"),
                    String::from("Occupation: Astronaut")
                    ]),
                None
            );

            net.create_poll(
                String::from("Lens Network Flagged Comment: Your mom's a wanker"),
                String::from("Was this post harmful? (y/n)"),
                1,
                true
            );

            let poll = net.get_poll(0).unwrap();

            println!("Poll Title: {}", poll.title);
            println!("Description: {}", poll.description);
            println!("Reward: {} AZERO", poll.reward);

        }
    }
}
