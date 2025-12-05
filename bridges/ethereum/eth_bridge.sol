// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract SultanEthBridge {
    address public constant SULTAN_BRIDGE = 0x0000000000000000000000000000000000Sultan;
    uint256 public constant SULTAN_FEE = 0; // Zero fees forever!
    
    mapping(address => uint256) public bridgedETH;
    
    event ETHBridged(address indexed user, uint256 amount, uint256 sultanFee);
    
    function bridgeToSultan() external payable {
        require(msg.value > 0, "Amount must be greater than 0");
        
        bridgedETH[msg.sender] += msg.value;
        
        // Emit event with 0 fee on Sultan Chain side
        emit ETHBridged(msg.sender, msg.value, SULTAN_FEE);
    }
    
    function getSultanFee() public pure returns (uint256) {
        return 0; // Always returns 0 - no fees on Sultan Chain!
    }
}
