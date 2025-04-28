// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "./Property.sol";

contract PropertyFactory is Ownable {
    address[] public allProperties;
    event PropertyDeployed(address indexed propertyAddress, uint256 index);

    constructor(address initialOwner_) Ownable(initialOwner_) {}

    function createProperty(
        string  memory uri_,
        IERC20 paymentToken_,
        bytes32 metadataHash_,
        uint256 totalTokens_,
        uint256 pricePerToken_,
        uint16 annualReturnBP_
    ) external onlyOwner returns (address) {
        Property prop = new Property(
            uri_,
            paymentToken_,
            metadataHash_,
            totalTokens_,
            pricePerToken_,
            annualReturnBP_,
            msg.sender
        );
        allProperties.push(address(prop));
        emit PropertyDeployed(address(prop), allProperties.length - 1);
        return address(prop);
    }

    function count() external view returns (uint256) {
        return allProperties.length;
    }
}
