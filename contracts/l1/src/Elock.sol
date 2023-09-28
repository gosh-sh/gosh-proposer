pragma solidity ^0.8.13;

contract Elock {
    event WithdrawRejected(uint256 indexed proposalKey, address indexed voter, uint8 reason);
    event WithdrawExecuted(uint256 indexed proposalKey);
    event Withdrawal(address indexed recepient, uint256 value, uint256 commission);

    uint256 public totalSupply; // 0x0
    uint128 public trxDepositCount; // 0x1
    uint128 public trxWithdrawCount; // 0x1
    uint256 elockStartBlock; // 0x2
    uint256 immutable glockStartBlock; // 0x3
    uint256 public lastProcessedL2Block; // 0x4

    address[] validators; // 0x5
    address[] proposedValidators; // 0x6
    uint256 validatorsProposalRound; // 0x7

    mapping (uint256 => WithdrawProposal) withdrawals; // 0x8
    uint256[] private _proposalKeys; // 0x9

    uint256 votesRequired; // 0xa

    mapping (address => bool) votingForChangeValidators; // 0xb
    uint256 collectedVotesForChangeValidators; // 0xc
    uint256 collectedVotesAgainstChangeValidators; // 0xd

    mapping (uint256 => mapping (address => bool)) votingForWithdrawal; // 0xe
    mapping (uint256 => uint256) votesPerProposal; // 0xf
    uint256 votingDisposalFee; // 0x10

    mapping (address => FreezeVote) votingForFreeze; // 0x11
    address[] votedForFreeze; // 0x12
    bool isDepositsFreezed; // 0x13

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

    enum FreezeVote { None, Freeze, Unfreeze }

    error Unauthorized();

    modifier onlyGoshValidators {
        if (!_isPermitedValidator(msg.sender)) {
            revert Unauthorized();
        }
        _;
    }

    constructor(uint256 initialL2Block, address[] memory validatorSet) {
        require(initialL2Block > 0);
        require(validatorSet.length > 0);

        elockStartBlock = uint256(blockhash(block.number));
        glockStartBlock = initialL2Block;
        lastProcessedL2Block = initialL2Block;
        validators = validatorSet;
        votesRequired = _calcRequiredVotes(validatorSet.length);
    }

    receive() external payable {}

    /// pubkey - GOSH pubkey (32 bytes)
    function deposit(uint256 pubkey) public payable {
        require(isDepositsFreezed == false);
        require(msg.value > 1 gwei);
        pubkey;

        unchecked {
            totalSupply += msg.value;
        }
        trxDepositCount += 1;
    }

    function proposeWithdrawal(
        uint256 fromBlock,
        uint256 tillBlock,
        Transfer[] calldata transfers
    ) public payable onlyGoshValidators() {
        require(fromBlock == lastProcessedL2Block);

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
        votingDisposalFee += _calculateFinalizeProposalFee();
    }

    function voteForWithdrawal(uint256 proposalKey) public payable onlyGoshValidators() {
        require(withdrawals[proposalKey].fromBlock != 0, "Unknown proposal key");

        if (votingForWithdrawal[proposalKey][msg.sender] == false) {
            votingForWithdrawal[proposalKey][msg.sender] = true;
            votesPerProposal[proposalKey] += 1;
        }

        if (votesPerProposal[proposalKey] >= votesRequired) {
            bool isExecuted = _executeWithdrawals(proposalKey);
            if (isExecuted) {
                lastProcessedL2Block = withdrawals[proposalKey].tillBlock;
                _cleanWithdrawProposals();
                emit WithdrawExecuted(proposalKey);
            } else {
                emit WithdrawRejected(proposalKey, msg.sender, 3); // reason not enough funds
            }
        }
    }

    function proposeChangeValidators(address[] memory validatorSet)
        public payable onlyGoshValidators()
    {
        require(proposedValidators.length == 0);
        require(validatorSet.length > 0);
        proposedValidators = validatorSet;
        validatorsProposalRound += 1;
        collectedVotesForChangeValidators = 0;
        collectedVotesAgainstChangeValidators = 0;
    }

    function voteForChangeValidators(bool vote) public payable onlyGoshValidators() {
        require(votingForChangeValidators[msg.sender] == false, "Already voted");

        votingForChangeValidators[msg.sender] = true;
        if (vote) {
            collectedVotesForChangeValidators += 1;
            if (collectedVotesForChangeValidators >= votesRequired) {
                _updateValidators();
                // TODO clear voting structs
            }
        } else {
            collectedVotesAgainstChangeValidators += 1;
            if (collectedVotesAgainstChangeValidators > validators.length - votesRequired) {
                delete proposedValidators;
                // TODO clear voting structs
            }
        }
    }

    function freezeDeposits() public payable onlyGoshValidators() {
        require(isDepositsFreezed == false, "Deposit already unfreezed");
        require(collectedVotesForChangeValidators == 0, "Validators changing is in progress");
        require(votingForFreeze[msg.sender] != FreezeVote.Freeze);

        if ((votedForFreeze.length + 1) >= votesRequired) {
            isDepositsFreezed = true;
            for (uint256 i = 0; i < votedForFreeze.length; i++) {
                address validator = votedForFreeze[i];
                votingForFreeze[validator] = FreezeVote.None;
            }
            delete votedForFreeze;
        } else {
            votingForFreeze[msg.sender] = FreezeVote.Freeze;
            votedForFreeze.push(msg.sender);
        }
    }

    function unfreezeDeposits() public payable onlyGoshValidators() {
        require(isDepositsFreezed == true, "Deposit already unfreezed");
        require(collectedVotesForChangeValidators == 0, "Validators changing is in progress");
        require(votingForFreeze[msg.sender] != FreezeVote.Unfreeze);

        if ((votedForFreeze.length + 1) >= votesRequired) {
            isDepositsFreezed = false;
            for (uint256 i = 0; i < votedForFreeze.length; i++) {
                address validator = votedForFreeze[i];
                votingForFreeze[validator] = FreezeVote.None;
            }
            delete votedForFreeze;
        } else {
            votingForFreeze[msg.sender] = FreezeVote.Unfreeze;
            votedForFreeze.push(msg.sender);
        }
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

    function getValidators() public view returns (address[] memory currentValidatorSet) {
        return validators;
    }

    function getProposedValidators() public view returns (address[] memory proposedValidatorSet) {
        return proposedValidators;
    }

    function getVotesForWithdrawal(uint256 proposalKey) public view returns (uint256 votes) {
        return votesPerProposal[proposalKey];
    }

    function getMyVoteForWithdrawal(uint256 proposalKey) public view returns (bool isVoted) {
        return votingForWithdrawal[proposalKey][msg.sender];
    }

    function _isPermitedValidator(address caller) private view returns (bool) {
        for (uint256 index = 0; index < validators.length; index++) {
            if (caller == validators[index]) {
                return true;
            }
        }
        return false;
    }

    function _calcRequiredVotes(uint256 validatorsCount) private pure returns (uint256) {
        return validatorsCount / uint256(2) + 1;
    }

    function _updateValidators() private {
        validators = proposedValidators;
        votesRequired = _calcRequiredVotes(proposedValidators.length);
        delete proposedValidators;
    }

    function _executeWithdrawals(uint256 proposalKey) private returns (bool) {
        Transfer[] memory transfers = withdrawals[proposalKey].transfers;
        uint256 requiredFunds;
        uint256 transactionFeeInGas =
            21_000 * tx.gasprice + votingDisposalFee * tx.gasprice / transfers.length;
        for (uint256 index = 0; index < transfers.length; index++) {
            requiredFunds += transfers[index].value;
        }
        if (requiredFunds > address(this).balance || requiredFunds > totalSupply) {
            return false;
        }

        for (uint256 index = 0; index < transfers.length; index++) {
            Transfer memory transfer = transfers[index];
            if (transfer.value > transactionFeeInGas) {
                uint256 withdrawalValue = transfer.value - transactionFeeInGas;
                transfer.to.transfer(withdrawalValue);
                emit Withdrawal(transfer.to, withdrawalValue, transactionFeeInGas);
            }

            trxWithdrawCount += 1;
            totalSupply -= transfer.value;
        }
        return true;
    }

    function _calculateFinalizeProposalFee() private pure returns (uint256) {
        // todo
        return 0;
    }

    function _cleanWithdrawProposals() private {
        for (uint256 i = 0; i < _proposalKeys.length; i++) {
            uint256 key = _proposalKeys[i];
            delete withdrawals[key];

            // clear voting for selected proposal
            for (uint256 j = 0; j < validators.length; j++) {
                address validator = validators[j];
                delete votingForWithdrawal[key][validator];
            }
            delete votesPerProposal[key];
        }
        delete _proposalKeys;
        votingDisposalFee = 0;
    }

    function gasPrice() public view returns (uint256) {
        return tx.gasprice;
    }
}