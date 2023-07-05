# Framework

## Main Concept
- An oracle-like network that integrates participation/reputation metrics, and a seamless UI to allow a broader range of computation and 
  tasks to be carried out
- Multiple APIs for programming languages and multiple blockchains
- connecting users to a competitive marketplace of problem solvers
- Allows automated systems to manage problems using human and collective intelligence
- Use-cases include but are not limited to:
  - a marketplace for jobs, opinions, and other tasks (homework, coding problems, data analysis, writing help, etc.)
    - initial focus on tasks that can be completed or verified over the internet, by anyone
  - the moderation of online and real-world ecosystems (social media, insurance)
  - on-demand ML model execution and training
 
## ML-Net
- co-operative and competitive AI training, as well as markets for models and datasets
  -  cloud network architecture for distributed machine learning execution
  -  on-demand API calls to these models for execution (e.g. for an LLM-based helper-bot online)
- types of ML-Tasks:
  - **bloom**: concurrent random initialized weights
    - layer_dims: input, output
    - workflow: pull data -> model forward pass -> loss -> model backward pass -> update
  - **cascade**: models combine to form one super-model
    - layer_dims: input, fully connected layer(s), output
    - workflow: pull data from source -> model forward pass -> loss -> model backward pass -> update
  - **ensemble**: multiple unique models being trained/executed and provided to the user
    - layer_dims: input, output
  - **execute**: execution of a specific model
    - layer_dims: all dims
- types of ML-Users:
  - **sensory**: takes input data and feeds it to another user
  - **interneuron**: takes user data and feeds it to another user (abstract)
  - **integrate**: combine previous data to an output whose loss can be calculated
- TODO:
  - integrate execution & learning types
  - job hashing / proof of work
  - contract for each ml-net?
  
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

## Users
- descriptors/filter words (i.e. sex, DoB, nationality, religion, occupation, interests)
- hold reputation of previous polls (% accuracy / majority votes for instance) and a hash to the completed tasks

## Preventing Abuse
- socket stream along side blockchain for fast data transfer, proofs can be prioritized on-chain
- repuation can be a score provided by the users, automated reputation based on accepted solutions / majority vote for tasks,
  and decreasing loss / increased accuracy over time, tasks completed
- users can dispute through a reporting system (disputed poll is fed back to the network or a seperate dispute network)
