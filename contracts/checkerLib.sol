// SPDX-License-Identifier: GPL-3.0-or-later
/*
 * GOSH contracts
 *
 * Copyright (C) 2022 Serhii Horielyshev, GOSH pubkey 0xd060e0375b470815ea99d6bb2890a2a726c5b0579b83c742f5bb70e10a771a04
 */
pragma ever-solidity >=0.66.0;
pragma AbiHeader expire;
pragma AbiHeader pubkey;

import "proposal.sol";

struct BlockData {
    bytes data;
    uint256 hash;
}

struct TransactionBatch {
    uint256 pubkey;
    uint128 value;
    uint256 hash;
}

uint16 constant BATCH_SIZE = 3;

uint16 constant ERR_WRONG_SENDER = 100;
uint16 constant ERR_WRONG_HASH = 101;

library ProposalLib {
    function calculateProposalAddress(TvmCell code, optional(uint256) hash, uint128 index) public returns(address) {
        TvmCell s1 = composeProposalStateInit(code, hash, index);
        return address.makeAddrStd(0, tvm.hash(s1));
    }

     function composeProposalStateInit(TvmCell code, optional(uint256) hash, uint128 index) public returns(TvmCell) {
        TvmCell Proposalcode = buildProposalCode(code, hash);
        TvmCell s1 = tvm.buildStateInit({
            code: Proposalcode,
            contr: Proposal,
            varInit: {_index: index}
        });
        return s1;
    }

    function buildProposalCode(
        TvmCell originalCode,
        optional(uint256) hash
    ) public returns (TvmCell) {
        TvmBuilder b;
        b.store(hash);
        return tvm.setCodeSalt(originalCode, b.toCell());
    }
}