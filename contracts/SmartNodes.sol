// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin-upgradeable/contracts/token/ERC20/ERC20Upgradeable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";

contract SmartNodes is Initializable, ERC20Upgradeable {
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
        string publicKeyHash;
        uint8 reputation;
    }

    struct Job {
        uint256 id;
        address owner;
        uint256 capacity;
        address[] validatorAddresses;
        bool isCompleted;
        mapping(address => bool) completeConfirmations;
        uint8 completeCount;
    }

    struct JobRequest {
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
    mapping(uint256 => string) public validatorHashById;
    mapping(uint256 => uint256) public validatorStateById;

    // User mappings
    mapping(uint256 => User) public users;
    mapping(address => uint256) public userIdByAddress;
    mapping(string => uint256) public userIdByHash;

    // Job mappings
    mapping(uint256 => Job) public jobs;
    mapping(uint256 => JobRequest) public jobRequests;

    // Counters for users
    uint256 public validatorIdCounter;
    uint256 public userIdCounter;
    uint256 public jobCounter;

    // Minimum and maximum required signatures for any job
    uint8 public minValidators;
    uint8 public maxValidators;

    // Events
    event ValidatorCreated(uint256 indexed id, address validatorAddress);
    event TokensLocked(address indexed validator, uint256 amount);
    event UnlockInitiated(address indexed validator, uint256 unlockTime);
    event TokensUnlocked(address indexed validator, uint256 amount);
    event ValidatorActivated(uint256 indexed validatorId);
    event ValidatorDeactivated(uint256 indexed validatorId);
    event JobCompleted(uint256 indexed jobId);

    function initialize() public initializer {
        __ERC20_init("TensorLink", "TLINK");
        _mint(msg.sender, 100);
        validatorIdCounter = 1;
        userIdCounter = 1;
        jobCounter = 1;
        minValidators = 2;
        maxValidators = 12;
    }

    // Role creation methods (User & Validator)
    function createUser(string memory publicKeyHash) external {
        require(userIdByAddress[msg.sender] == 0, "User already registered.");
        users[userIdCounter] = User({
            id: userIdCounter,
            userAddress: msg.sender,
            reputation: 0,
            publicKeyHash: publicKeyHash
        });

        userIdByHash[publicKeyHash] = userIdCounter;
        userIdByAddress[msg.sender] = userIdCounter++;
    }

    function createValidator(string memory publicKeyHash) external {
        require(
            validatorIdByHash[publicKeyHash] == 0,
            "Validator with this publicKeyHash already exists."
        );

        Validator storage validator = validators[validatorIdCounter];
        validator.id = validatorIdCounter;
        validator.validatorAddress = msg.sender;
        validator.publicKeyHash = publicKeyHash;
        validator.isActive = true;

        validatorIdByAddress[msg.sender] = validatorIdCounter;
        validatorIdByHash[publicKeyHash] = validatorIdCounter;
        validatorHashById[validatorIdCounter] = publicKeyHash;
        validatorStateById[validatorIdCounter] = 1;

        emit ValidatorCreated(validatorIdCounter, msg.sender);

        validatorIdCounter++;
    }

    /// Validator Token Locking and Unlocking ///
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

    // User Job Requesting
    function requestJob(
        uint8 nValidators,
        uint256 capacity
    ) external returns (uint256[] memory) {
        require(userIdByAddress[msg.sender] != 0, "User not registered.");
        require(capacity > 0, "Capacity must be greater zero.");
        // require(workerAddresses.length > 0)

        JobRequest storage request = jobRequests[jobCounter];
        request.id = jobCounter;
        request.creator = msg.sender;
        request.capacity = capacity;

        // Select validators pseudorandomly
        uint256[] memory selectedValidatorIds = _pseudorandomValidatorSelection(
            nValidators
        );

        address[] memory selectedValidators = new address[](
            selectedValidatorIds.length
        );

        // Change selected validators to the busy (active) state and grab their address
        for (uint256 i = 0; i < selectedValidatorIds.length; i++) {
            selectedValidators[i] = validators[selectedValidatorIds[i]]
                .validatorAddress;
        }

        request.validatorAddresses = selectedValidators; // Store the selected validators
        jobCounter++; // Increment jobCounter after storing the job request

        return selectedValidatorIds;
    }

    // Validator job creation voting
    function approveJobCreation(uint256 jobId) external {
        uint256 validatorId = validatorIdByAddress[msg.sender];
        Validator storage validator = validators[validatorId];
        require(validator.isActive, "Validator is not active.");
        require(
            jobRequests[jobId].id == jobId,
            "Job creation request does not exist."
        );

        JobRequest storage request = jobRequests[jobId];
        uint8 nValidators = uint8(request.validatorAddresses.length);
        request.signatures[msg.sender] = true;

        // Final signature triggers the deletion of the job request and the creation of the job
        if (_countSignatures(request) == nValidators) {
            Job storage job = jobs[jobId];
            job.id = jobId;
            job.owner = request.creator;
            job.capacity = request.capacity;
            job.validatorAddresses = request.validatorAddresses;
            job.completeCount = 0;
            delete jobRequests[jobId];
        }
    }

    // Validator job completion voting
    function completeJob(uint256 jobId) external {
        Job storage job = jobs[jobId];
        require(job.id == jobId, "Job does not exist.");
        require(job.isCompleted == false, "Job is already completed.");

        // Check if sender is one of the validators
        bool isValidator = false;
        for (uint i = 0; i < job.validatorAddresses.length; i++) {
            if (msg.sender == job.validatorAddresses[i]) {
                isValidator = true;
                break;
            }
        }

        require(
            isValidator,
            "Only validators of this job may call this function"
        );
        require(
            job.completeConfirmations[msg.sender] = false,
            "Validator already confirmed."
        );

        job.completeConfirmations[msg.sender] = true;
        job.completeCount++;

        // If all validators have confirmed status, mark job as complete
        if (job.completeCount == job.validatorAddresses.length) {
            job.isCompleted = true;

            // Move all validators to the inactive state
            for (uint i = 0; i < job.validatorAddresses.length; i++) {
                Validator storage validator = validators[
                    validatorIdByAddress[job.validatorAddresses[i]]
                ];
                _updateValidatorState(validator.id, false);
            }

            emit JobCompleted(jobId);
        }
    }

    function _countSignatures(
        JobRequest storage request
    ) internal view returns (uint256) {
        uint256 count = 0;
        for (uint256 i = 0; i < request.validatorAddresses.length; i++) {
            if (request.signatures[request.validatorAddresses[i]]) {
                count++;
            }
        }

        return count;
    }

    // Pseudorandom selection of validators
    function _pseudorandomValidatorSelection(
        uint8 nValidators
    ) internal returns (uint256[] memory) {
        require(
            nValidators >= minValidators && nValidators <= maxValidators,
            "nValidators must be between minValidator and maxValidator"
        );
        require(
            nValidators > 0 && nValidators <= validatorIdCounter,
            "Not enough available validators for job, please try again later."
        );

        uint256[] memory selectedValidators = new uint256[](nValidators);
        uint256 selectedCount = 0;

        for (uint256 i = 0; i < nValidators; i++) {
            uint256 nonce = 0;
            while (selectedCount == i) {
                uint256 randId = uint256(
                    keccak256(
                        abi.encodePacked(block.timestamp, msg.sender, nonce++)
                    )
                ) % validatorIdCounter;

                if (validatorStateById[randId] == 1) {
                    // Mark selected validator as busy
                    _updateValidatorState(randId, true);
                    selectedValidators[i] = randId;
                    selectedCount++;
                } else if (nonce < 50) {
                    nonce++;
                } else {
                    revert("Max validator requests reached");
                }
            }
        }

        return selectedValidators;
    }

    function _updateValidatorState(
        uint256 validatorId,
        bool activate
    ) internal {
        require(
            validatorId > 0 && validatorId < validatorIdCounter,
            "Invalid ValidatorId"
        );
        require(
            validatorStateById[validatorId] > 0,
            "Validator is not online."
        );

        if (activate) {
            // Move validator to the active state if not already active
            if (validatorStateById[validatorId] == 1) {
                validatorStateById[validatorId] = 2;
            } else {
                revert("Validator being activated is already active!");
            }
        } else {
            // Move validator to the inactive state if not already inactive
            if (validatorStateById[validatorId] == 2) {
                validatorStateById[validatorId] = 1;
            } else {
                revert("Validator is already inactive!");
            }
        }
    }

    function _activateValidator(uint256 validatorId) internal {
        require(
            0 < validatorId && validatorId < validatorIdCounter,
            "Validator ID must be valid."
        );

        Validator storage validator = validators[validatorId];
        require(!validator.isActive, "Validator already active!");
        validator.isActive = true;
        validatorStateById[validatorId] = 1;
        emit ValidatorActivated(validatorId);
    }

    function _deactivateValidator(uint256 validatorId) internal {
        require(
            0 < validatorId && validatorId < validatorIdCounter,
            "Validator ID must be valid."
        );
        Validator storage validator = validators[validatorId];

        require(validator.isActive, "Validator not active!");
        validator.isActive = false;

        validatorStateById[validatorId] = 0;
        emit ValidatorActivated(validatorId);
    }

    function random() external view returns (uint256) {
        return uint256(keccak256(abi.encodePacked((block.timestamp))));
    }

    function getjobCounter() external view returns (uint256) {
        return jobCounter;
    }

    function getValidatorIdCounter() external view returns (uint256) {
        return validatorIdCounter;
    }

    function getUserIdCounter() external view returns (uint256) {
        return userIdCounter;
    }

    function getUserId(
        string memory publicKeyHash
    ) external view returns (uint256) {
        return userIdByHash[publicKeyHash];
    }

    function getValidatorState(
        uint256 validatorId
    ) external view returns (uint256) {
        require(
            validatorId > 0 && validatorId < validatorIdCounter,
            "Invalid ValidatorId!"
        );
        return validatorStateById[validatorId];
    }

    // Returns a list of all the selected validators for a job request
    function getJobRequestValidators(
        uint256 reqId
    ) external view returns (address[] memory) {
        require(reqId < jobCounter, "Invalid jobRequest ID");
        JobRequest storage jobReq = jobRequests[reqId];
        if (jobReq.id == reqId) {
            address[] memory jobReqAddresses = jobRequests[reqId]
                .validatorAddresses;
            return jobReqAddresses;
        } else {
            revert("JobRequest not found!");
        }
    }
}
