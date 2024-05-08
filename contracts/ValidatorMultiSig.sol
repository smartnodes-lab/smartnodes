// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/access/Ownable.sol";

contract ValidatorMultiSig is Ownable {
    struct Transaction {
        uint256 id;
        bytes data;
        bool executed;
        uint256 approvals;
    }

    address[] public validators;
    mapping(address => bool) public isValidator;
    mapping(uint256 => Transaction) public transactions;
    mapping(uint256 => mapping(address => bool)) public approvals;

    uint256 public nextTransactionId;
    uint256 public requiredApprovals;

    event ValidatorAdded(address validator);
    event ValidatorRemoved(address validator);
    event ProposalCreated(
        uint256 transactionId,
        address targetContract,
        bytes data
    );
    event Voted(uint256 transactionId, address validator);
    event TransactionExecuted(uint256 transactionId);
    event Deposit(address indexed sender, uint amount);

    // modifier onlyOwner() {
    //     require(isValidator[msg.sender], "Not owner");
    //     _;
    // }

    constructor(address[] memory _validators, uint256 _requiredApprovals) {
        require(_validators.length > 0, "Owners required");
        require(
            _requiredApprovals > 0 && _requiredApprovals <= _validators.length,
            "Approvals required must be > 0"
        );

        for (uint i = 0; i < _validators.length; i++) {
            address validator = _validators[i];

            require(validator != address(0), "Invalid address");
            require(!isValidator[validator], "Validator already registered");

            isValidator[validator] = true;
            validators.push(validator);
        }

        requiredApprovals = _requiredApprovals;
    }

    receive() external payable {
        emit Deposit(msg.sender, msg.value);
    }

    function createTransaction(bytes calldata _data) external onlyOwner {
        transactions[nextTransactionId] = Transaction({
            id: nextTransactionId,
            data: _data,
            executed: false,
            approvals: 0
        });
    }
}
