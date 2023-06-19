# Framework

## Main Concept

- An oracle network that integrates participation and reputation metrics to allow for a broader range of computation and 
  tasks to be addressed
- A seamless UI and multiple APIs that connect users to a competitive marketplace of problem solvers
- Allows automated systems to manage problems using human and collective intelligence, whenever
  human input is required
- Servers as a marketplace for jobs, opinions, and other tasks
- May require seperate contracts for each use-case implementation, all task contracts and users could interact 
  through one main network contract
- Use-cases include but are not limited to: 
  - the moderation of online and real-world ecosystems (social media, insurance)
  - on-demand AI model execution
  - jobs/tasks that can be done or verified over the internet (coding problems, writing)
  - homework help
- There will also be the option for co-operative/competitive AI training, as well as markets for models and datasets
  -  cloud network architecture for distributed machine learning execution
  -  on-demand API calls to these models for execution (e.g. for an LLM-based helper-bot online)

## Tasks

- The most basic task will be a multiple choice/voting problem and will contain a title, description (problem), possible
  repsonses, reward, and a list of users
- Once a task is cast to the network (i.e. question or censuses), then a random or filtered selection of users are asked
  to respond to the task
- Tasks have a locked reward, which can be distributed among the majority voters or randomly
- can be closed manually, after a certain number of votes is reached, or after a number of blocks
- cost can be fixed/proportional to number or quality of voters 
- poll description can include recommended format/responses (e.g. multiple choice), this can help the automated system 
  determine when to close the poll and the quality of the answers

## Task types/formats

- Multiple choice (decision making, voting)
  - only allow Stings of particlar formats to be accepted
  - if tied, allow more votes
- text (writing, code)
  - If the answer is time consuming, once a user has submitted the task is locked with the reward and the issuer can 
    accept or decline the solution. If declined, the disputed task must be re-uploaded to the tasknet (or maybe a 
    resolution net) to vote on the outcome (return funds, re-post the task, etc)
- ML (solve a dataset, return the code, outputs, or allow for on-demand model execution)

## Computing System
- datasets can be uploaded/interacted with through the blockchain, allowing for various distributed forms of model training and execution
- users can compete to solve datasets or train collaboratively, depending on job type
- models can be instantiated and distributed to be executed on a large-scale for dApps and other applications

## Users
- username. AddressId, descriptors/filter words (i.e. sex, DoB, nationality, religion, occupation, interests)
- hold reputation of previous polls (% accuracy / majority votes for instance)

## Preventing Abuse
- pay to mint a user token (user smart contract)
- lock payment for work
- repuation / score for jobs
- users can dispute through a reporting system (disputed poll is fed back to the network to be voted upon again)
