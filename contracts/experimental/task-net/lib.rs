#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use self::tasknet::TaskNetRef;

#[ink::contract]
mod tasknet {
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;
    use ink::prelude::string::String;

    #[derive(scale::Decode, scale::Encode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Task {
        author: AccountId,
        title: String,
        description: String,
        reward: Balance,
        reward_distribution: bool,
        participants: Vec<AccountId>,
        filters: Vec<String>,
        max_responses: i32, // interchangable with max_block_len?
        required_format: String, // can be used to justify disputes
        open: bool
    }

    #[ink(storage)]
    #[derive(Default)]
    pub struct TaskNet {
        tasks: Mapping<i64, Task>,
        next_task_id: i64
    }

    impl TaskNet {
        #[ink(constructor)]
        pub fn new() -> Self {
            // Instantiate TaskNet environment
            Self {
                next_task_id: 0,
                tasks: Mapping::new()
                // must implement pruning mechanism / way to hash the data
                // contained from the job such that it can still be displayed
                // and accessed online but doesn't take up much space
            }
        }

        #[ink(message, payable)]
        pub fn create(
            &mut self,
            title: String,
            description: String,
            reward_distribution: bool,
            filters: Vec<String>,
            max_responses: i32,
            required_format: String,
        ) {
            let author: AccountId = Self::env().caller();

            let task: Task = Task {
                author,
                title,
                description,
                reward: self.env().transferred_value(),
                reward_distribution,
                participants: Vec::new(),
                filters,
                max_responses, // interchangable with max_block_len?
                required_format, // can be used to justify disputes
                open: true
            };

            // Insert task-net to contract and update task_id
            self.tasks.insert(self.next_task_id, &task);
            self.next_task_id += 1;
        }

        #[ink(message)]
        pub fn close(&self, task_id: i64) {
            let caller: AccountId = Self::env().caller();

            // Close task-net if caller is the task-net author
            // For future reference, only allow task-net to close if the reward-type was specified or a
            // a problem occurs (rewards should be locked and sent to participants)
            if let Some(mut task) = self.get_task(task_id) {
                if task.author == caller {
                    task.open = false;
                }
            }
        }

        #[ink(message)]
        pub fn respond(&mut self, task_id: i64, response:String) {
            let caller = Self::env().caller();

            if let Some(mut task) = self.get_task(task_id) {
                if task.author != caller {
                    // Later add limits for participant #s and users filters
                    if !task.participants.contains(&caller) {
                        task.participants.push(caller);
                    }
                }
            }
        }

        #[ink(message)]
        pub fn dispute(&self, task_id: i64, note: String) {}

        #[ink(message)]
        pub fn get_task(&self, task_id: i64) -> Option<Task> {
            return self.tasks.get(task_id);
        }

        // #[ink(message)]
        // pub fn get_user_tasks(&self) -> Vec<task-net> {
        //     let caller = Self::env().caller();
        //     let mut user_tasks = Vec::new();
        //     for task-net in self.traits {
        //         if task-net.author == caller {
        //             user_tasks.push(task-net.clone());
        //         }
        //     }
        //     return user_tasks;
        // }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        pub fn tasknet_test() {
            let net: TaskNet = TaskNet::new();
        }
    }
}