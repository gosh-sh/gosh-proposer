pragma solidity ^0.8.13;

import {IERC20} from "./IERC20.sol";

contract Elock {
    event Deposited(address indexed token, address from, uint256 pubkey, uint256 value);
    event Withdrawal(address indexed token, address indexed to, uint256 value, uint256 commission);
    event WithdrawExecuted(uint256 indexed proposalKey);
    event WithdrawRejected(uint256 indexed proposalKey, address indexed voter, uint8 reason);

    uint256 constant COMMISSION_WITHDRAWAL_THRESHOLD = 0.25 ether;
    address constant ETH = address(0);
    uint256 immutable glockStartBlock;
    address payable immutable commissionWallet;
    address immutable deployer;

    uint256 public totalSupply; // 0x0
    uint128 public trxDepositCount; // 0x1
    uint128 public trxWithdrawCount; // 0x1
    uint256 elockStartBlock; // 0x2
    uint256 public lastProcessedL2Block; // 0x3

    mapping (address => bool) validators; // 0x4
    address[] proposedValidators; // 0x5
    uint256 validatorsProposalRound; // 0x6

    mapping (uint256 => WithdrawProposal) withdrawals; // 0x7
    uint256[] private _proposalKeys; // 0x8

    uint256 votesRequired; // 0x9

    mapping (address => bool) votingForChangeValidators; // 0xa
    uint256 collectedVotesForChangeValidators; // 0xb
    uint256 collectedVotesAgainstChangeValidators; // 0xc

    mapping (uint256 => mapping (address => bool)) votingForWithdrawal; // 0xd
    mapping (uint256 => uint256) votesPerProposal; // 0xe
    uint256 votingDisposalFee; // 0xf

    mapping (address => FreezeVote) votingForFreeze; // 0x10
    address[] votedForFreeze; // 0x11
    bool isDepositsFreezed; // 0x12
    uint256 collectedCommission; // 0x13

    address[] validatorsIndex; // 0x14

    mapping (address => mapping (address => ERC20WithdrawalApprovement)) approvedWithdrawals; // 0x15
    mapping (address => uint) totalSupplies; // 0x16
    address[] tokenRoots; // 0x17
    uint public proposalWithdrawalFee = 400_000;

    struct Transfer {
        address token;
        address payable to;
        uint256 value;
        uint256 trxHash;
    }

    struct WithdrawProposal {
        uint256 fromBlock;
        uint256 tillBlock;
        Transfer[] transfers;
    }

    struct ERC20WithdrawalApprovement {
        uint value;
        uint commission;
    }

    enum FreezeVote { None, Freeze, Unfreeze }

    error Unauthorized();

    modifier onlyGoshValidators {
        if (!validators[msg.sender]) {
            revert Unauthorized();
        }
        _;
    }

    constructor(
        uint256 _initialL2Block,
        address payable _commissionWallet,
        address[] memory _goshValidators
    ) {
        require(_initialL2Block > 0);
        require(_goshValidators.length > 0);

        elockStartBlock = uint256(blockhash(block.number));
        glockStartBlock = _initialL2Block;
        lastProcessedL2Block = _initialL2Block;
        commissionWallet = _commissionWallet;
        deployer = msg.sender;
        for (uint256 i = 0; i < _goshValidators.length; i++) {
            validators[_goshValidators[i]] = true;
        }
        validatorsIndex = _goshValidators;
        votesRequired = _calcRequiredVotes(_goshValidators.length);
    }

    receive() external payable {}

    /// pubkey - GOSH pubkey (32 bytes)
    function deposit(uint256 pubkey) public payable {
        require(isDepositsFreezed == false);
        require(msg.value > 0.01 ether);
        pubkey;

        unchecked {
            totalSupply += msg.value;
        }
        if (totalSupplies[address(0)] == 0) {
            tokenRoots.push(address(0));
        }
        totalSupplies[address(0)] += msg.value;
        trxDepositCount += 1;
        emit Deposited(address(0), msg.sender, pubkey, msg.value);
    }

    function depositERC20(address token, uint256 value, uint256 pubkey) public payable {
        require(isDepositsFreezed == false);
        require(token != address(0));

        uint256 allowedAmount = IERC20(token).allowance(msg.sender, address(this));
        require(allowedAmount >= value, "Not approved");

        bool isOk = IERC20(token).transferFrom(msg.sender, address(this), value);
        require(isOk, "Transfer failed");

        trxDepositCount += 1;

        if (totalSupplies[token] == 0) {
            tokenRoots.push(token);
        }
        totalSupplies[token] += value;

        emit Deposited(token, msg.sender, pubkey, value);
    }

    function withdrawERC20(address token) public payable {
        ERC20WithdrawalApprovement storage approvement = approvedWithdrawals[token][msg.sender];
        require(approvement.value > 0);
        require(msg.value == approvement.commission);

        uint value = approvement.value;
        uint commission = approvement.commission;
        delete approvedWithdrawals[token][msg.sender];
        bool isOk = IERC20(token).transfer(msg.sender, value);
        if (isOk) {
            trxWithdrawCount += 1;
            totalSupplies[token] -= value;

            if (totalSupplies[token] == 0) {
                removeTokenRoot(token);
            }
            emit Withdrawal(token, msg.sender, value, commission);
        }
    }

    function proposeWithdrawal(
        uint256 fromBlock,
        uint256 tillBlock,
        Transfer[] calldata transfers
    )
        public payable onlyGoshValidators()
    {
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

    function voteForWithdrawal(uint256 proposalKey)
        public payable onlyGoshValidators()
    {
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
            }
        } else {
            collectedVotesAgainstChangeValidators += 1;
            if (collectedVotesAgainstChangeValidators > validatorsIndex.length - votesRequired) {
                delete proposedValidators;
            }
        }
    }

    function freezeDeposits() public payable onlyGoshValidators() {
        require(isDepositsFreezed == false, "Deposit already freezed");
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

    function setProposalWithdrawalFee(uint fee) public {
        require(msg.sender == deployer);
        require(fee > 0);
        require(fee != proposalWithdrawalFee);

        proposalWithdrawalFee = fee;
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
        return validatorsIndex;
    }

    function getProposedValidators() public view returns (address[] memory proposedValidatorSet) {
        return proposedValidators;
    }

    function getVotesForWithdrawal(uint256 proposalKey) public view returns (uint256 votes) {
        return votesPerProposal[proposalKey];
    }

    function getTotalSupply(address token_root) public view returns (uint amount) {
        return totalSupplies[token_root];
    }

    function getTokenRoots() public view returns (address[] memory roots) {
        return tokenRoots;
    }

    function getMyVoteForWithdrawal(uint256 proposalKey, address myAddress)
        public view returns (bool isVoted)
    {
        return votingForWithdrawal[proposalKey][myAddress];
    }

    function getERC20Approvement(address token, address recipient)
        public view
    returns (uint value, uint commission) {
        return (
            approvedWithdrawals[token][recipient].value,
            approvedWithdrawals[token][recipient].commission
        );
    }

    function _calcRequiredVotes(uint256 validatorsCount) private pure returns (uint256) {
        return validatorsCount / uint256(2) + 1; // 50% + 1 vote
    }

    function _calculateFinalizeProposalFee() private view returns (uint256) {
        return proposalWithdrawalFee;
    }

    function _updateValidators() private {
        // TODO don't reset validators that come by proposal
        for (uint256 i = 0; i < validatorsIndex.length; i++) {
            delete validators[validatorsIndex[i]];
        }
        for (uint256 i = 0; i < proposedValidators.length; i++) {
            validators[proposedValidators[i]] = true;
        }
        validatorsIndex = proposedValidators;
        votesRequired = _calcRequiredVotes(proposedValidators.length);
        delete proposedValidators;
    }

    function _executeWithdrawals(uint256 proposalKey) private returns (bool) {
        Transfer[] memory transfers = withdrawals[proposalKey].transfers;
        uint256 requiredFunds;
        uint256 validatorCostsPerUser = votingDisposalFee * tx.gasprice / transfers.length;
        uint256 transactionFeeInGas = 21_000 * tx.gasprice + validatorCostsPerUser;
        uint transactionCollectedCommission = 0;
        uint128 tempTrxWithdrawCount = 0;
        uint tempTotalSupply = 0;

        for (uint256 index = 0; index < transfers.length; index++) {
            if (transfers[index].token == ETH) { // ETH withdrawal
                requiredFunds += transfers[index].value;
            }
        }
        if (requiredFunds > address(this).balance || requiredFunds > totalSupply) {
            return false;
        }

        for (uint256 index = 0; index < transfers.length; index++) {
            Transfer memory transfer = transfers[index];
            if (transfer.token == ETH) { // ETH withdrawal
                if (transfer.value > transactionFeeInGas) {
                    uint256 withdrawalValue = transfer.value - transactionFeeInGas;
                    transfer.to.transfer(withdrawalValue);
                    emit Withdrawal(transfer.token, transfer.to, withdrawalValue, transactionFeeInGas);
                    transactionCollectedCommission += validatorCostsPerUser;
                }

                tempTrxWithdrawCount += 1;
                tempTotalSupply += transfer.value;
            } else {
                _approveWithdrawal(transfer.token, transfer.to, transfer.value, validatorCostsPerUser);
            }
        }

        trxWithdrawCount += tempTrxWithdrawCount;
        totalSupply -= tempTotalSupply;
        totalSupplies[address(0)] -= tempTotalSupply;

        if (totalSupplies[address(0)] == 0) {
            removeTokenRoot(address(0));
        }
        collectedCommission += transactionCollectedCommission;
        if (collectedCommission > COMMISSION_WITHDRAWAL_THRESHOLD) {
            commissionWallet.transfer(collectedCommission - 21_000 * tx.gasprice);
            collectedCommission = 0;
        }
        return true;
    }

    function _approveWithdrawal(address token, address to, uint value, uint interimCommission)
        private
    {
        ERC20WithdrawalApprovement storage approvement = approvedWithdrawals[token][to];
        approvement.value += value;
        approvement.commission += interimCommission;
    }

    function _cleanWithdrawProposals() private {
        for (uint256 i = 0; i < _proposalKeys.length; i++) {
            uint256 key = _proposalKeys[i];
            delete withdrawals[key];

            // clear voting for selected proposal
            for (uint256 j = 0; j < validatorsIndex.length; j++) {
                address validator = validatorsIndex[j];
                delete votingForWithdrawal[key][validator];
            }
            delete votesPerProposal[key];
        }
        delete _proposalKeys;
        votingDisposalFee = 0;
    }

    function removeTokenRoot(address root) private {
        for (uint256 i = 0; i < tokenRoots.length; i++) {
            if (tokenRoots[i] == root) {
                tokenRoots[i] = tokenRoots[tokenRoots.length - 1];
                tokenRoots.pop();
                return;
            }
        }
    }

    function gasPrice() public view returns (uint256) {
        return tx.gasprice;
    }

    function votedForFreezeList() public view returns (address[] memory) {
        return votedForFreeze;
    }
}