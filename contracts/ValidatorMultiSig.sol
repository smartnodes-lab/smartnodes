// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/access/Ownable.sol";

contract ValidatorMultiSig is Ownable {
    struct Proposal {
        uint256 id;
        bytes[] data;
        bool executed;
        uint256 approvals;
    }

    mapping(address => bool) public isValidator;
    mapping(uint256 => Proposal) public proposals;
    mapping(uint256 => mapping(address => bool)) public approvals;
    
    address[] public validators;
    uint256 public nextProposalId;
    uint256 public requiredApprovals;
    uint256 public lastProposal = 0;
    uint256 public updateTime = 120;

    address public parentContract;
    bytes public confirmValidator;

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

    constructor(
        address target, // Address of the main contract (Smart Nodes)
        bytes memory data, // Call data for getValidator from Smart Nodes
        address[] memory _validators,
        uint256 _requiredApprovals
    ) {
        require(_validators.length > 0, "Owners required");
        require(
            _requiredApprovals > 0 && _requiredApprovals <= _validators.length,
            "Approvals required must be > 0"
        );

        parentContract = target;
        confirmValidator = data;

        // Add initial validators to the contract
        for (uint i = 0; i < _validators.length; i++) {
            address validator = _validators[i];

            bool isVal = _checkValidator(validator);

            require(isVal, "Validator must be registered on SmartNodes");

            isValidator[validator] = true;
            validators.push(validator);
        }

        requiredApprovals = _requiredApprovals;
    }

    receive() external payable {
        emit Deposit(msg.sender, msg.value);
    }

    function createTransaction(bytes[] calldata _data) external onlyOwner {
        require(block.timestamp - updateTime > lastProposal, "Proposals must be submitted after updateTime since last approved proposal.")
        proposals[nextProposalId] = Proposal({
            id: nextProposalId,
            data: _data,
            executed: false,
            approvals: 0
        });

        emit ProposalCreated(nextProposalId, _data);
        nextProposalId++;
    }

    function approveTransaction(uint256 _proposalId) external onlyOwner {
        require(
            isValidator[msg.sender],
            "Only validators can approve transactions"
        );
        require(
            !approvals[_proposalId][msg.sender],
            "Validator already approved this transaction"
        );

        approvals[_proposalId][msg.sender] = true;
        proposals[_proposalId].approvals++;

        if (proposals[_proposalId].approvals >= requiredApprovals) {
            _executeTransaction(_proposalId);
        }

        emit Voted(_proposalId, msg.sender);
    }

    function _executeTransaction(uint256 _proposalId) internal {
        require(
            !proposals[_proposalId].executed,
            "Proposal already executed"
        );

        for (uint i = 0; i < proposals[_proposalId].data.length; i++) {
            (bool success, ) = address(this).call(
                proposals[_proposalId].data[i]
            );
            require(success, "Proposal execution failed");
        }

        proposals[_proposalId].executed = true;
        lastProposal = block.timestamp;

        emit TransactionExecuted(_proposalId);
    }

    function _checkValidator(address validator) internal returns(bool) {
        require(validator != address(0), "Invalid address.");
        require(!isValidator[validator], "Validator already registered.");

        // Append the validator address to the contract query
        bytes memory confirmValidatorCall = abi.encodePacked(confirmValidator, validator);

        // Perform staticcall with the new data (get validator)
        (bool success, bytes memory returnData) = target.staticcall(
            confirmValidatorCall
        );
        require(success, "Static call failed.");

        // Decode the return data to check if the validator is registered
        bool isVal = abi.decode(returnData, (bool));
        return isVal;
    }

    function addValidator(address validator) external {
        require()
    }
}
