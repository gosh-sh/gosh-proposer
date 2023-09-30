// SPDX-License-Identifier: GPL-3.0-or-later
/*
 * GOSH contracts
 *
 * Copyright (C) 2022 Serhii Horielyshev, GOSH pubkey 0xd060e0375b470815ea99d6bb2890a2a726c5b0579b83c742f5bb70e10a771a04
 */
pragma ever-solidity >=0.66.0;
pragma AbiHeader expire;
pragma AbiHeader pubkey;

import "checkerLib.sol";
import "proposal.sol";

contract Checker {

    string constant _version = "1.0.0";

    uint256 _prevhash;

    address _root;
    uint128 _proposalCount = 0;

    //value = a*x/10000 + b
    uint128 a = 10; 
    uint128 b = 0; //1e-18 GETH

    TvmCell _proposalCode;

    TransactionBatch[] _transactions;

    bool _isReady = false;

    modifier onlyOwner {
        require (msg.pubkey() == tvm.pubkey(), ERR_WRONG_SENDER) ;
        _;
    }

    modifier accept {
        tvm.accept();
        _;
    }

    modifier senderIs(address sender) {
        require(msg.sender == sender, ERR_WRONG_SENDER);
        _;
    }
    
    constructor(
        uint256 prevhash
    ) accept {
        _prevhash = prevhash;
    }

    function setHashRoot(uint256 hash) public onlyOwner accept {
        _prevhash = hash;
    }

    function setReadyRoot(bool ready) public onlyOwner accept {
        _isReady = ready;
    }

    function setRootContract (address root) public onlyOwner accept {
        _root = root;
    }

    function setProposalCode(TvmCell code) public onlyOwner accept {
        _proposalCode = code;
    }

    function setCommission(uint128 a_from_ax_div10000_plus_b, uint128 b_from_ax_div10000_plus_b) public onlyOwner accept {
        a = a_from_ax_div10000_plus_b;
        b = b_from_ax_div10000_plus_b;
    }

    function checkData(BlockData[] data, TransactionBatch[] transactions) public view accept {
        if (_isReady == false) { return; }
        if (data.length == 0) {
            return;
        }
        this.checkDataIndex{value: 0.1 ton, flag: 1}(data, transactions, 0);
    }

    function checkDataIndex(BlockData[] data, TransactionBatch[] transactions, uint128 index) public senderIs(this) accept {
        for (uint i = 0; i <= BATCH_SIZE; i++) {
            if (index >=  data.length) { 
                TvmSlice dataslicenew = TvmSlice(data[0].data);
                (uint8 countnew) = dataslicenew.load(uint8);
                countnew -= 247;
                dataslicenew.skip(countnew * 8);
                dataslicenew.skip(8);
                (uint256 newhashagain) = dataslicenew.load(uint256);
                if (_prevhash != newhashagain) {
                    return;
                }
                TvmCell s1 =  ProposalLib.composeProposalStateInit(_proposalCode, _prevhash, _proposalCount, this);
                new Proposal{stateInit: s1, value: 10 ton, wid: 0, flag: 1}(_prevhash, data[index - 1].hash, transactions);
                _proposalCount += 1;        
                return; 
            }
            TvmSlice dataslice = TvmSlice(data[index].data);
            (uint8 count) = dataslice.load(uint8);
            count -= 247;
            dataslice.skip(count * 8);
            dataslice.skip(8);
            (uint256 newhash) = dataslice.load(uint256);
            if (index == 0) {
                if (_prevhash != newhash) {
                    return;
                }
            }
            else {
                if (data[index - 1].hash != newhash) {
                    return;
                }
            }
            if (gosh.keccak256(data[index].data) != data[index].hash) {
                return; 
            }
            index += 1;
        }
        this.checkDataIndex{value: 0.1 ton, flag: 1}(data, transactions, index);
    }

    function setNewHash(uint256 prevhash, uint256 newhash, uint128 index, TransactionBatch[] transactions) public senderIs(ProposalLib.calculateProposalAddress(_proposalCode, _prevhash, index, this)) accept{
        require(_prevhash == prevhash, ERR_WRONG_HASH);
        ARootToken(_root).grantbatch{value:0.3 ton, flag: 1}(0, _transactions, a, b);
        _transactions = transactions;
        this.destroyTrash{value: 0.1 ton, flag: 1}(_prevhash, _proposalCount, 0);
        _prevhash = newhash;
        _proposalCount = 0;
    }

    function destroyTrash(uint256 prevhash, uint128 indexmax, uint128 index) public view senderIs(this) accept {
        for (uint128 i = 0; i < BATCH_SIZE; i++) {
            if (index + i > indexmax) {
                return;
            }
            Proposal(ProposalLib.calculateProposalAddress(_proposalCode, prevhash, index + i, this)).destroy{value: 0.1 ton, flag: 1}();
        }
        this.destroyTrash{value: 0.1 ton, flag: 1}(_prevhash, indexmax, index + BATCH_SIZE);
    }

    function updateCode(TvmCell newcode, TvmCell cell) public onlyOwner accept {
        tvm.setcode(newcode);
        tvm.setCurrentCode(newcode);
        cell = abi.encode(_prevhash, _root, _proposalCount, a, b, _proposalCode);
        onCodeUpgrade(cell);
    }

    function onCodeUpgrade(TvmCell cell) private  {
        (_prevhash, _root, _proposalCount, a, b, _proposalCode) = abi.decode(cell, (uint256, address, uint128, uint128, uint128, TvmCell));
        _isReady = false;
        TransactionBatch[] transactions;
        _transactions = transactions;
    }

    function onCodeUpgrade() private pure  {
    }

    //Fallback/Receive
    receive() external pure {
    }
    
    onBounce(TvmSlice body) external pure {
        body;
    }
    
    fallback() external pure {
    }

    //Getter 
    function getVersion() external pure returns(string, string) {
        return ("checker", _version);
    }

    function getProposalAddr(uint128 index) external view returns(address) {
        return ProposalLib.calculateProposalAddress(_proposalCode, _prevhash, index, this);
    }

    function getAllProposalAddr() external view returns(address[]) {
        address[] result;
        for (uint128 i = 0; i < _proposalCount; i++){
            result.push(ProposalLib.calculateProposalAddress(_proposalCode, _prevhash, i, this));
        }
        return result;
    }

    function getTransactions() external view returns(TransactionBatch[]) {
        return _transactions;
    }

    function getStatus() external view returns(uint256 prevhash, uint128 index) {
        return (_prevhash, _proposalCount);
    }
}
