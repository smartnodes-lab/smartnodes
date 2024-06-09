// SPDX-License-Identifier: MIT
pragma solidity ^0.8.5;

import "@openzeppelin-upgradeable/contracts/token/ERC20/ERC20Upgradeable.sol";
import "./interfaces/ISmartnodesMultiSig.sol";

/**
 * @title SmartNodes
 * @dev An ERC20 contract for managing off-chain networks
 */
contract SmartnodesCore is ERC20Upgradeable {
    // ERC20 token supply metrics
    uint256 public constant INITIAL_SUPPLY = 8_000_000e18;
    uint256 public constant MAX_SUPPLY = 21_000_000e18;
    uint256 public halving = 52_560; // number of state updates until next halving
    uint256 public emissionRate = 128e18; // amount of tokens to be emitted per state update
    uint256 public lockAmount = 50_000e18; // minimum validator locked tokens required
    uint256 public unlockPeriod = 50400;

    // Validator multi-sig address
    ISmartnodesMultiSig private _validatorContractInstance;
    address public validatorContractAddress;

    // Counters for storage indexing / IDs
    uint256 public validatorIdCounter;
    uint256 public userIdCounter;
    uint256 public jobCounter;
    uint256 public activeValidators;
    uint256 public minValidators;
    uint256 public maxValidators;

    // Events
    event TokensLocked(address indexed validator, uint256 amount);
    event UnlockInitiated(address indexed validator, uint256 unlockTime);
    event TokensUnlocked(address indexed validator, uint256 amount);
    event JobRequested(uint256 indexed jobId, address[] seedValidators);
    event JobCompleted(uint256 indexed jobId);
    event JobDisputed(uint256 indexed jobId);

    // User data with key information (address, RSA key hash, reputation)
    struct User {
        uint256 id;
        address userAddress;
        bytes32 publicKeyHash;
        uint8 reputation;
    }

    // Validator data with key information (address, RSA key hash, locked value, reputation, activity)
    struct Validator {
        uint256 id;
        address validatorAddress;
        bytes32 publicKeyHash;
        uint256 locked;
        uint256 unlockTime;
        uint8 reputation;
    }

    // Information for  a generic off-chain job (job hash [key for kademlia lookup], seed validators, participants, author, etc)
    struct Job {
        uint256 id;
        address author;
        address[] seedValidators;
        address[] workers;
        uint256[] capacities;
        bool activity;
    }

    uint256[] public completedJobs;

    // Main datastructure mappings via id lookup
    mapping(uint256 => User) public users;
    mapping(uint256 => Validator) public validators;
    mapping(uint256 => Job) public jobs;

    // Helpful mappings
    mapping(address => uint256) public userIdByAddress;
    mapping(address => uint256) public validatorIdByAddress;

    modifier onlyValidatorMultiSig() {
        require(
            msg.sender == validatorContractAddress,
            "Caller must be the SmartnodesMultiSig."
        );
        _;
    }

    function initialize() public initializer {
        __ERC20_init("Smartnodes", "SNO");

        // Set all counters to 1 (when looking up values, 0 = Not found)
        userIdCounter = 1;
        jobCounter = 1;
        validatorIdCounter = 1;
        minValidators = 1;
        maxValidators = 3;
        completedJobs = new uint256[](5_000);
        emissionRate = 128e18; // amount of tokens to be emitted per state update
        lockAmount = 50_000e18; // minimum validator locked tokens required
        halving = 52_560;
        unlockPeriod = 50400;
    }

    function setValidatorContract(address validatorAddress) external {
        require(
            validatorContractAddress == address(0),
            "Validator address already set."
        );
        _validatorContractInstance = ISmartnodesMultiSig(validatorAddress);
        validatorContractAddress = validatorAddress;
    }

    /**
     * @dev Create a User, limit one per address & public key hash (RSA)
     */
    function createUser(bytes32 _publicKeyHash) external {
        // Only one address & public key hash per user.
        require(userIdByAddress[msg.sender] == 0, "User already registered.");

        users[userIdCounter] = User({
            id: userIdCounter,
            userAddress: msg.sender,
            publicKeyHash: _publicKeyHash,
            reputation: 50
        });

        userIdByAddress[msg.sender] = userIdCounter;
        userIdCounter++;
    }

    /**
     * @dev Create a Validator, limit one per address & public key hash (RSA), requires locking up sno
     */
    function createValidator(bytes32 _publicKeyHash) external payable {
        require(
            validatorIdByAddress[msg.sender] == 0,
            "Validator already exists."
        );

        require(
            balanceOf(msg.sender) >= lockAmount,
            "Insufficient token balance."
        );

        // Create validator on SmartnodesCore
        validators[validatorIdCounter] = Validator({
            id: validatorIdCounter,
            validatorAddress: msg.sender,
            publicKeyHash: _publicKeyHash,
            locked: lockAmount,
            unlockTime: 0,
            reputation: 50
        });

        validatorIdByAddress[msg.sender] = validatorIdCounter;

        // Lock token in contract
        _lockTokens(msg.sender, lockAmount);

        validatorIdCounter++;
    }

    // User Job Requesting Via Chainlink VRF TODO
    function requestJob(
        uint256[] calldata _capacities
    ) external returns (uint256[] memory validatorIds) {
        uint256 uid = userIdByAddress[msg.sender];

        require(uid != 0, "User not registered.");
        require(_capacities[0] > 0, "Capacity must be greater zero.");
        require(
            validatorContractAddress != address(0),
            "Validator contract not set!"
        );

        address[] memory _seedValidators = _validatorContractInstance
            .generateValidatorCandidates();
        uint256[] memory _validatorIds = new uint256[](_seedValidators.length);

        for (uint256 i = 0; i < _seedValidators.length; i++) {
            _validatorIds[i] = validatorIdByAddress[_seedValidators[i]];
        }

        // Store the job in the jobs mapping
        jobs[jobCounter] = Job({
            id: jobCounter,
            author: msg.sender,
            seedValidators: _seedValidators,
            workers: new address[](_capacities.length),
            capacities: _capacities,
            activity: true
        });

        emit JobRequested(jobCounter, _seedValidators);
        jobCounter++;

        return _validatorIds;
    }

    function completeJob(
        uint256 jobId,
        address[] memory _workers
    ) external onlyValidatorMultiSig returns (uint256[] memory) {
        require(_workers.length == jobs[jobId].capacities.length);
        jobs[jobId].workers = _workers;
        jobs[jobId].activity = false;
        emit JobCompleted(jobId);
        if (completedJobs.length >= 5_000) {
            completedJobs.pop();
        }
        completedJobs.push(jobId);
        return jobs[jobId].capacities;
    }

    function disputeJob(uint256 jobId) external onlyValidatorMultiSig {
        Job storage job = jobs[jobId];
        job.activity = false;
        emit JobDisputed(jobId);
    }

    /**
     * @dev Internal function to lock tokens, callable from other functions
     */
    function _lockTokens(address sender, uint256 amount) internal {
        require(amount > 0, "Amount must be greater than zero.");
        require(balanceOf(sender) >= amount, "Insufficient balance.");

        uint256 validatorId = validatorIdByAddress[sender];
        require(validatorId != 0, "Validator does not exist.");

        transferFrom(sender, address(this), amount);
        validators[validatorId].locked += amount;
        uint256 totalLocked = validators[validatorId].locked;

        _validatorContractInstance.updateLockedTokens(
            sender,
            totalLocked,
            totalLocked >= lockAmount
        );

        emit TokensLocked(sender, amount);
    }

    /**
     * @dev Validator token unlocking, 30 day withdrawal period.
     */
    function lockTokens(uint32 amount) external {
        require(amount > 0, "Amount must be greater than zero.");
        require(balanceOf(msg.sender) >= amount, "Insufficient balance.");

        uint256 validatorId = validatorIdByAddress[msg.sender];
        require(validatorId != 0, "Validator does not exist.");

        transferFrom(msg.sender, address(this), amount);
        validators[validatorId].locked += amount;
        uint256 totalLocked = validators[validatorId].locked;

        _validatorContractInstance.updateLockedTokens(
            msg.sender,
            totalLocked,
            totalLocked >= lockAmount
        );

        emit TokensLocked(msg.sender, amount);
    }

    function unlockTokens(uint32 amount) external {
        uint256 validatorId = validatorIdByAddress[msg.sender];
        Validator storage validator = validators[validatorId];

        require(amount <= validator.locked, "Amount exceeds locked balance.");
        require(amount > 0, "Amount must be greater than zero.");

        // Initialize the unlock time if it's the first unlock attempt
        if (validator.unlockTime == 0) {
            validator.unlockTime = block.timestamp + unlockPeriod; // unlocking period

            // Update multisig validator
            uint256 totalLocked = validator.locked - amount;
            _validatorContractInstance.updateLockedTokens(
                msg.sender,
                totalLocked,
                totalLocked >= lockAmount
            );

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
    function mintTokens(address recipient, uint256 amount) external {
        require(
            totalSupply() + emissionRate <= MAX_SUPPLY,
            "Maximum supply reached!"
        );
        // _mint(msg.sender, emissionRate);
        _mint(recipient, amount);
    }

    // Returns a list of all the selected validators for a job request
    function getJobValidators(
        uint256 jobId
    ) external view returns (address[] memory) {
        require(jobId < jobCounter, "Invalid job ID");
        if (jobs[jobId].id == jobId) {
            address[] memory jobValidators = jobs[jobId].seedValidators;
            return jobValidators;
        } else {
            revert("Job not found!");
        }
    }

    // function updateSmartnodesValidatorContract() external {
    //     _
    // }

    function getUserCount() external view returns (uint256) {
        return userIdCounter - 1;
    }

    function getValidatorCount() external view returns (uint256) {
        return validatorIdCounter - 1;
    }

    function getActiveValidatorCount() external view returns (uint256) {
        return activeValidators;
    }

    function getEmissionRate() external view returns (uint256) {
        return emissionRate;
    }

    function getSupply() external view returns (uint256) {
        return this.totalSupply();
    }

    function isLocked(address validatorAddr) external view returns (bool) {
        uint256 _id = validatorIdByAddress[validatorAddr];
        return validators[_id].locked >= lockAmount;
    }
}
