#![cfg_attr(not(feature = "std"), no_std, no_main)]

//     #[derive(scale::Decode, scale::Encode, Debug)]
//     #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]

#[ink::contract]
mod framework {
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;
    use ink::prelude::string::String;
    use user::User;
    use job::Job;

    #[derive(Debug, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum SubNetError {
        UserAlreadyExists,
        UserAlreadyResponded,
        TaskRewardTooLow
    }

    // Introduce the SubNet architecture, each must have the ability to deploy and remove tasks
    pub trait SubNet {
        fn create(&mut self, title: String, description: String);
        fn close(&self, task_id: i64);
        fn dispute(&self, task_id: i64);
        fn respond(&self, task_id: i64);
    }

    #[ink(storage)]
    pub struct Framework {
        // Declaring TaskNet environment (storage variables for contract)
        next_task_id: i64,
        tasknet: Mapping<i64, Task>,
        jobnet: Mapping<i64, Job>,
        mlnet: Mapping<i64, ML>,
        users: Mapping<AccountId, User>
    }

    impl TaskNet {
        #[ink(constructor)]
        pub fn new() -> Self {
            // Instantiate TaskNet environment
            Self {
                next_task_id: 0,
                tasks: Mapping::new(),
                users: Mapping::new()
            }
        }

        #[ink(message, payable)]
        pub fn create_task(
            &mut self,
            title: String,
            description: String,
            // reward_distribution: bool
        ) {
            let author: AccountId = Self::env().caller();

            // Create task if user is signed on the network
            if self.users.contains(author) {
                let task: Task = Task {
                    author,
                    title,
                    description,
                    reward: self.env().transfered_value(),
                    responses: Mappin::new(),
                    // participants: Vec::new(),
                    // open: true,
                    // reward_distribution
                };

                // Insert task to contract and update task_id
                self.tasks.insert(self.next_task_id, &task);
                self.next_task_id += 1;
            }
        }

        #[ink(message)]
        pub fn close_task(&self, task_id: i64) {
            let caller: AccountId = Self::env().caller();

            // Close task if caller is the task author
            // For future reference, only allow task to close if the reward-type was specified or a
            // a problem occurs (rewards should be locked and sent to participants)
            if self.users.contains(caller) {
                if let Some(mut task) = self.get_task(task_id) {
                    if task.author == caller {
                        task.open = false;
                    }
                }
            }
        }

        #[ink(message)]
        pub fn create_user(
            &mut self,
            username: String,
            experience: Option<Vec<String>>,
            skills: Option<Vec<String>>
        ) {
            let caller = Self::env().caller();

            // Create user if address isn't linked to an account
            if !self.users.contains(caller) {
                let mut user = User::new(
                    username,
                    experience,
                    skills
                );

                // Add user to contract
                self.users.insert(caller, &user);
            }
        }

        #[ink(message)]
        pub fn respond_to_task(&mut self, task_id: i64, response:String) {
            let caller = Self::env().caller();

            // If user is not the author and the task exists, allow response
            if self.users.contains(caller) {
                if let Some(mut task) = self.get_task(task_id) {
                    // Later add limits for participant #s and user filters
                    if !task.participants.contains(&caller) {
                        task.participants.push(caller);
                        task.responses.push(response);
                    }
                }
            }
        }

        #[ink(message)]
        pub fn get_task(&self, task_id: i64) -> Option<task> {
            return self.tasks.get(task_id);
        }

        // #[ink(message)]
        // pub fn get_user_tasks(&self) -> Vec<task> {
        //     let caller = Self::env().caller();
        //     let mut user_tasks = Vec::new();
        //     for task in self.tasks {
        //         if task.author == caller {
        //             user_tasks.push(task.clone());
        //         }
        //     }
        //     return user_tasks;
        // }
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

            net.create_task(
                String::from("Lens Network Flagged Comment: Your mom's a wanker"),
                String::from("Was this post harmful? (y/n)"),
                1,
            );

            let task = net.get_task(0).unwrap();

            println!("task Title: {}", task.title);
            println!("Description: {}", task.description);
            println!("Reward: {} AZERO", task.reward);

        }
    }
}
