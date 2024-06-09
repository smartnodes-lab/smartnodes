// SPDX-License-Identifier: MIT
pragma solidity ^0.8.5;

import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/contracts/interfaces/IERC20Upgradeable.sol";
import "./interfaces/ISmartnodesCore.sol";

/** 
    * @title SmartnodesMultiSig
    * @dev A multi-signature contract composed of Smartnodes validators responsible for
     managing the Core contract
*/
contract SmartnodesMultiSig is Initializable {
    // State update constraints
    uint256 public lastProposal = 0; // time of last proposal
    uint256 public constant UPDATE_TIME = 600; // seconds required between state updates
    uint256 public requiredApprovalsPercentage = 66;

    uint256 public maxStateUpdates;

    // Counters for storage indexing / IDs
    uint256 public nextProposalId;
    uint256 public requiredApprovals;

    // Metadata and bytecode for SmartNodes calls
    ISmartnodesCore private _smartnodesContractInstance;
    address public smartnodesContractAddress;

    enum FunctionType {
        UpdateValidator,
        ConfirmValidator,
        CompleteJob,
        DisputeJob
    }

    // Proposal for a Smartnodes Update
    struct Proposal {
        uint256 id;
        FunctionType[] functionTypes;
        bytes[] data;
        bool executed;
        uint256 approvals;
    }

    struct ValidatorTokens {
        uint256 locked;
        bool enough;
    }

    address[] public validators;
    mapping(address => bool) public isValidator; // For quick validator check
    mapping(address => ValidatorTokens) public lockedTokens;
    mapping(uint256 => Proposal) public proposals;
    mapping(uint256 => mapping(address => bool)) public approvals;

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

    function initialize(
        address target // Address of the main contract (Smart Nodes)
    ) public initializer {
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
        FunctionType[] calldata _functionTypes,
        bytes[] calldata _data
    ) external onlyValidator {
        require(
            block.timestamp - lastProposal > UPDATE_TIME,
            "Proposals must be submitted after UPDATE_TIME since last approved proposal!"
        );
        require(
            _functionTypes.length == _data.length,
            "Function types and data length must match!"
        );

        proposals[nextProposalId] = Proposal({
            id: nextProposalId,
            functionTypes: _functionTypes,
            data: _data,
            executed: false,
            approvals: 0
        });

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

        approvals[_proposalId][msg.sender] = true;
        proposals[_proposalId].approvals++;

        if (proposals[_proposalId].approvals >= requiredApprovals) {
            _executeTransaction(_proposalId);
        }

        emit Voted(_proposalId, msg.sender);
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

        // get total number of workers and capacities
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
            // if (proposal.functionTypes[i] == FunctionType.UpdateValidator) {
            //     address validator = abi.decode(proposal.data[i], (address));
            //     _smartnodesContractInstance.updateValidator(validator);
            if (proposal.functionTypes[i] == FunctionType.CompleteJob) {
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
            } else if (proposal.functionTypes[i] == FunctionType.DisputeJob) {
                uint256 jobId = abi.decode(proposal.data[i], (uint256));
                _smartnodesContractInstance.disputeJob(jobId);
            }
            // else if (proposal.functionTypes[i] == FunctionType.UpdateWorkers) {
            //     address[] memory workers = abi.decode(proposal.data[i], (address[]));
            //     _updateWorkers(workers);
            // }
        }

        // Distribute rewards for workers
        uint256 emissionRate = _smartnodesContractInstance.getEmissionRate();
        for (uint256 k = 0; k < allWorkers.length; k++) {
            uint256 reward = ((allCapacities[k] * emissionRate) /
                totalCapacity);
            _smartnodesContractInstance.mintTokens(allWorkers[k], reward);
            emit RewardDistributed(allWorkers[k], emissionRate);
        }

        proposal.executed = true;
        lastProposal = block.timestamp;

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
            (validators.length * requiredApprovalsPercentage) /
            100;

        if (requiredApprovals == 0) {
            requiredApprovals = 1; // Ensure at least 1 approval is required
        }
    }

    /**
     * @notice Adds a new validator to the contract
     * @param validator The address of the new validator
     */
    function addValidator(address validator) public {
        require(
            lockedTokens[msg.sender].enough,
            "Validator must be registered and locked on SmartnodesCore!"
        );
        require(
            !isValidator[validator],
            "Validator already registered on MultSig!"
        );

        validators.push(msg.sender);
        isValidator[validator] = true;
        _updateApprovalRequirements();

        emit ValidatorAdded(validator);
    }

    function removeValidator(
        address validator
    ) external onlyValidator onlySmartnodes {
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

    // Pseudorandom selection of validators TODO Swap to VRF implementation?
    function generateValidatorCandidates()
        external
        view
        returns (address[] memory)
    {
        uint8 nValidators = 1;

        address[] memory selectedValidators = new address[](nValidators);
        uint256 selectedCount = 0;

        for (uint256 i = 0; i < nValidators; i++) {
            uint256 randId = uint256(
                keccak256(abi.encodePacked(block.timestamp, msg.sender, i))
            ) % validators.length;

            selectedValidators[i] = validators[randId];
            selectedCount++;
        }

        return selectedValidators;
    }

    function updateLockedTokens(
        address _validator,
        uint256 locked,
        bool enough
    ) external onlySmartnodes {
        lockedTokens[_validator].locked = locked;
        lockedTokens[_validator].enough = enough;
    }
}
