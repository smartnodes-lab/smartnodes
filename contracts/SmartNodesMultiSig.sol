// SPDX-License-Identifier: MIT
pragma solidity ^0.8.5;

/** 
    * @title SmartNodesMultiSig
    * @dev A multi-signature contract composed of Smart Nodes validators responsible for
     managing the Smart Nodes contract
*/
contract SmartNodesMultiSig {
    enum FunctionType {
        ConfirmValidator,
        UpdateValidator,
        UpdateJob,
        UpdateWorkers
    }

    // Proposal for a Smart Nodes Update
    struct Proposal {
        uint256 id;
        FunctionType[] functionTypes;
        bytes[] data;
        bool executed;
        uint256 approvals;
    }

    // State update constraints
    uint256 public lastProposal = 0; // time of last proposal
    uint256 public constant UPDATE_TIME = 600; // seconds required between state updates
    uint256 public requiredApprovalsPercentage = 66;

    address[] public validators;
    uint256 public nextProposalId;
    uint256 public requiredApprovals;

    // Metadata and bytecode for SmartNodes calls
    address public parentContract;

    mapping(address => bool) public isValidator; // For quick validator check
    mapping(uint256 => Proposal) public proposals;
    mapping(uint256 => mapping(address => bool)) public approvals;

    event ValidatorAdded(address validator);
    event ValidatorRemoved(address validator);
    event ProposalCreated(
        uint256 proposalId,
        address targetContract,
        bytes data
    );
    event Voted(uint256 proposalId, address validator);
    event ProposalExecuted(uint256 proposalId);
    event Deposit(address indexed sender, uint amount);

    modifier onlyValidator() {
        require(
            isValidator[msg.sender],
            "Caller is not a Smart Nodes Validator!"
        );
        _;
    }

    constructor(
        address target // Address of the main contract (Smart Nodes)
    ) {
        parentContract = target;
    }

    receive() external payable {
        emit Deposit(msg.sender, msg.value);
    }

    /**
     * @notice Creates a new proposal
     * @param _functionTypes The types of functions to be called in the proposal
     * @param _data The call data for the proposal
     */
    function createTransaction(
        FunctionType[] calldata _functionTypes,
        bytes[] calldata _data
    ) external onlyValidator {
        require(
            block.timestamp - lastProposal > UPDATE_TIME,
            "Proposals must be submitted after UPDATE_TIME since last approved proposal."
        );

        proposals[nextProposalId] = Proposal({
            id: nextProposalId,
            functionTypes: _functionTypes,
            data: _data,
            executed: false,
            approvals: 0
        });

        emit ProposalCreated(
            nextProposalId,
            parentContract,
            abi.encode(_functionTypes, _data)
        );
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
            _addValidator(msg.sender);
        }

        approvals[_proposalId][msg.sender] = true;
        proposals[_proposalId].approvals++;

        if (proposals[_proposalId].approvals >= requiredApprovals) {
            _executeTransaction(_proposalId);
        }

        emit Voted(_proposalId, msg.sender);
    }

    /**
     * @notice Adds a new validator to the contract
     * @param validator The address of the new validator
     */
    function _addValidator(address validator) internal {
        require(
            _checkValidator(validator),
            "Validator must be registered on Smart Nodes"
        );

        isValidator[validator] = true;
        validators.push(validator);
        _updateApprovalRequirements();

        emit ValidatorAdded(validator);
    }

    /**
     * @notice Executes a proposal if it has enough approvals. Only to be called by approveTransaction
     * @param _proposalId The ID of the proposal to be executed
     */
    function _executeTransaction(uint256 _proposalId) internal {
        require(!proposals[_proposalId].executed, "Proposal already executed");

        Proposal storage proposal = proposals[_proposalId];

        for (uint i = 0; i < proposal.data.length; i++) {
            (bool success, ) = parentContract.call(
                abi.encodeWithSelector(
                    _getFunctionSelector(proposal.functionTypes[i]),
                    proposal.data[i]
                )
            );
            require(success, "Proposal execution failed!");
        }

        proposals[_proposalId].executed = true;
        lastProposal = block.timestamp;

        _rewardValidators(_proposalId);

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
     * @notice Checks if a given address is a valid validator on the parent contract
     * @param validator The address to check
     * @return True if the address is a valid validator, false otherwise
     */
    function _checkValidator(address validator) internal view returns (bool) {
        require(validator != address(0), "Invalid address.");
        require(!isValidator[validator], "Validator already registered.");

        // Append the validator address to the contract query
        bytes memory confirmValidatorCall = abi.encodeWithSelector(
            _getFunctionSelector(FunctionType.ConfirmValidator),
            validator
        );

        // Perform staticcall with the new data (get validator)
        (bool success, bytes memory returnData) = parentContract.staticcall(
            confirmValidatorCall
        );
        require(success, "Static call failed.");

        // Decode the return data to check if the validator is registered
        bool isVal = abi.decode(returnData, (bool));
        return isVal;
    }

    /**
     * @notice Returns the function selector for a given function type
     * @param functionType The type of function
     * @return The function selector
     */
    function _getFunctionSelector(
        FunctionType functionType
    ) internal pure returns (bytes4) {
        if (functionType == FunctionType.ConfirmValidator) {
            return bytes4(keccak256("validatorIdByAddress(address)"));
        } else if (functionType == FunctionType.UpdateValidator) {
            return bytes4(keccak256("updateValidator(address)"));
        } else if (functionType == FunctionType.UpdateJob) {
            return bytes4(keccak256("updateJob(bytes)"));
        } else if (functionType == FunctionType.UpdateWorkers) {
            return bytes4(keccak256("updateWorkers(bytes)"));
        } else {
            revert("Invalid function type");
        }
    }
}
