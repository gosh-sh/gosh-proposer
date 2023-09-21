// SPDX-License-Identifier: GPL-3.0-or-later
/*
 * GOSH contracts
 *
 * Copyright (C) 2022 Serhii Horielyshev, GOSH pubkey 0xd060e0375b470815ea99d6bb2890a2a726c5b0579b83c742f5bb70e10a771a04
 */
pragma ever-solidity >=0.66.0;
pragma AbiHeader expire;
pragma AbiHeader pubkey;

struct UnknownData {
    bytes data;
    uint256 hash;
}

uint16 constant ERR_WRONG_SENDER = 100;
uint16 constant ERR_WRONG_HASH = 101;

contract Checker {
    optional(uint256) _prevhash;
    bool _status = false;

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

    function checkData(UnknownData[] data) public onlyOwner accept {
        tvm.accept();
        if (data.length == 0) {
            this.isCorrect{value: 0.2 ton, flag: 1}(true); 
            return;
        }
        _status = true;
        this.checkDataIndex{value: 0.1 ton, flag: 1}(data, 0);
    }

    function checkDataIndex(UnknownData[] data, uint128 index) public senderIs(this) accept {
        if (index >=  data.length) { 
            _prevhash = data[index - 1].hash;
            this.isCorrect{value: 0.2 ton, flag: 1}(true); 
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
                    this.isCorrect{value: 0.2 ton, flag: 1}(false);      
                    return;
                }
            }
        }
        else {
            if (data[index - 1].hash != newhash) {
                this.isCorrect{value: 0.2 ton, flag: 1}(false);      
            }
        }
        if (gosh.keccak256(data[index].data) != data[index].hash) { 
            this.isCorrect{value: 0.2 ton, flag: 1}(false); 
            return; 
        }
        this.checkDataIndex{value: 0.1 ton, flag: 1}(data, index + 1);
    }

    function isCorrect(bool res) public senderIs(this) accept {
        _status = false;
        res;
        return;
    }

    //Fallback/Receive
    receive() external {
        if (msg.sender == this) {
            this.isCorrect{value: 0.2 ton, flag: 1}(false); 
            _status = false;
        }
    }
    
    onBounce(TvmSlice body) external {
        body;
        if (msg.sender == this) {
            this.isCorrect{value: 0.2 ton, flag: 1}(false); 
            _status = false;
        }
    }
    
    fallback() external {
        if (msg.sender == this) {
            this.isCorrect{value: 0.2 ton, flag: 1}(false); 
            _status = false;
        }
    }

    //Getter 
    function getStatus() external view returns(optional(uint256) prevhash, bool status) {
        return (_prevhash, _status);
    }
}
