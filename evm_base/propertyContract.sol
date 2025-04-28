// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/token/ERC1155/ERC1155.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract Property is ERC1155, Ownable, ReentrancyGuard {
    using SafeERC20 for IERC20;

    IERC20 public immutable paymentToken;
    bytes32 public immutable metadataHash;
    uint16  public immutable annualReturnBP;
    uint256 public immutable totalTokens;
    uint256 public immutable pricePerToken;
    uint256 public tokensLeft;

    uint256 public constant TOKEN_ID = 0;
    uint256 public constant SOL_MOCK_ID = 1;
    uint256 private constant PRECISION= 1e18;

    uint256 public yieldPerToken;
    mapping(address => uint256) public lastYieldPerToken;

    event TokensPurchased(address indexed buyer, uint256 amount);
    event YieldDeposited(address indexed from, uint256 amount);
    event YieldClaimed(address indexed holder, uint256 amount);
    event SolanaMockMinted(address indexed to, uint256 amount);

    constructor(
        string  memory uri_,
        IERC20 paymentToken_,
        bytes32 metadataHash_,
        uint256 totalTokens_,
        uint256 pricePerToken_,
        uint16 annualReturnBP_,
        address owner_
    ) ERC1155(uri_) Ownable(owner_) {
        paymentToken = paymentToken_;
        metadataHash = metadataHash_;
        totalTokens = totalTokens_;
        tokensLeft = totalTokens_;
        pricePerToken = pricePerToken_;
        annualReturnBP = annualReturnBP;

        _mint(address(this), TOKEN_ID, totalTokens, "");
    }

    function buyTokens(uint256 amount) external nonReentrant {
        require(amount > 0 && amount <= tokensLeft, "Not enough left");
        uint256 cost = pricePerToken * amount;
        paymentToken.safeTransferFrom(msg.sender, address(this), cost);

        tokensLeft -= amount;
        _safeTransferFrom(address(this), msg.sender, TOKEN_ID, amount, "");

        emit TokensPurchased(msg.sender, amount);
    }

    function depositYield(uint256 amount) external onlyOwner {
        require(totalTokens > 0, "No tokens exist");
        paymentToken.safeTransferFrom(msg.sender, address(this), amount);

        yieldPerToken += (amount * PRECISION) / totalTokens;
        emit YieldDeposited(msg.sender, amount);
    }

    function claimYield() external nonReentrant {
        uint256 holderBalance = balanceOf(msg.sender, TOKEN_ID);
        require(holderBalance > 0, "No shares held");

        uint256 delta   = yieldPerToken - lastYieldPerToken[msg.sender];
        uint256 payment = (holderBalance * delta) / PRECISION;
        require(payment > 0, "Nothing to claim");

        lastYieldPerToken[msg.sender] = yieldPerToken;
        paymentToken.safeTransfer(msg.sender, payment);

        emit YieldClaimed(msg.sender, payment);
    }

    function mintSolanaMock(address to, uint256 amount) external onlyOwner {
        _mint(to, SOL_MOCK_ID, amount, "");
        emit SolanaMockMinted(to, amount);
    }
}
