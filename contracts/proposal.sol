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

contract Proposal {
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

    optional(uint256) _hash;
    uint256 _newhash;
    address _root;
    TransactionBatch _transactions;
    uint128 static _index;
    
    constructor(
        optional(uint256) hash,
        uint256 newhash,
        TransactionBatch transactions
    ) accept {
        _hash = hash;
        _newhash = newhash;
        _root = msg.sender;
        _transactions = transactions;
    }

    
    //Fallback/Receive
    receive() external {
    }
    
    onBounce(TvmSlice body) pure external {
        body;
    }
    
    fallback() external {
    }

    //Getter 
}
