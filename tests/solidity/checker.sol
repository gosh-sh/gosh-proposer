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

contract Checker {

    modifier onlyOwner {
        require (msg.pubkey() == tvm.pubkey(), 100) ;
        _;
    }

    modifier accept {
        tvm.accept();
        _;
    }

    modifier senderIs(address sender) {
        require(msg.sender == sender, 100);
        _;
    }
    
    constructor(
    ) accept {
    }

    function checkData(UnknownData[] data) public view onlyOwner accept {
        tvm.accept();
        this.checkDataIndex{value: 0.1 ton, flag: 1}(data, 0);
    }

    function checkDataIndex(UnknownData[] data, uint128 index) public pure senderIs(this) accept {
        if (index >=  data.length) { 
            this.isCorrect{value: 0.2 ton, flag: 1}(true); 
            return; 
        }
        if (gosh.keccak256(data[index].data) != data[index].hash) { 
            this.isCorrect{value: 0.2 ton, flag: 1}(false); 
            return; 
        }
        this.checkDataIndex{value: 0.1 ton, flag: 1}(data, index + 1);
    }

    function isCorrect(bool res) public pure senderIs(this) accept {
        res;
        return;
    }
}
