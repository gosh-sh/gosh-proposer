pragma solidity ^0.8.13;

// todo
// * check if elock's balance is enough for execute withdrawals
// * processing if not enough funds for some withdrawals (mark proposal as unfeasible?)
// * calculate how many withdrawals we can do per transaction
//
contract Elock {
    uint256 public totalSupply; // 0x0
    uint128 public trxDepositCount; // 0x1
    uint128 public trxWithdrawCount; // 0x1
    uint256 elockStartBlock; // 0x2
    uint256 glockStartBlock; // 0x3
    uint256 public lastProcessedBlock; // 0x4

    address[] validators; // 0x5
    address[] proposedValidators; // 0x6

    mapping (uint256 => WithdrawProposal) withdrawals; // 0x7
    uint256[] private _proposalKeys; // 0x8

    uint256 votesRequired; // 0x9

    mapping (address => bool) votingForChangeValidators; // 0xa
    uint256 collectedVotesForChangeValidators; // 0xb

    mapping (uint256 => mapping (address => bool)) votingForWithdrawal; // 0xc
    mapping (uint256 => uint256) votesPerProposal; // 0xd

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

    error Unauthorized();

    modifier onlyGoshValidators {
        if (!_isPermitedValidator(msg.sender)) {
            revert Unauthorized();
        }
        _;
    }

    constructor(uint256 startBlock, address[] memory validatorSet) {
        require(validatorSet.length > 0);

        elockStartBlock = uint256(blockhash(block.number));
        glockStartBlock = startBlock;
        lastProcessedBlock = startBlock;
        validators = validatorSet;
        votesRequired = _calcVotes(validatorSet.length);
    }

    receive() external payable {
        //
    }

    /// pubkey - GOSH pubkey (32 bytes)
    function deposit(uint256 pubkey) public payable {
        require(msg.value > 100 wei);
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

    function voteForWithdrawal(uint256 proposalKey) public payable onlyGoshValidators() {
        require(votingForWithdrawal[proposalKey][msg.sender] == false, "Already voted");

        votingForWithdrawal[proposalKey][msg.sender] = true;
        votesPerProposal[proposalKey] += 1;

        if (votesPerProposal[proposalKey] >= votesRequired) {
            bool isExecuted = _executeWithdrawals(proposalKey);
            if (!isExecuted) {
                // revert vote
                votingForWithdrawal[proposalKey][msg.sender] = false;
                votesPerProposal[proposalKey] -= 1;
            } else {
                _finalizeProposal(proposalKey);
            }
        }
    }

    function proposeChangeValidators(address[] memory validatorSet)
        public payable onlyGoshValidators()
    {
        require(proposedValidators.length == 0);
        require(validatorSet.length > 0);
        proposedValidators = validatorSet;
    }


    function voteForChangeValidators() public payable onlyGoshValidators() {
        require(votingForChangeValidators[msg.sender] == false, "Already voted");

        votingForChangeValidators[msg.sender] = true;
        collectedVotesForChangeValidators += 1;

        if (collectedVotesForChangeValidators >= votesRequired) {
            _updateValidators();
        }
    }

    // function proposeCancelValidatorsChange() public onlyGoshValidators() {}
    // function voteForCancelValidatorsChange() public onlyGoshValidators() {}

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

    function _isPermitedValidator(address caller) private view returns (bool) {
        for (uint256 index = 0; index < validators.length; index++) {
            if (caller == validators[index]) {
                return true;
            }
        }
        return false;
    }

    function _calcVotes(uint256 validatorsCount) private pure returns (uint256) {
        if (validatorsCount == 1) {
            return 1;
        } else if (validatorsCount <= 3) {
            return 2;
        } else {
            return validatorsCount / uint256(2) + 1;
        }
    }

    function _updateValidators() private {
        collectedVotesForChangeValidators = 0;
        validators = proposedValidators;
        votesRequired = _calcVotes(proposedValidators.length);
        delete proposedValidators;
    }

    function _executeWithdrawals(uint256 proposalKey) private returns (bool) {
        Transfer[] memory transfers = withdrawals[proposalKey].transfers;
        uint256 requiredFunds;
        for (uint256 index = 0; index < transfers.length; index++) {
            requiredFunds += transfers[index].value;
        }
        if (requiredFunds > address(this).balance || requiredFunds > totalSupply) {
            return false;
        }

        for (uint256 index = 0; index < transfers.length; index++) {
            Transfer memory transfer = transfers[index];
            transfer.to.transfer(transfer.value);
            trxWithdrawCount += 1;
            totalSupply -= transfer.value;
        }
        return true;
    }

    function _finalizeProposal(uint256 proposalKey) private {
        lastProcessedBlock = withdrawals[proposalKey].fromBlock;

        // delete all proposals
        for (uint256 i = 0; i < _proposalKeys.length; i++) {
            uint256 key = _proposalKeys[i];
            delete withdrawals[key];

            // clear voting for selected proposal
            for (uint256 j = 0; j < validators.length; j++) {
                address validator = validators[j];
                delete votingForWithdrawal[key][validator];
            }
        }
        delete votesPerProposal[proposalKey];
        delete _proposalKeys;
    }
}