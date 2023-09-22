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
    optional(uint256) _prevhash;
    bool _status = false;

    uint128 _index = 0;

    TvmCell _proposalCode;

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
    ) accept {
    }

    function setProposalCode(TvmCell code) public onlyOwner accept {
        _proposalCode = code;
    }

    function checkData(BlockData[] data, TransactionBatch transactions) public onlyOwner accept {
        tvm.accept();
        if (data.length == 0) {
            return;
        }
        _status = true;
        this.checkDataIndex{value: 0.1 ton, flag: 1}(data, transactions, 0);
    }

    function checkDataIndex(BlockData[] data, TransactionBatch transactions, uint128 index) public senderIs(this) accept {
        if (index >=  data.length) { 
            TvmCell s1 =  ProposalLib.composeProposalStateInit(_proposalCode, _prevhash, _index);
            new Proposal{stateInit: s1, value: 5 ton, wid: 0, flag: 1}(_prevhash, data[index - 1].hash, transactions);
            _index += 1;        
            return; 
        }
        TvmSlice dataslice = TvmSlice(data[index].data);
        (uint8 count) = dataslice.load(uint8);
        count -= 247;
        dataslice.skip(count * 8);
        dataslice.skip(8);
        (uint256 newhash) = dataslice.load(uint256);
        if (index == 0) {
            if (_prevhash.hasValue()) {
                if (_prevhash.get() != newhash) {
                    _status = false;     
                    return;
                }
            }
        }
        else {
            if (data[index - 1].hash != newhash) {
                _status = false;     
                return;
            }
        }
        if (gosh.keccak256(data[index].data) != data[index].hash) { 
            _status = false;
            return; 
        }
        this.checkDataIndex{value: 0.1 ton, flag: 1}(data, transactions, index + 1);
    }

    function setNewHash(uint256 prevhash, uint256 newhash) public {
        if (_prevhash.hasValue()) {
            require(_prevhash.get() == prevhash, ERR_WRONG_HASH);
        }
        _prevhash = newhash;
    }

    //Fallback/Receive
    receive() external {
        if (msg.sender == this) {
            _status = false;
        }
    }
    
    onBounce(TvmSlice body) external {
        body;
        if (msg.sender == this) {
            _status = false;
        }
    }
    
    fallback() external {
        if (msg.sender == this) {
            _status = false;
        }
    }

    //Getter 
    function getStatus() external view returns(optional(uint256) prevhash, bool status) {
        return (_prevhash, _status);
    }
}
