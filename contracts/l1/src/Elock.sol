pragma solidity ^0.8.13;

contract Elock {
    uint256 public totalSupply; // 0x0
    uint128 public trxDepositCount; // 0x1
    uint128 public trxWithdrawCount; // 0x1
    uint256 elockStartBlock; // 0x2
    uint256 glockStartBlock; // 0x3
    uint256 public lastProcessedBlock; // 0x4
    mapping (uint256 => WithdrawProposal) withdrawals; // 0x5
    uint256[] private _proposalKeys; // 0x6

    struct Transfer {
        address payable to;
        uint256 value;
        uint256 trxHash;
    }

    struct WithdrawProposal {
        uint256 fromBlock;
        uint256 tillBlock;
        Transfer[] transfers;
    }

    constructor(uint256 startBlock) {
        elockStartBlock = uint256(blockhash(block.number));
        glockStartBlock = startBlock;
        lastProcessedBlock = startBlock;
    }

    function deposit(uint256 pubkey) public payable {
        require(msg.value > 100 wei);
        pubkey;

        unchecked {
            totalSupply += msg.value;
        }
        trxDepositCount += 1;
    }

    function startWithdrawalProposal(
        uint256 fromBlock,
        uint256 tillBlock,
        Transfer[] calldata transfers
    ) public {
        if (lastProcessedBlock > 0) {
            require(uint256(fromBlock) == lastProcessedBlock);
        }
        bytes memory blockRange = bytes.concat(bytes32(fromBlock), bytes32(tillBlock));
        uint256 proposalKey = uint256(keccak256(blockRange));

        require(withdrawals[proposalKey].fromBlock == 0, "Proposal already exists");

        WithdrawProposal storage wp = withdrawals[proposalKey];
        wp.fromBlock = fromBlock;
        wp.tillBlock = tillBlock;
        for (uint256 index = 0; index < transfers.length; index++) {
            wp.transfers.push(transfers[index]);
        }
        _proposalKeys.push(proposalKey);
    }

    function getProposalList() public view returns (uint256[] memory proposalKeys) {
        return _proposalKeys;
    }

    function getProposal(uint256 proposalKey) public view
    returns (uint256 fromBlock, uint256 tillB, Transfer[] memory transfers)
    {
        WithdrawProposal memory prop = withdrawals[proposalKey];
        return (prop.fromBlock, prop.tillBlock, prop.transfers);
    }
}