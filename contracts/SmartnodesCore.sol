// SPDX-License-Identifier: MIT
pragma solidity ^0.8.5;

import "@openzeppelin-upgradeable/contracts/token/ERC20/ERC20Upgradeable.sol";
import "./interfaces/ISmartnodesMultiSig.sol";

/**
 * @title SmartNodes
 * @dev An ERC20 contract for managing off-chain networks
 */
contract SmartnodesCore is ERC20Upgradeable {
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
    event JobRequested(
        uint256 indexed jobId,
        uint256 timestamp,
        address[] seedValidators
    );
    event JobCompleted(uint256 indexed jobId, uint256 timestamp);
    event JobDisputed(uint256 indexed jobId, uint256 timestamp);

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

    // ERC20 token supply metrics
    uint256 public constant MAX_SUPPLY = 21_000_000e18;
    uint256 constant tailEmission = 1e18;

    uint256 public halving = 2160; // number of state updates until next halving (~3 months)
    uint256 public emissionRate = 2048e18; // amount of tokens to be emitted per state update
    uint256 public lockAmount = 50_000e18; // minimum validator locked tokens required
    uint256 public unlockPeriod = 50400;
    uint256 public timeSinceLastHalving;

    // Main datastructure mappings via id lookup
    mapping(uint256 => User) public users;
    mapping(uint256 => Validator) public validators;
    mapping(uint256 => Job) public jobs;

    uint256[] public activeJobs;

    // Helpful mappings
    mapping(address => uint256) public userIdByAddress;
    mapping(address => uint256) public validatorIdByAddress;
    mapping(bytes32 => uint256) public jobIdByUserIdHash; // temporary for validator job creation?

    modifier onlyValidatorMultiSig() {
        require(
            msg.sender == validatorContractAddress,
            "Caller must be the SmartnodesMultiSig."
        );
        _;
    }

    function initialize(
        address[] memory _leadDevelopers,
        address _communityWallet
    ) public initializer {
        __ERC20_init("Smartnodes", "SNO");

        // Set all counters to 1 (when looking up values, 0 = Not found)
        userIdCounter = 1;
        jobCounter = 1;
        validatorIdCounter = 1;

        // Set ERC20 token parameters
        emissionRate = 2048e18; // amount of tokens to be emitted per state update
        lockAmount = 50_000e18; // minimum validator locked tokens required
        halving = 2160;
        unlockPeriod = 50400; // (seconds)
        timeSinceLastHalving = 0;

        uint256 devWallets = 2_341_280e18;
        uint256 communityFunding = 2_000_000e18;
        uint256 idoAmount = 3_500_000e18;
        communityFunding += idoAmount; // We are sending the ido amount to the community wallet while we await an initial token sale

        for (uint i = 0; i < _leadDevelopers.length; i++) {
            _mint(_leadDevelopers[i], devWallets / _leadDevelopers.length);
        }

        _mint(_communityWallet, communityFunding);

        // Other parameters
        minValidators = 1;
        maxValidators = 3;
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
    function createValidator(
        bytes32 _publicKeyHash,
        uint256 _lockAmount
    ) external {
        require(
            validatorIdByAddress[msg.sender] == 0,
            "Validator already exists."
        );

        require(
            balanceOf(msg.sender) >= _lockAmount && _lockAmount >= lockAmount,
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

        emit JobRequested(jobCounter, block.timestamp, _seedValidators);
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
        // jobIdByUserIdHash[userIdHash] = 0;

        emit JobCompleted(jobId, block.timestamp);

        return jobs[jobId].capacities;
    }

    function disputeJob(uint256 jobId) external onlyValidatorMultiSig {
        Job storage job = jobs[jobId];
        job.activity = false;
        emit JobDisputed(jobId, block.timestamp);
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
    function lockTokens(uint256 amount) external {
        _lockTokens(msg.sender, amount);
    }

    function unlockTokens(uint256 amount) external {
        uint256 validatorId = validatorIdByAddress[msg.sender];
        require(validatorId > 0, "Not a registered validator.");

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
     * @dev Mint tokens for updating state rewards, distribute 40/60 among validators and workers
     * Updates the emission rate and halving accordingly
     */
    function mintTokens(
        address[] memory _workers,
        uint256[] memory _workerCapacities,
        uint256 _totalCapacity,
        address[] memory _validatorsVoted
    ) external onlyValidatorMultiSig {
        require(
            totalSupply() + emissionRate <= MAX_SUPPLY,
            "Maximum supply reached!"
        );

        if (timeSinceLastHalving >= halving) {
            if (emissionRate > 1e18) {
                emissionRate /= 2;
            }
        }

        uint256 validatorRewardTotal = (emissionRate * 40) / 100;
        uint256 workerRewardTotal = (emissionRate * 60) / 100;

        // Distribute rewards for validators equally
        uint256 validatorReward = validatorRewardTotal /
            _validatorsVoted.length;
        for (uint256 v = 0; v < _validatorsVoted.length; v++) {
            _mint(_validatorsVoted[v], validatorReward);
        }

        // Distribute rewards for workers
        for (uint256 w = 0; w < _workers.length; w++) {
            uint256 reward = ((_workerCapacities[w] * workerRewardTotal) /
                _totalCapacity);
            _mint(_workers[w], reward);
        }

        timeSinceLastHalving++;
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

    function getValidatorInfo(
        uint256 _validatorId
    ) external view returns (bool, bytes32) {
        require(_validatorId < validatorIdCounter, "Invalid ID.");
        Validator storage _validator = validators[_validatorId];
        bool isActive = _validatorContractInstance.isActiveValidator(
            _validator.validatorAddress
        );
        return (isActive, _validator.publicKeyHash);
    }

    function getUserCount() external view returns (uint256) {
        return userIdCounter - 1;
    }

    function getValidatorCount() external view returns (uint256) {
        return validatorIdCounter - 1;
    }

    function getActiveValidatorCount() external view returns (uint256) {
        return _validatorContractInstance.getNumValidators();
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

    function getProposees() external view returns (address[] memory) {
        return _validatorContractInstance.getSelectedValidators();
    }
}
