// SPDX-License-Identifier: MIT
pragma solidity ^0.8.5;

import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/contracts/interfaces/IERC20Upgradeable.sol";
import "./interfaces/ISmartnodesCore.sol";
import "@chainlink/contracts/src/v0.8/vrf/dev/VRFConsumerBaseV2Upgradeable.sol";
import "@chainlink/contracts/src/v0.8/interfaces/VRFCoordinatorV2Interface.sol";
import {VRFV2PlusClient} from "@chainlink/contracts/src/v0.8/vrf/dev/libraries/VRFV2PlusClient.sol";

/** 
    * @title SmartnodesMultiSig
    * @dev A multi-signature contract composed of Smartnodes validators responsible for
     managing the Core contract
*/
contract SmartnodesMultiSig is Initializable {
    enum FunctionType {
        DeactivateValidator,
        CreateJob,
        CompleteJob
        // DisputeJob
    }

    // Proposal for a Smartnodes Update
    struct Proposal {
        uint256 id;
        FunctionType[] functionTypes;
        bytes[] data;
        bool executed;
    }

    // Chainlink VRF Parameters
    // uint64 s_subscriptionId;
    // address linkAddress = 0x779877A7B0D9E8603169DdbD7836e478b4624789;
    // bytes32 s_keyHash =
    //     0x787d74caea10b2b357790d5b5247c2f63d1d91572a9846f780606e4d953677ae;
    // uint32 callbackGasLimit = 100000;
    // uint16 requestConfirmations = 2;
    // uint32 numWords = 1;
    // address vrfCoordinator;
    // VRFCoordinatorV2Interface COORDINATOR;

    // State update constraints
    uint256 public constant UPDATE_TIME = 300; // 5 minutes minimum required between state updates
    uint256 public requiredApprovalsPercentage;
    uint256 public requiredApprovals;
    uint256 public maxStateUpdates; // Maximum number of function calls per proposal
    uint256 public lastProposalTime; // time of last proposal

    // Metadata and bytecode for SmartNodes calls
    ISmartnodesCore private _smartnodesContractInstance;
    address public smartnodesContractAddress;

    // Counters for storage indexing / IDs
    uint256 public randomRequestId;
    uint256 public nextProposalId;
    uint8 public nValidators;

    address[] public validators;
    address[] public currentRoundValidators;
    Proposal[] public currentProposals;

    mapping(address => bool) public isValidator; // For quick validator checks
    mapping(uint256 => Proposal) public proposals;
    mapping(address => bool) public hasSubmittedProposal;
    mapping(address => mapping(uint8 => bool)) public votes;
    mapping(uint8 => uint256) public numVotes;

    event ProposalCreated(
        uint256 proposalId,
        uint8 proposalNum,
        bytes32 validatorId
    );
    event Voted(uint256 proposalId, address validator);
    event ProposalExecuted(uint256 proposalId);
    event Deposit(address indexed sender, uint amount);
    event ValidatorAdded(address validator);
    event ValidatorRemoved(address validator);
    event RewardDistributed(address reciever, uint256 amount);

    modifier onlyValidator() {
        require(
            isValidator[msg.sender],
            "Caller is not a Smart Nodes Validator!"
        );
        _;
    }

    modifier onlySmartnodesCore() {
        require(
            msg.sender == smartnodesContractAddress,
            "Caller must be the SmartnodesMultiSig."
        );
        _;
    }

    modifier onlySelectedValidator() {
        require(
            _isCurrentRoundValidator(msg.sender) ||
                currentRoundValidators.length == 0,
            "Caller is not a selected validator for this round!"
        );
        require(
            !hasSubmittedProposal[msg.sender],
            "Validator has already submitted a proposal this round!"
        );
        _;
    }

    function initialize(
        address target // Address of the main contract (Smart Nodes)
    )
        public
        // address _vrfCoordinator,
        // uint64 _subscriptionId
        initializer
    {
        // __VRFConsumerBaseV2_init(_vrfCoordinator);
        // COORDINATOR = VRFCoordinatorV2Interface(_vrfCoordinator);

        smartnodesContractAddress = target;
        maxStateUpdates = 30;
        _smartnodesContractInstance = ISmartnodesCore(target);
        lastProposalTime = 0; // time of last proposal
        requiredApprovalsPercentage = 65;
        nValidators = 1;
        // s_subscriptionId = _subscriptionId;
    }

    receive() external payable {
        emit Deposit(msg.sender, msg.value);
    }

    /**
     * @notice Creates a new proposal, to update all the essential data structures given some aggregated off-chain state.
     * @param _functionTypes The types of functions to be called in the proposal
     * @param _data The call data for the proposal
     */
    function createProposal(
        FunctionType[] memory _functionTypes,
        bytes[] memory _data
    ) external onlySelectedValidator {
        require(
            block.timestamp - lastProposalTime >= UPDATE_TIME,
            "Proposals must be submitted 0-10 mins after since last executed proposal!"
        );

        require(
            _functionTypes.length == _data.length,
            "Function types and data length must match!"
        );

        Proposal memory proposal = Proposal({
            id: nextProposalId,
            functionTypes: _functionTypes,
            data: _data,
            executed: false
        });

        currentProposals.push(proposal);
        uint8 proposalNum = uint8(currentProposals.length) - 1;
        hasSubmittedProposal[msg.sender] = true;

        bytes32 validatorId = _smartnodesContractInstance.getValidatorBytes(
            msg.sender
        );

        emit ProposalCreated(nextProposalId, proposalNum, validatorId);
    }

    /**
     * @notice Casts a vote for a proposal and executes once required approvals are met. Add Validator to storage
      if it has just registered and is not stored on MultiSig. 
     * @param proposalNum The ID of the current round proposal
     */
    function approveTransaction(uint8 proposalNum) external onlyValidator {
        require(
            !votes[msg.sender][proposalNum],
            "Validator has already voted!"
        );
        require(
            currentProposals.length >= proposalNum,
            "Invalid proposal number!"
        );

        if (isValidator[msg.sender] == false) {
            addValidator(msg.sender);
        }

        votes[msg.sender][proposalNum] = true;
        numVotes[proposalNum]++;

        emit Voted(proposalNum, msg.sender);

        if (numVotes[proposalNum] >= requiredApprovals) {
            // proposals[nextProposalId] = proposal;
            _executeTransaction(proposalNum);
        }
    }

    /**
     * @notice Executes a proposal if it has enough approvals. Only to be called by approveTransaction
     * @param proposalNum The ID of the proposal to be executed
     */
    function _executeTransaction(uint8 proposalNum) internal onlyValidator {
        // Load Proposal
        Proposal memory proposal = currentProposals[proposalNum];

        require(!proposal.executed, "Proposal already executed.");
        require(
            proposal.functionTypes.length <= maxStateUpdates,
            "Must not exceed max state updates!"
        );

        // Get total number of participant workers and their capacities
        uint256 totalWorkers = 0;

        for (uint i = 0; i < proposal.functionTypes.length; i++) {
            if (proposal.functionTypes[i] == FunctionType.CompleteJob) {
                (uint256 jobId, address[] memory workers) = abi.decode(
                    proposal.data[i],
                    (uint256, address[])
                );
                totalWorkers += workers.length;
            }
        }

        // Vars for calculating proportional capacities for each worker
        uint256[] memory allCapacities = new uint256[](totalWorkers);
        address[] memory allWorkers = new address[](totalWorkers);
        uint256 totalCapacity = 0;
        uint256 allWorkerInd = 0;

        // Handle each state update function call
        for (uint i = 0; i < proposal.functionTypes.length; i++) {
            // Update connected validator stats
            if (proposal.functionTypes[i] == FunctionType.DeactivateValidator) {
                address validator = abi.decode(proposal.data[i], (address));
                _removeValidator(validator);

                // Create new jobs
            } else if (proposal.functionTypes[i] == FunctionType.CreateJob) {
                (
                    bytes32 userHash,
                    bytes32 jobHash,
                    uint256[] memory _capacities
                ) = abi.decode(proposal.data[i], (bytes32, bytes32, uint256[]));
                _smartnodesContractInstance.requestJob(
                    userHash,
                    jobHash,
                    _capacities
                );

                // Update completed existing jobs
            } else if (proposal.functionTypes[i] == FunctionType.CompleteJob) {
                (bytes32 jobId, address[] memory workers) = abi.decode(
                    proposal.data[i],
                    (bytes32, address[])
                );

                uint256[] memory capacities = _smartnodesContractInstance
                    .completeJob(jobId, workers);

                for (uint256 j = 0; j < capacities.length; j++) {
                    totalCapacity += capacities[j];
                    allCapacities[allWorkerInd] = capacities[j];
                    allWorkers[allWorkerInd] = workers[j];
                    allWorkerInd++;
                }
            }
            // } else if (proposal.functionTypes[i] == FunctionType.DisputeJob) {
            //     uint256 jobId = abi.decode(proposal.data[i], (uint256));
            //     _smartnodesContractInstance.disputeJob(jobId);
        }

        // Gather validators who voted on the transaction
        address[] memory _approvedValidators = new address[](
            numVotes[proposalNum]
        );

        for (uint i = 0; i < validators.length; i++) {
            address validator = validators[i];

            if (votes[validator][proposalNum]) {
                _approvedValidators[i] = validator;
            }
        }

        // Call mint function to generate rewards for workers and validators
        _smartnodesContractInstance.mintTokens(
            allWorkers,
            allCapacities,
            totalCapacity,
            _approvedValidators
        );

        // Clean up old proposals, 1000 length window allows 20 days for proposals to be disputed, delete unused proposals
        if (nextProposalId > 1000) {
            delete proposals[nextProposalId - 1000];
        }

        proposal.executed = true;
        emit ProposalExecuted(proposal.id);
        proposals[nextProposalId] = proposal;
        _resetCurrentValidators();
        _updateRound();
    }

    /**
     * @notice Adds a new validator to the contract, must be staked on SmartnodesCore.
     * @param validator The address of the new validator
     */
    function addValidator(address validator) public {
        require(
            _checkLockedTokens(validator),
            "Validator must be registered and locked on SmartnodesCore!"
        );
        require(
            !isValidator[validator],
            "Validator already registered on Multsig!"
        );

        validators.push(validator);
        isValidator[validator] = true;
        _updateApprovalRequirements();

        emit ValidatorAdded(validator);
    }

    function removeValidator(address validator) external onlyValidator {
        require(msg.sender == validator, "Cannot remove another validator!");
        _removeValidator(validator);
    }

    /**
     * @notice Rewards validators who voted for the majority on a proposal
     * @param _proposalId The ID of the proposal
     */
    function _rewardValidators(uint256 _proposalId) internal {}

    /**
     * @dev Update the number of required approvals (66% of the active validators)
     */
    function _updateApprovalRequirements() internal {
        requiredApprovals =
            (validators.length * requiredApprovalsPercentage + 80) /
            100;

        if (requiredApprovals == 0) {
            requiredApprovals = 1; // Ensure at least 1 approval is required
        }
    }

    function _removeValidator(address validator) private {
        require(isValidator[validator], "Validator not registered!");
        isValidator[validator] = false;

        for (uint i = 0; i < validators.length; i++) {
            if (validators[i] == validator) {
                validators[i] = validators[validators.length - 1];
                validators.pop();
                break;
            }
        }

        _updateApprovalRequirements();
        emit ValidatorRemoved(validator);
    }

    function _isCurrentRoundValidator(
        address _validator
    ) internal view returns (bool) {
        for (uint i = 0; i < currentRoundValidators.length; i++) {
            if (currentRoundValidators[i] == _validator) {
                return true;
            }
        }
        return false;
    }

    function _updateRound() internal {
        for (uint8 i = 0; i < currentProposals.length; i++) {
            numVotes[i] = 0;
        }
        for (uint i = 0; i < validators.length; i++) {
            address validator = validators[i];
            for (uint8 n = 0; n < currentProposals.length; n++) {
                votes[validator][n] = false;
            }
        }

        while (currentProposals.length > 0) {
            currentProposals.pop();
        }

        lastProposalTime = block.timestamp;
        // randomRequestId = _requestRandomness();
        nextProposalId++;
    }

    function _resetCurrentValidators() internal {
        // Clear submission status and other parameters
        for (uint256 i = 0; i < currentRoundValidators.length; i++) {
            hasSubmittedProposal[currentRoundValidators[i]] = false;
        }

        // If it's the genesis proposal and no round validators exist
        if (currentRoundValidators.length == 0) {
            for (uint256 i = 0; i < validators.length; i++) {
                hasSubmittedProposal[validators[i]] = false;
            }
        }

        delete currentRoundValidators;

        require(
            validators.length >= nValidators,
            "Not enough active validators!"
        );

        // Create a temporary array to store selected validators
        address[] memory selectedValidators = new address[](nValidators);
        uint256 selectedCount = 0;
        uint nonce = 0;

        while (selectedCount < nValidators) {
            uint256 randId = uint256(
                keccak256(
                    abi.encode(
                        block.timestamp,
                        msg.sender,
                        selectedCount,
                        nonce
                    )
                )
            ) % validators.length;

            address selectedValidator = validators[randId];

            // Check if the validator is already selected
            bool alreadySelected = false;
            for (uint256 j = 0; j < selectedCount; j++) {
                if (selectedValidators[j] == selectedValidator) {
                    alreadySelected = true;
                    break;
                }
            }

            // If not selected, add to the current round and increment counter
            if (!alreadySelected) {
                selectedValidators[selectedCount] = selectedValidator;
                currentRoundValidators.push(selectedValidator);
                selectedCount++;
            }

            nonce++;
        }
    }

    function generateValidators(
        uint256 numValidators
    ) external view returns (address[] memory) {
        require(
            validators.length > numValidators,
            "Not enough active validators!"
        );

        address[] memory selectedValidators = new address[](numValidators);
        uint256 selectedCount = 0;

        for (uint256 i = 0; i < numValidators; i++) {
            uint256 randId = uint256(
                keccak256(abi.encode(block.timestamp, msg.sender, i))
            ) % validators.length;

            selectedValidators[i] = validators[randId];
            selectedCount++;
        }

        return selectedValidators;
    }

    // function _requestRandomness() internal returns (uint256 requestId) {
    //     require(validators.length > 0, "No validators available");

    //     // Request random words from Chainlink VRF
    //     requestId = COORDINATOR.requestRandomWords(
    //         s_keyHash,
    //         s_subscriptionId,
    //         requestConfirmations,
    //         callbackGasLimit,
    //         numWords
    //     );

    //     return requestId;
    // }

    // // Random selection of validators for next proposals once a random number is received
    // function fulfillRandomWords(
    //     uint256 requestId,
    //     uint256[] memory randomWords
    // ) internal override {
    //     require(validators.length > 0, "No validators available");
    //     require(
    //         msg.sender == vrfCoordinator,
    //         "Only VRF coordinator can fulfil this request!"
    //     );
    //     require(randomWords.length > 0, "No random words provided");
    //     require(requestId == randomRequestId, "Invalid request!");

    //     uint256 randomValue = randomWords[0];
    //     uint8 nValidators = 3; // TODO dynamic or static gloabal nValidator parameter

    //     address[] memory selectedValidators = new address[](nValidators);

    //     for (uint8 i = 0; i < nValidators; i++) {
    //         uint256 randomIndex = randomValue % validators.length;
    //         selectedValidators[i] = validators[randomIndex];

    //         // Re-select if already picked
    //         randomValue = uint256(keccak256(abi.encodePacked(randomValue, i)));
    //     }

    //     currentRoundValidators = selectedValidators;
    // }

    function isActiveValidator(
        address _validatorAddress
    ) external view returns (bool) {
        return isValidator[_validatorAddress];
    }

    function _checkLockedTokens(
        address validatorAddress
    ) internal view returns (bool) {
        return _smartnodesContractInstance.isLocked(validatorAddress);
    }

    function getNumValidators() external view returns (uint256) {
        return validators.length;
    }

    function getSelectedValidators() external view returns (address[] memory) {
        return currentRoundValidators;
    }

    // Get detailed info on a current proposal
    function getProposalData(
        uint8 _proposalId
    ) external view returns (Proposal memory) {
        return proposals[_proposalId];
    }

    function getCurrentProposal(
        uint8 proposalNum
    ) external view returns (uint[] memory, bytes[] memory) {
        require(proposalNum < currentProposals.length, "Proposal not found!");
        Proposal memory proposal = currentProposals[proposalNum];

        uint[] memory functionTypeAsUint = new uint[](
            proposal.functionTypes.length
        );

        for (uint i = 0; i < proposal.functionTypes.length; i++) {
            functionTypeAsUint[i] = uint(proposal.functionTypes[i]);
        }

        return (functionTypeAsUint, proposal.data);
    }

    // Get basic info on the current state of the validator multisig
    function getState()
        external
        view
        returns (uint256, uint256, uint256, address[] memory)
    {
        return (
            lastProposalTime,
            nextProposalId,
            validators.length,
            currentRoundValidators
        );
    }

    // function getContractParams() external view returns () {}
}
