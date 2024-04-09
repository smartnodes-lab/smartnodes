// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract SmartNodes is ERC20 {
    // // Modifier to require multisignature validation
    // modifier onlyMultiSig() {
    //     // Check if the function invocation is authorized by a specified number of validators
    //     require(
    //         countSignatures(msg.sender) >= requiredSignatures,
    //         "Multisignature validation required"
    //     );
    //     _;
    // }

    struct Validator {
        uint256 id;
        address validatorAddress;
        string publicKeyHash;
        uint32 locked;
        bool isActive;
    }

    struct Job {
        uint256 id;
        address owner;
        uint256 capacity;
        address[] workerAddresses; // Changed to workerAddresses to store addresses directly
        bool isCompleted;
    }

    struct JobCreationRequest {
        uint256 id;
        address creator;
        uint256 capacity;
        address[] workerAddresses; // Changed to workerAddresses to store addresses directly
        mapping(address => bool) signatures;
    }

    mapping(address => uint256) public validatorIdByAddress;
    mapping(string => uint256) public validatorIdByHash;
    mapping(uint256 => Validator) public validators;
    uint256 public validatorIdCounter;

    mapping(uint256 => Job) public jobs;
    uint256 public jobIdCounter;

    mapping(uint256 => JobCreationRequest) public pendingJobs;
    uint256 public requiredSignatures = 2; // Number of signatures required for job creation

    constructor() ERC20("TensorLink", "TLINK") {
        _mint(msg.sender, 100);
        validatorIdCounter = 1;
        jobIdCounter = 1;
    }

    function getJobIdCounter() external view returns (uint256) {
        return jobIdCounter;
    }

    function getValidatorIdCounter() external view returns (uint256) {
        return validatorIdCounter;
    }

    function createValidator(string memory publicKeyHash) external {
        Validator memory validator = Validator({
            id: validatorIdCounter,
            validatorAddress: msg.sender,
            publicKeyHash: publicKeyHash,
            locked: 0,
            isActive: true
        });

        validators[validatorIdCounter] = validator;
        validatorIdByAddress[msg.sender] = validatorIdCounter;
        validatorIdByHash[publicKeyHash] = validatorIdCounter;
        validatorIdCounter++;
    }

    function lockTokens(uint256 amount) external {
        require(amount > 0, "Amount must be greater than zero.");
        require(balanceOf(msg.sender) >= amount, "Insufficient balance.");

        uint256 validatorId = validatorIdByAddress[msg.sender];
        Validator storage validator = validators[validatorId];

        validator.locked += uint32(amount);
        _burn(msg.sender, amount);
    }

    function unlockTokens(uint256 amount) external {
        uint256 validatorId = validatorIdByAddress[msg.sender];
        Validator storage validator = validators[validatorId];

        require(amount <= validator.locked, "Amount exceeds locked balance.");

        validator.locked -= uint32(amount);
        _mint(msg.sender, amount);
    }

    function createJob(
        uint256 capacity,
        address[] memory workerAddresses
    ) external {
        require(capacity > 0, "Capacity must be greater than zero.");
        require(
            workerAddresses.length > 0,
            "At least one worker must be specified."
        );

        uint256 validatorId = validatorIdByAddress[msg.sender];
        Validator storage validator = validators[validatorId];
        require(validator.isActive, "Validator is not active.");

        // Create the JobCreationRequest struct without initializing the signatures mapping
        JobCreationRequest storage newRequest = pendingJobs[jobIdCounter];
        newRequest.id = jobIdCounter;
        newRequest.creator = msg.sender;
        newRequest.capacity = capacity;
        newRequest.workerAddresses = workerAddresses;

        jobIdCounter++;

        // Iterate over the workerAddresses array to initialize the signatures mapping entries
        for (uint256 i = 0; i < workerAddresses.length; i++) {
            newRequest.signatures[workerAddresses[i]] = false;
        }

        // Set the signature for the creator of the job
        newRequest.signatures[msg.sender] = true;
    }

    function approveJobCreation(uint256 jobId) external {
        uint256 validatorId = validatorIdByAddress[msg.sender];
        Validator storage validator = validators[validatorId];
        require(validator.isActive, "Validator is not active.");
        require(
            pendingJobs[jobId].id == jobId,
            "Job creation request does not exist."
        );

        JobCreationRequest storage request = pendingJobs[jobId];
        request.signatures[msg.sender] = true;

        if (countSignatures(request) == requiredSignatures) {
            Job memory job = Job({
                id: jobId,
                owner: request.creator,
                capacity: request.capacity,
                workerAddresses: request.workerAddresses,
                isCompleted: false
            });

            jobs[jobId] = job;
            delete pendingJobs[jobId];
        }
    }

    function countSignatures(
        JobCreationRequest storage request
    ) internal view returns (uint256) {
        uint256 count = 0;
        for (uint256 i = 0; i < request.workerAddresses.length; i++) {
            if (request.signatures[request.workerAddresses[i]]) {
                count++;
            }
        }

        return count;
    }
}
