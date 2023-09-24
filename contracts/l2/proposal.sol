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
import "checker.sol";

contract Proposal {

    string constant _version = "1.0.0";

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

    uint256 _hash;
    uint256 _newhash;
    address static _root;
    TransactionBatch[] _transactions;
    uint128 static _index;
    uint128 _need;
    mapping(uint16 => TvmSlice) _vdict;
    
    constructor(
        uint256 hash,
        uint256 newhash,
        TransactionBatch[] transactions
    ) accept {
        _hash = hash;
        _newhash = newhash;
        require (_root == msg.sender, ERR_WRONG_SENDER);
        _transactions = transactions;
        (optional(TvmCell) data) = tvm.rawConfigParam(34);
        ValidatorSet vset = data.get().toSlice().load(ValidatorSet);
        _vdict = vset.vdict;
        _need =  uint128(_vdict.keys().length);
        _need = _need - _need * 34 / 100;
        if (data.hasValue() == false) { selfdestruct(_root); }
    }

    function setVote(uint16 id) public {
        if (_vdict.exists(id)) {
            TvmSlice data = _vdict[id];
            data.skip(4 * 8);
            uint256 pub = data.load(uint256);
            require(pub == msg.pubkey(), ERR_WRONG_SENDER);
            tvm.accept();
            optional(TvmSlice) deleted = _vdict.getDel(id);
            deleted;
            _need -= 1;
            if (_need == 0) {
                Checker(_root).setNewHash{value: 0.1 ton, flag: 1}(_hash, _newhash, _index, _transactions);
            }
        }
    }

    function destroy() public senderIs(_root) accept {
        selfdestruct(_root);
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
    function getDetails() external view returns(uint256 hash, uint256 newhash, TransactionBatch[] transactions, uint128 index, uint128 need){
        return (_hash, _newhash, _transactions, _index, _need);
    }

    function getVersion() external pure returns(string, string) {
        return ("proposal", _version);
    }

    function getSet() external view returns (mapping(uint16 => uint256)) {
        uint16 key;
        optional(uint16, TvmSlice) res = _vdict.next(key);
        mapping(uint16 => uint256) result;
        while (res.hasValue()) {
            (uint16 newkey, TvmSlice data) = res.get();
            data.skip(4 * 8);
            uint256 pub = data.load(uint256);
            result[newkey] = pub;
            res = _vdict.next(newkey);
        }
        return result;
    }
}
