// SPDX-License-Identifier: MIT
pragma solidity ^0.8.5;

interface ISmartnodesCore {
    function createUser(bytes32 _publicKeyHash) external;
    function createValidator(bytes32 _publicKeyHash) external;
    function requestJob(
        uint256[] calldata _capacities
    ) external returns (uint256[] memory);
    function completeJob(
        uint256 jobId,
        address[] memory _workers
    ) external returns (uint256[] memory);
    function disputeJob(uint256 jobId) external;
    function lockTokens(uint32 amount) external;
    function unlockTokens(uint32 amount) external;
    function mintTokens(address recipient, uint256 amount) external;
    function getJobValidators(
        uint256 jobId
    ) external view returns (address[] memory);
    function getUserCount() external view returns (uint256);
    function getValidatorCount() external view returns (uint256);
    function getActiveValidatorCount() external view returns (uint256);
    function getEmissionRate() external view returns (uint256);
    function getSupply() external view returns (uint256);
    function isLocked(address validatorAddr) external view returns (bool);
}
