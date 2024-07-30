# Introduction

Advancements in technology have continued to increase the demand for computing power and large scale data-driven resources. Access to resources is often a limiting factor and attainable by only select individuals and institutions. Likewise, these institutions are often at the forefront of developing these underlying technologies, creating barriers to its broader access and democratization. Smartnodes is an ecosystem of peer-to-peer networks that are designed to pool and deliver computational resources to users at scale. We aim to provide users access to data and computationally intensive resources for general and scientific use. Furthermore, Smartnodes aims to empower individuals by providing a robust framework for monetizing their hardware and intuition over the internet. Through smart contract-secured peer-to-peer (P2P) networks, we facilitate coordination of nodes and resources in an efficient way to accelerate and increase and improve existing models, workflows, decision-making processes, and general task delegation.

# Peer-to-Peer Networks

Smartnodes P2P networks enable efficient and secure interactions among validators and workers to provide users frictionless access to services. Each network is designed for a specific category of task, and specialized to aggregate, distribute, and deliver computational resources directly to users. Participants of the Smartnodes ecosystem possess a set of cryptographic keys that link their off-chain identity to the smart contract. This cryptographic layer ensures that only those with explicit authority to establish connections can do so, bolstering the security and authenticity of every interaction. Users can be classified into three categories.
Users interact with the Smartnodes network to submit job requests for access to the network’s pooled resources. They connect to validators to request jobs and initialize connections with workers for task execution.

Validators are responsible for the management of on and off-chain information. These validators operate on a specific P2P network, working to maintain a distributed hash table for node and job querying, while managing interactions between users and workers. They peer into worker-user processes, conduct proof-of-work requests to ensure proper actions, and update the job data structure with relevant information (e.g., worker reputations, job-specific data). Validators aggregate this off-chain data and vote on the integrity of these updates to the main contract, where information is finalized.

Workers are task-specific nodes that carry out jobs for users. They stand by and execute computational tasks assigned by validators, ensuring efficient completion of user requests.

Malicious workers are identified, and their reputations are updated or blacklisted based on their behavior.
By leveraging these mechanisms, Smartnodes creates a secure, efficient, and accessible platform for computational tasks, ensuring fair and transparent interactions among all participants.


# Smart Contracts

At the heart of the Smartnodes ecosystem lies a deployment of smart contracts (SCs) that handle several critical functions:

**Delegation of Roles and Permissions:** role-specific permissions to methods ensure proper access control, validator-multisig control over job and state updates.

**Storage of Worker and Validator History:** keeps track of the history and reputation of users, along with storage of key user and node data.

**Distribution of Rewards and Payments:** native token SNO generated and distributed fairly among workers and validators. To be used for payment of services and securing validator nodes.

The main contract is managed by a multisignature wallet composed of active, collateralized validators. Contract updates operate on a rolling basis, where proposals are submitted every 10 minutes by a random selection of validators. The proposal receiving the most votes triggers a contract update with the provided information, such as user registration, worker information, and job-related events. Every state update is accompanied by a token generation event which is distributed fairly among participating validators and workers. 
