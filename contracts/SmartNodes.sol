// SPDX-License-Identifier: MIT
pragma solidity ^0.8.5;

import "@openzeppelin-upgradeable/contracts/token/ERC20/ERC20Upgradeable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@chainlink/contracts/src/v0.8/vrf/dev/VRFConsumerBaseV2Upgradeable.sol";
import "@chainlink/contracts/src/v0.8/interfaces/VRFCoordinatorV2Interface.sol";
import {VRFV2PlusClient} from "@chainlink/contracts/src/v0.8/vrf/dev/libraries/VRFV2PlusClient.sol";

/**
 * @title SmartNodes
 * @dev An ERC20 contract for managing off-chain networks
 */
contract SmartNodes is ERC20Upgradeable, VRFConsumerBaseV2Upgradeable {
    // ERC20 token supply metrics
    uint256 public constant initialSupply = 10_000_000e18;
    uint256 public constant maxSupply = 21_000_000e18;
    uint256 public constant emissionRate = 8e18; // amount of tokens to be emitted per state update
    uint256 public constant lockAmount = 5e18; // minimum validator locked tokens required
    uint256 public constant halving = 2; //52_560; // number of state updates until next halving

    // Validator multi-sig address
    address public validatorContract;

    // Counters for storage indexing / IDs
    uint256 public validatorIdCounter;
    uint256 public userIdCounter;
    uint256 public jobCounter;
    uint256 public minValidators;
    uint256 public maxValidators;

    // Chainlink VRF Parameters
    uint64 s_subscriptionId;
    address linkAddress = 0x779877A7B0D9E8603169DdbD7836e478b4624789;
    bytes32 s_keyHash =
        0x787d74caea10b2b357790d5b5247c2f63d1d91572a9846f780606e4d953677ae;
    uint32 callbackGasLimit = 100000;
    uint16 requestConfirmations = 2;
    uint32 numWords = 1;
    address vrfCoordinator;
    VRFCoordinatorV2Interface COORDINATOR;

    // Events
    event ValidatorCreated(uint256 indexed id, address validatorAddress);
    event TokensLocked(address indexed validator, uint256 amount);
    event UnlockInitiated(address indexed validator, uint256 unlockTime);
    event TokensUnlocked(address indexed validator, uint256 amount);
    event ValidatorActivated(uint256 indexed validatorId);
    event ValidatorDeactivated(uint256 indexed validatorId);
    event JobRequested(uint256 indexed jobId);
    event JobStarted(uint256 indexed jobId);
    event JobCompleted(uint256 indexed jobId);

    // User data with key information (address, RSA key hash, reputation)
    struct User {
        uint256 id;
        address userAddress;
        uint8 reputation;
    }

    // Validator data with key information (address, RSA key hash, locked value, reputation, activity)
    struct Validator {
        uint256 id;
        address validatorAddress;
        uint256 locked;
        uint256 unlockTime;
        uint8 reputation;
        bool isActive;
    }

    // Validator data with key information (address, RSA key hash, reputation)
    struct Worker {
        uint256 id;
        address userAddress;
        uint8 reputation;
    }

    // Information for  a generic off-chain job (job hash [key for kademlia lookup], seed validators, participants, author, etc)
    struct Job {
        uint256 id;
        address owner;
        uint256 capacity;
        uint256[] validatorIds;
    }

    // Main datastructure mappings via id lookup
    mapping(uint256 => User) public users;
    mapping(uint256 => Validator) public validators;
    mapping(uint256 => Worker) public workers;
    mapping(uint256 => Job) public jobs;

    // Helpful mappings
    mapping(address => uint256) public userIdByAddress;
    mapping(address => uint256) public validatorIdByAddress;

    mapping(string => uint256) public userIdByHash;
    mapping(string => uint256) public validatorIdByHash;
    mapping(string => uint256) public jobIdByHash;
    mapping(uint256 => uint256) public jobIdByUser;
    mapping(uint256 => uint256) public jobIdByRequestId;

    mapping(uint256 => string) public validatorKeyById;
    mapping(uint256 => string) public userKeyById;

    mapping(uint256 => bool) public validatorActivity;
    mapping(uint256 => bool) public userActivity;
    mapping(uint256 => bool) public jobActivity;

    modifier onlyValidatorMultiSig() {
        require(
            msg.sender == validatorContract,
            "Caller must be the ValidatorMultiSig contract"
        );
        _;
    }

    function initialize(
        address _validatorContract,
        address _vrfCoordinator,
        uint64 _subscriptionId
    ) public initializer {
        __ERC20_init("Smart Nodes", "SNO");
        __VRFConsumerBaseV2_init(_vrfCoordinator);
        COORDINATOR = VRFCoordinatorV2Interface(_vrfCoordinator);

        // Set all counters to 1 (when looking up values, 0 = Not found)
        validatorContract = _validatorContract;
        validatorIdCounter = 1;
        vrfCoordinator = _vrfCoordinator;
        s_subscriptionId = _subscriptionId;
        userIdCounter = 1;
        jobCounter = 1;
        minValidators = 1;
        maxValidators = 3;
    }

    /**
     * @dev Create a User, limit one per address & public key hash (RSA)
     */
    function createUser(string memory publicKey) external {
        // Only one address & public key hash per user.
        require(userIdByAddress[msg.sender] == 0, "User already registered.");

        users[userIdCounter] = User({
            id: userIdCounter,
            userAddress: msg.sender,
            reputation: 0
        });

        userIdByAddress[msg.sender] = userIdCounter;
        userKeyById[userIdCounter] = publicKey;
        userIdCounter++;
    }

    /**
     * @dev Create a Validator, limit one per address & public key hash (RSA), requires locking up sno
     */
    function createValidator(string memory publicKey) external {
        require(
            validatorIdByAddress[msg.sender] == 0,
            "Validator with this publicKeyHash already exists"
        );

        require(
            balanceOf(msg.sender) >= lockAmount,
            "Insufficient token balance."
        );

        // Lock token in contract
        bool success = transferFrom(msg.sender, address(this), lockAmount);

        require(success, "Token transfer failed");

        validators[validatorIdCounter] = Validator({
            id: validatorIdCounter,
            validatorAddress: msg.sender,
            locked: lockAmount,
            unlockTime: 0,
            reputation: 100,
            isActive: true
        });

        validatorKeyById[userIdCounter] = publicKey;
        validatorIdByAddress[msg.sender] = validatorIdCounter;
        validatorActivity[validatorIdCounter] = true;
        validatorIdCounter++;
    }

    // User Job Requesting
    function requestJob(uint8 nValidators, uint256 capacity) external {
        uint256 uid = userIdByAddress[msg.sender];
        require(uid != 0, "User not registered.");
        require(capacity > 0, "Capacity must be greater zero.");
        require(userActivity[uid] == false, "User has an active job.");
        require(
            nValidators > 0 && nValidators < maxValidators,
            "Invalid validator count."
        );

        // Request random numbers for selecting validators
        uint256 requestId = COORDINATOR.requestRandomWords(
            s_keyHash,
            s_subscriptionId,
            requestConfirmations,
            callbackGasLimit,
            numWords
        );

        jobIdByUser[uid] = jobCounter;
        jobIdByRequestId[requestId] = jobCounter;
        jobActivity[jobCounter] = false;
        // Store the job in the jobs mapping
        jobs[jobCounter] = Job({
            id: jobCounter,
            owner: msg.sender,
            capacity: capacity,
            validatorIds: new uint256[](nValidators)
        });

        emit JobRequested(jobCounter);
        jobCounter++;
    }

    // Select random validators for job once a random number is received
    function fulfillRandomWords(
        uint256 requestId,
        uint256[] memory randomWords
    ) internal override {
        require(
            msg.sender == vrfCoordinator,
            "Only VRFCoordinator can fulfill"
        );
        require(randomWords.length > 0, "No random words provided");

        // Assume only one random word is requested
        uint256 randomNumber = randomWords[0];
        uint256 jobId = jobIdByRequestId[requestId];
        uint256 nValidators = jobs[jobId].validatorIds.length;

        // Use randomNumber to select validators
        uint256[] memory selectedValidators = new uint256[](nValidators);

        // Logic to select validators based on random numbers
        for (uint8 i = 0; i < nValidators; i++) {
            uint256 randomIndex = randomNumber % validatorIdCounter;
            uint256 validatorId = ((randomIndex + i) % validatorIdCounter) + 1;

            // Skip inactive validators
            while (!validatorActivity[validatorId]) {
                validatorId = (validatorId % validatorIdCounter) + 1;
            }

            selectedValidators[i] = validatorId;
        }

        // Update the job with the selected validators
        jobs[jobId].validatorIds = selectedValidators;

        // Emit an event or update the job struct with the selected validators
        // emit ValidatorsSelected(jobId, selectedValidators);
    }

    // function _updateValidatorState(
    //     uint256 validatorId,
    //     bool activate
    // ) internal {
    //     require(
    //         validatorId > 0 && validatorId < validatorIdCounter,
    //         "Invalid ValidatorId"
    //     );
    //     require(
    //         validatorStateById[validatorId] > 0,
    //         "Validator is not online."
    //     );

    //     if (activate) {
    //         // Move validator to the active state if not already active
    //         if (validatorStateById[validatorId] == 1) {
    //             validatorStateById[validatorId] = 2;
    //         } else {
    //             revert("Validator being activated is already active!");
    //         }
    //     } else {
    //         // Move validator to the inactive state if not already inactive
    //         if (validatorStateById[validatorId] == 2) {
    //             validatorStateById[validatorId] = 1;
    //         } else {
    //             revert("Validator is already inactive!");
    //         }
    //     }
    // }

    // function _activateValidator(uint256 validatorId) internal {
    //     require(
    //         0 < validatorId && validatorId < validatorIdCounter,
    //         "Validator ID must be valid."
    //     );

    //     Validator storage validator = validators[validatorId];
    //     require(!validator.isActive, "Validator already active!");
    //     validator.isActive = true;
    //     validatorStateById[validatorId] = 1;
    //     emit ValidatorActivated(validatorId);
    // }

    // function _deactivateValidator(uint256 validatorId) internal {
    //     require(
    //         0 < validatorId && validatorId < validatorIdCounter,
    //         "Validator ID must be valid."
    //     );
    //     Validator storage validator = validators[validatorId];

    //     require(validator.isActive, "Validator not active!");
    //     validator.isActive = false;

    //     validatorStateById[validatorId] = 0;
    //     emit ValidatorActivated(validatorId);
    // }

    /**
    TODO
     * @dev Update the contract state, to be called by the ValidatorMultiSig contract.
     * Updates job, worker, user, and validator key information, mines and distributes rewards.
     */
    function updateValidator(
        uint256 validatorId,
        uint8 newReputation
    ) external onlyValidatorMultiSig {
        require(validatorId != 0, "Invalid validator ID");
        require(
            newReputation >= 0 && newReputation <= 100,
            "Invalid reputation value"
        );

        validators[validatorId].reputation = newReputation;
    }

    function updateJob(
        uint256 jobId,
        bool active
    ) external onlyValidatorMultiSig {
        require(jobId != 0, "Invalid job ID");

        jobActivity[jobId] = active;

        if (active) {
            emit JobStarted(jobId);
        } else {
            emit JobCompleted(jobId);
        }
    }

    function updateWorker(
        uint256 workerId,
        uint8 newReputation
    ) external onlyValidatorMultiSig {
        require(workerId != 0, "Invalid worker ID");
        require(
            newReputation >= 0 && newReputation <= 100,
            "Invalid reputation value"
        );

        workers[workerId].reputation = newReputation;
    }

    /**
     * @dev Validator token unlocking, 30 day withdrawal period.
     */
    function lockTokens(uint32 amount) external {
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

    function unlockTokens(uint32 amount) external {
        uint256 validatorId = validatorIdByAddress[msg.sender];
        Validator storage validator = validators[validatorId];

        require(amount <= validator.locked, "Amount exceeds locked balance.");
        require(amount > 0, "Amount must be greater than zero.");

        // Initialize the unlock time if it's the first unlock attempt
        if (validator.unlockTime == 0) {
            validator.unlockTime = block.timestamp + 50400; // 14 day unlocking period
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

    /**
     * @dev Mint tokens for updating state rewards,
     TODO change to internal (external for testing)
     */
    function mintTokens(uint256 amount) external {
        require(
            totalSupply() + emissionRate <= maxSupply,
            "Maximum supply reached!"
        );
        // _mint(msg.sender, emissionRate);
        _mint(msg.sender, amount);
    }
}
