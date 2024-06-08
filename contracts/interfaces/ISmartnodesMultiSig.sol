// SPDX-License-Identifier: MIT
pragma solidity ^0.8.5;

interface ISmartnodesMultiSig {
    function initialize(address target) external;
    function createProposal(
        FunctionType[] calldata _functionTypes,
        bytes[] calldata _data
    ) external;
    function approveTransaction(uint256 _proposalId) external;
    function removeValidator(address validator) external;
    function generateValidatorCandidates()
        external
        view
        returns (address[] memory);

    enum FunctionType {
        UpdateValidator,
        ConfirmValidator,
        CompleteJob,
        DisputeJob
    }

    event ProposalCreated(uint256 proposalId, bytes data);
    event Voted(uint256 proposalId, address validator);
    event ProposalExecuted(uint256 proposalId);
    event Deposit(address indexed sender, uint amount);
    event ValidatorAdded(address validator);
    event ValidatorRemoved(address validator);
}
