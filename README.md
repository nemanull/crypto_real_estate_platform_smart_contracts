# crypto_real_estate_platform_smart_contracts
Basic smart contracts for EVM-compatible blockchains and Solana that are required to organize simple real estate operations on-chain.

## Project Structure

├── evm_base/
│   ├── propertyFactory.sol    # Factory for deploying property contracts
│   ├── propertyContract.sol   # Contract representing a property
│   └── custom_erc20.sol       # Custom ERC20 token 
|
├── sol_base/
│   └── real_estate_usdc.rs    # Solana program for property tokenization


## Contract Overview

### EVM Contracts

- PropertyFactory: Creates new property token contracts and tracks all deployed properties on the network.
- Property: ERC1155 token representing fractional ownership of real estate with yield distribution capabilities.
- custom_erc20: Simple ERC20 token implementation for testing payment functionality.

### Example of Remix friendly payload for property creating using propertyFactory

```javascript
// Function: createProperty(uri, paymentToken, metadataHash, totalTokens, pricePerToken, annualReturnBP)

// Example values:
"ipfs://bafybeiedfmlanue4o3hry7a6iov2p6gid7dqvods5x7m5q6ykblazgd7qu/10.json", // Property metadata URI
"0xdAC17F958D2ee523a2206206994597C13D831ec7",                                  // USDT contract address
"0xc74d62a40c739158622c47740fba58c30d1b1fbea91d7cd172557b1978609b17",          // Metadata hash for validation
1200,                                                                           // Annual return in basis points (12.00%)
4000000000000000000000,                                                         // Price per token (4000 tokens with 18 decimals)
1300                                                                            // Total number of tokens for this property
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `uri` | string | IPFS link to property metadata JSON containing images, description, location, etc. |
| `paymentToken` | address | ERC20 token contract address used for payments (USDT, USDC, DAI, etc.) |
| `metadataHash` | bytes32 | Keccak256 hash of property documents for verification |
| `annualReturnBP` | uint16 | Expected annual return in basis points (100bp = 1%) |
| `pricePerToken` | uint256 | Cost per token in paymentToken units (includes decimals) |
| `totalTokens` | uint256 | Total number of ownership tokens for this property |
