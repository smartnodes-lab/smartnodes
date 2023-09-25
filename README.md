# Framework (ChainSpace Incentive Layer)

## Main Concept
- expansive user-reputation system for peer-to-peer (P2P) interactions unlocking the potential of online and off-chain work
- integrates participation & reputation-based metrics, rewards, and a seamless UI to form a vibrant marketplace of peer-to-peer tasks
  - proofs of jobs/interactions are stored with each user to secure the network and aid in the recruitment of users for tasks
- Use-cases include but are not limited to:
  - The moderation of online and real-world ecosystems (social media, insurance)
  - On-demand ML model execution and training
  - Marketplace for jobs & job connections, utilizing the unique proofs for better recruitment of employees
  - Various tasks whos work can be proven & stored in some way on the blockchain (e.g. homework help, coding problems, data analysis, writing etc.)
- APIs in multiple programming languages, allowing users to seamlessly connect their systems to the network
- Allows automated systems to manage problems & workflows using collective human intelligence, as well as distributed computations
- Multiple blockchains for different scalability/security trade-offs and native reward coins/tokens

## Users
- Descriptors/filter words (i.e. sex, DoB, nationality, religion, occupation, interests, skills)
- Hold reputation and/or hash of previous completed jobs (eg ratings, % accuracy, majority votes)
 
## ML-Net
- Co-operative and competitive AI training & model execution, potentially even markets for models and datasets
  -  "Cloud network architecture" for distributed machine learning execution
  -  On-demand API calls to these models for execution (e.g. for an LLM-based helper-bot online)
- Types of ML task ideas:
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
  - **sense**: takes input data and feeds it to another user
  - **inter**: takes user data and feeds it to another user (abstract)
  - **integ**: combine previous data to an output whose loss can be calculated
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
- can be closed manually, after a certain number of votes is reached, or ultimately after a number of blocks to manage contract storage
- cost can be fixed/proportional to number or quality of voters 
- poll description can include recommended format/responses (e.g. multiple choice), this can help the automated system 
  determine when to close the poll and the quality of the answers
- If the answer is time consuming, once a user has submitted the task is locked with the reward and the issuer can 
  accept or decline the solution. If declined, the disputed task must be re-instantiated to the tasknet (or maybe a 
  dispute-net) to vote on the outcome (return funds, re-post the task, etc)

## Preventing Abuse
- Off-chain data stream for fast data transfer, proofs can be prioritized on-chain
- Repuation can be a combination of a score provided by the users (i.e. the 'employers'), the number of tasks completed, % accepted solutions/majority vote, and decreasing loss/increasing accuracy of validation data
- Contest-net where users can dispute through a reporting system (disputed poll is fed back to a seperate dispute network)
  - all users are required to participate, similar to a jury