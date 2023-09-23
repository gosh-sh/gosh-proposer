pragma solidity ^0.8.13;

contract Elock {
    uint256 public totalSupply; // 0x0
    uint256 public trxCount; // 0x1


    constructor() {}

    function deposit(uint256 pubkey) public payable {
        require(msg.value > 100 wei);

        unchecked {
            totalSupply += msg.value;
        }
        trxCount += 1;
    }
}