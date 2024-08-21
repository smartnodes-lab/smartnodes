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
contract SmartnodesMultiSig is Initializable, VRFConsumerBaseV2Upgradeable {
    enum FunctionType {
        DeactivateValidator,
        CompleteJob
        // DisputeJob,
        // CreateJob
    }

    // Proposal for a Smartnodes Update
    struct Proposal {
        uint256 id;
        FunctionType[] functionTypes;
        bytes[] data;
        bool executed;
        uint256 _approvals;
    }

    struct ValidatorTokens {
        uint256 locked;
        bool enough;
    }

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

    // State update constraints
    uint256 public lastProposal; // time of last proposal
    uint256 public constant UPDATE_TIME = 0; // 600; seconds required between state updates
    uint256 public requiredApprovalsPercentage;
    uint256 public maxStateUpdates;

    // Counters for storage indexing / IDs
    uint256 public nextProposalId;
    uint256 public requiredApprovals;

    // Metadata and bytecode for SmartNodes calls
    ISmartnodesCore private _smartnodesContractInstance;
    address public smartnodesContractAddress;

    address[] public validators;
    address[] public currentRoundValidators;
    uint256 public randomRequestId;

    mapping(address => bool) public isValidator; // For quick validator checks
    mapping(address => ValidatorTokens) public lockedTokens;
    mapping(uint256 => Proposal) public proposals;
    mapping(uint256 => mapping(address => bool)) public approvals;
    mapping(uint256 => mapping(uint256 => address)) public approvedValidators;

    event ProposalCreated(uint256 proposalId, bytes data);
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

    modifier onlySmartnodes() {
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
        _;
    }

    function initialize(
        address target, // Address of the main contract (Smart Nodes)
        address _validatorContract,
        address _vrfCoordinator,
        uint64 _subscriptionId
    ) public initializer {
        __VRFConsumerBaseV2_init(_vrfCoordinator);
        COORDINATOR = VRFCoordinatorV2Interface(_vrfCoordinator);

        smartnodesContractAddress = target;
        maxStateUpdates = 20;
        _smartnodesContractInstance = ISmartnodesCore(target);
        lastProposal = 0; // time of last proposal
        requiredApprovalsPercentage = 66;
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
            block.timestamp - lastProposal >= UPDATE_TIME,
            "Proposals must be submitted after UPDATE_TIME since last approved proposal!"
        );
        require(
            _functionTypes.length == _data.length,
            "Function types and data length must match!"
        );

        Proposal storage proposal = proposals[nextProposalId];
        proposal.id = nextProposalId;
        proposal.functionTypes = _functionTypes;
        proposal.data = _data;

        emit ProposalCreated(nextProposalId, abi.encode(_functionTypes, _data));

        nextProposalId++;
    }

    /**
     * @notice Casts a vote for a proposal and executes once required approvals are met. Add Validator to storage
      if it has just registered and is not stored on MultiSig. 
     * @param _proposalId The ID of the proposal
     */
    function approveTransaction(uint256 _proposalId) external {
        require(
            !approvals[_proposalId][msg.sender],
            "Validator already voted on this transaction!"
        );

        if (isValidator[msg.sender] == false) {
            addValidator(msg.sender);
        }

        Proposal storage proposal = proposals[_proposalId];
        approvedValidators[_proposalId][proposal._approvals] = msg.sender;
        proposal._approvals++;

        approvals[_proposalId][msg.sender] = true;
        emit Voted(_proposalId, msg.sender);

        if (proposals[_proposalId]._approvals >= requiredApprovals) {
            _executeTransaction(_proposalId);
        }
    }

    /**
     * @notice Adds a new validator to the contract
     * @param validator The address of the new validator
     */
    function addValidator(address validator) public {
        require(
            lockedTokens[validator].enough,
            "Validator must be registered and locked on SmartnodesCore!"
        );
        require(
            !isValidator[validator],
            "Validator already registered on MultSig!"
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
     * @notice Executes a proposal if it has enough approvals. Only to be called by approveTransaction
     * @param _proposalId The ID of the proposal to be executed
     */
    function _executeTransaction(uint256 _proposalId) internal onlyValidator {
        Proposal storage proposal = proposals[_proposalId];
        require(!proposal.executed, "Proposal already executed.");
        require(
            proposal.functionTypes.length <= maxStateUpdates,
            "Must not exceed max state updates!"
        );
        uint256 totalWorkers = 0;

        // Get total number of participant workers and their capacities
        for (uint i = 0; i < proposal.functionTypes.length; i++) {
            if (proposal.functionTypes[i] == FunctionType.CompleteJob) {
                (uint256 jobId, address[] memory workers) = abi.decode(
                    proposal.data[i],
                    (uint256, address[])
                );
                totalWorkers += workers.length;
            }
        }

        // Get proportional capacities for each worker
        uint256[] memory allCapacities = new uint256[](totalWorkers);
        address[] memory allWorkers = new address[](totalWorkers);
        uint256 totalCapacity = 0;
        uint256 allWorkerInd = 0;

        for (uint i = 0; i < proposal.functionTypes.length; i++) {
            // Update connected validator stats
            if (proposal.functionTypes[i] == FunctionType.DeactivateValidator) {
                address validator = abi.decode(proposal.data[i], (address));
                _removeValidator(validator);

                // Update job completions
            } else if (proposal.functionTypes[i] == FunctionType.CompleteJob) {
                (uint256 jobId, address[] memory workers) = abi.decode(
                    proposal.data[i],
                    (uint256, address[])
                );

                uint256[] memory capacities = _smartnodesContractInstance
                    .completeJob(jobId, workers);

                for (uint256 j = 0; j < capacities.length; j++) {
                    totalCapacity += capacities[j];
                    allCapacities[allWorkerInd] = capacities[j];
                    allWorkers[allWorkerInd] = workers[j];
                    allWorkerInd++;
                }

                // } else if (proposal.functionTypes[i] == FunctionType.DisputeJob) {
                //     uint256 jobId = abi.decode(proposal.data[i], (uint256));
                //     _smartnodesContractInstance.disputeJob(jobId);
                // } else if (proposal.functionTypes[i] == FunctionType.CreateJob) {
                //     (bytes32 userHash, uint256[] memory _capacities) = abi.decode(
                //         proposal.data[i],
                //         (bytes32, uint256[])
                //     );
                //     _smartnodesContractInstance.requestJob(userHash, _capacities);
            }
        }

        address[] memory _approvedValidators = new address[](
            proposal._approvals
        );

        for (uint i = 0; i < proposal._approvals; i++) {
            _approvedValidators[i] = approvedValidators[_proposalId][i];
        }

        _smartnodesContractInstance.mintTokens(
            allWorkers,
            allCapacities,
            totalCapacity,
            _approvedValidators
        );

        proposal.executed = true;
        lastProposal = block.timestamp;
        randomRequestId = _requestRandomness();
        emit ProposalExecuted(_proposalId);
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

    function updateLockedTokens(
        address _validator,
        uint256 locked,
        bool enough
    ) external onlySmartnodes {
        lockedTokens[_validator].locked = locked;
        lockedTokens[_validator].enough = enough;
    }

    function _requestRandomness() internal returns (uint256 requestId) {
        require(validators.length > 0, "No validators available");

        // Request random words from Chainlink VRF
        requestId = COORDINATOR.requestRandomWords(
            s_keyHash,
            s_subscriptionId,
            requestConfirmations,
            callbackGasLimit,
            numWords
        );

        return requestId;
    }

    // Random selection of validators for next proposals once a random number is received
    function fulfillRandomWords(
        uint256 requestId,
        uint256[] memory randomWords
    ) internal override {
        require(validators.length > 0, "No validators available");
        require(
            msg.sender == vrfCoordinator,
            "Only VRF coordinator can fulfil this request!"
        );
        require(randomWords.length > 0, "No random words provided");
        require(requestId == randomRequestId, "Invalid request!");

        uint256 randomValue = randomWords[0];
        uint8 nValidators = 3; // TODO dynamic or static gloabal nValidator parameter

        address[] memory selectedValidators = new address[](nValidators);

        for (uint8 i = 0; i < nValidators; i++) {
            uint256 randomIndex = randomValue % validators.length;
            selectedValidators[i] = validators[randomIndex];

            // Re-select if already picked
            randomValue = uint256(keccak256(abi.encodePacked(randomValue, i)));
        }

        currentRoundValidators = selectedValidators;
    }

    function getProposalData(
        uint256 _proposalId
    ) external view returns (Proposal memory) {
        return proposals[_proposalId];
    }

    function isActiveValidator(
        address _validatorAddress
    ) external view returns (bool) {
        return isValidator[_validatorAddress];
    }

    function getNumValidators() external view returns (uint256) {
        return validators.length;
    }

    function getSelectedValidators() external view returns (address[] memory) {
        return currentRoundValidators;
    }
}
