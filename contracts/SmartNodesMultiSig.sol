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

    // Counters for storage indexing / IDs
    uint256 public nextProposalId;
    uint256 public requiredApprovals;

    address[] public validators;

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

    mapping(address => bool) public isValidator; // For quick validator check
    mapping(uint256 => Proposal) public proposals;
    mapping(uint256 => mapping(address => bool)) public approvals;

    event ProposalCreated(uint256 proposalId, bytes data);
    event Voted(uint256 proposalId, address validator);
    event ProposalExecuted(uint256 proposalId);
    event Deposit(address indexed sender, uint amount);
    event ValidatorAdded(address validator);
    event ValidatorRemoved(address validator);

    modifier onlyValidator() {
        require(
            isValidator[msg.sender],
            "Caller is not a Smart Nodes Validator!"
        );
        _;
    }

    function initialize(
        address target // Address of the main contract (Smart Nodes)
    ) public initializer {
        smartnodesContractAddress = target;
        _smartnodesContractInstance = ISmartnodesCore(target);
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
        uint256 totalCapacity = 0;
        uint256[] memory allCapacities;
        address[] memory allWorkers;

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

                uint256 newLength = capacities.length + allCapacities.length;
                uint256[] memory newCapacities = new uint256[](newLength);
                address[] memory newWorkers = new address[](newLength);

                for (uint256 j = 0; j < capacities.length; j++) {
                    totalCapacity += capacities[j];
                }

                allCapacities = 

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
            IERC20Upgradeable(
                _smartnodesContractInstance.transfer(allWorkers[k], reward)
            );
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
            _checkValidator(validator),
            "Validator must be registered on SmartnodesCore!"
        );
        require(
            !isValidator[validator],
            "Validator already registered on MultSig!"
        );

        isValidator[validator] = true;
        validators.push(validator);
        _updateApprovalRequirements();

        emit ValidatorAdded(validator);
    }

    function removeValidator(address validator) external onlyValidator {
        require(isValidator[validator], "Validator not registered!");
        isValidator[validator] = false;
        for (uint256 i = 0; i < validators.length; i++) {
            if (validators[i] == validator) {
                validators[i] = validators[validators.length - 1];
                validators.pop();
                break;
            }
        }
        _updateApprovalRequirements();
        emit ValidatorRemoved(validator);
    }

    /**
     * @notice Checks if a given address is a valid validator on the parent contract
     * @param validator The address to check
     * @return True if the address is a valid validator, false otherwise
     */
    function _checkValidator(address validator) internal view returns (bool) {
        require(validator != address(0), "Invalid address.");
        require(!isValidator[validator], "Validator already registered.");

        // Perform staticcall with the new data (get validator)
        bool isVal = _smartnodesContractInstance.isLocked(validator);
        return isVal;
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
}
