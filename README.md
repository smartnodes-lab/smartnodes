# Framework

## Main Concept

- An oracle network that integrates participation and reputation metrics to allow for a broader range of computation and 
  tasks to be addressed
- A seamless UI and multiple APIs that connect users to a competitive marketplace of problem solvers
- Allows automated systems to manage problems using human and collective intelligence
- Use-cases include but are not limited to:
  - a marketplace for jobs, opinions, and other tasks
  - the moderation of online and real-world ecosystems (social media, insurance)
  - on-demand AI model execution
  - jobs/tasks that can be done or verified over the internet (coding problems, writing, homework help, stackoverflow/reddit etc.)

## Task-Net

- The most basic task will be a multiple choice/voting problem and will contain a title, description (problem), possible
  repsonses, reward, and a list of users
- Once a task is cast to the network (i.e. question or censuses), then a random or filtered selection of users are asked
  to respond to the task
- Tasks have a locked reward, which can be distributed among the majority voters or randomly
- can be closed manually, after a certain number of votes is reached, or after a number of blocks
- cost can be fixed/proportional to number or quality of voters 
- poll description can include recommended format/responses (e.g. multiple choice), this can help the automated system 
  determine when to close the poll and the quality of the answers
- If the answer is time consuming, once a user has submitted the task is locked with the reward and the issuer can 
  accept or decline the solution. If declined, the disputed task must be re-instantiated to the tasknet (or maybe a 
  dispute-net) to vote on the outcome (return funds, re-post the task, etc)

## ML-Net

- co-operative and competitive AI training, as well as markets for models and datasets
  -  cloud network architecture for distributed machine learning execution
  -  on-demand API calls to these models for execution (e.g. for an LLM-based helper-bot online)
- Types of ML-Tasks:
  - **bloom**: concurrent random initialized weights
    - layer_dims: input, output
  - **cascade**: models combine to form one super-model
    - layer_dims: input, fully connected layer(s), output
  - **ensemble**: multiple unique models being trained/executed and provided to the user
    - layer_dims: input, output

## Users

- descriptors/filter words (i.e. sex, DoB, nationality, religion, occupation, interests)
- hold reputation of previous polls (% accuracy / majority votes for instance) and a hash to the completed tasks

## Preventing Abuse

- socket stream along side blockchain for fast data transfer, proofs can be prioritized on-chain
- repuation can be a score provided by the users, automated reputation based on accepted solutions / majority vote for tasks,
  and decreasing loss / increased accuracy over time, tasks completed
- users can dispute through a reporting system (disputed poll is fed back to the network or a seperate dispute network)
