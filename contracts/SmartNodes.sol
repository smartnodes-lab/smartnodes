// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

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
        uint256 unlockTime;
        uint8 reputation;
        bool isActive;
    }

    struct User {
        uint256 id;
        address userAddress;
        uint8 reputation;
    }

    struct Job {
        uint256 id;
        address owner;
        uint256 capacity;
        address[] validatorAddresses;
        bool isCompleted;
    }

    struct JobCreationRequest {
        uint256 id;
        address creator;
        uint256 capacity;
        address[] validatorAddresses;
        mapping(address => bool) signatures;
    }

    // Validator mappings
    mapping(uint256 => Validator) public validators;
    mapping(address => uint256) public validatorIdByAddress;
    mapping(string => uint256) public validatorIdByHash;

    // User mappings
    mapping(uint256 => User) public users;
    mapping(address => uint256) public userIdByAddress;

    // Job mappings
    mapping(uint256 => Job) public jobs;
    mapping(uint256 => JobCreationRequest) public pendingJobs;

    // Counters
    uint256 public validatorIdCounter = 1;
    uint256 public userIdCounter = 1;
    uint256 public jobIdCounter = 1;
    uint256 public requiredSignatures = 2; // Minimum required signatures for any job

    // Events
    event ValidatorCreated(uint256 indexed id, address validatorAddress);
    event TokensLocked(address indexed validator, uint256 amount);
    event UnlockInitiated(address indexed validator, uint256 unlockTime);
    event TokensUnlocked(address indexed validator, uint256 amount);

    constructor() ERC20("TensorLink", "TLINK") {
        _mint(msg.sender, 100);
    }

    /// Role creation methods (User & Validator) ///
    function createUser() external {
        require(userIdByAddress[msg.sender] == 0, "User already registered.");
        users[userIdCounter] = User({
            id: userIdCounter,
            userAddress: msg.sender,
            reputation: 0 // Initialize reputation if applicable
        });

        userIdByAddress[msg.sender] = userIdCounter++;
    }

    function createValidator(string memory publicKeyHash) external {
        require(
            validatorIdByHash[publicKeyHash] == 0,
            "Validator with this publicKeyHash already exists."
        );

        validators[validatorIdCounter] = Validator({
            id: validatorIdCounter,
            validatorAddress: msg.sender,
            publicKeyHash: publicKeyHash,
            locked: 0,
            unlockTime: 0,
            reputation: 0,
            isActive: true
        });

        validatorIdByAddress[msg.sender] = validatorIdCounter;
        validatorIdByHash[publicKeyHash] = validatorIdCounter;

        emit ValidatorCreated(validatorIdCounter, msg.sender);

        validatorIdCounter++;
    }

    /// Validator Token Locking and Unlocking ///
    function lockTokens(uint256 amount) external {
        require(amount > 0, "Amount must be greater than zero.");
        require(balanceOf(msg.sender) >= amount, "Insufficient balance.");

        uint256 validatorId = validatorIdByAddress[msg.sender];
        require(validatorId != 0, "Validator does not exist.");
        Validator storage validator = validators[validatorId];
        require(validator.isActive, "Validator is not active.");

        transferFrom(msg.sender, address(this), amount);
        validator.locked += amount;

        emit TokensLocked(msg.sender, amount);
    }

    function unlockTokens(uint256 amount) external {
        uint256 validatorId = validatorIdByAddress[msg.sender];
        Validator storage validator = validators[validatorId];

        require(amount <= validator.locked, "Amount exceeds locked balance.");
        require(amount > 0, "Amount must be greater than zero.");

        // Initialize the unlock time if it's the first unlock attempt
        if (validator.unlockTime == 0) {
            validator.unlockTime = block.timestamp + 1 years; // e.g., set to 1 year from now
            emit UnlockInitiated(msg.sender, validator.unlockTime); // Optional: emit an event
        } else {
            // On subsequent attempts, check if the unlock period has elapsed
            require(
                block.timestamp >= validator.unlockTime,
                "Tokens are still locked."
            );

            validator.locked -= amount;
            _mint(msg.sender, amount); // Mint tokens back to the validator's address

            emit TokensUnlocked(msg.sender, amount); // Optional: emit an event when tokens are unlocked
        }
    }

    // User Job Requesting
    function requestJob(uint256 capacity) external {
        require(
            users[userIdByAddress[msg.sender]] != 0,
            "User must be registered."
        );
        require(capacity > 0, "Capacity must be greater zero.");
        // require(workerAddresses.length > 0)

        JobCreationRequest storage request = pendingJobs[jobIdCounter++];
        request.id = jobIdCounter;
        request.creator = msg.sender;
        request.capacity = capacity;
        request.workerAddresses = workerAddresses;

        // Select validators pseudorandomly
        address[] memory selectedValidators = _pseudorandomValidatorSelection(
            requiredSignatures
        );
        request.validatorAddresses = selectedValidators; // Store the selected validators

        jobIdCounter++; // Increment jobIdCounter after storing the job request
    }

    // Validator job request voting
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

    function _countSignatures(
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

    function _pseudorandomValidatorSelection(
        uint8 nValidators
    ) internal view returns (address[] memory) {
        require(
            nValidators > 0 && nValidators <= validatorIdCounter - 1,
            "Invalid number of validators requested."
        );

        address[] memory selectedValidators = new address[](nValidators);
        uint256[] memory seenIndices = new uint256[](nValidators);
        uint256 randIndex = uint256(
            keccak256(
                abi.encodePacked(block.timestamp, block.difficulty, msg.sender)
            )
        ) % validatorIdCounter;
        uint256 count = 0;

        while (count < nValidators) {
            if (
                !validators[randIndex].isActive || seenIndices[randIndex] == 1
            ) {
                randIndex = (randIndex + 1) % validatorIdCounter;
                continue;
            }
            seenIndices[randIndex] = 1;
            selectedValidators[count++] = validators[randIndex]
                .validatorAddress;
            randIndex = (randIndex + 1) % validatorIdCounter;
        }

        return selectedValidators;
    }

    function getJobIdCounter() external view returns (uint256) {
        return jobIdCounter;
    }

    function getValidatorIdCounter() external view returns (uint256) {
        return validatorIdCounter;
    }
}
