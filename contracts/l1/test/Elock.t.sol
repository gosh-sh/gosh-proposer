pragma solidity ^0.8.13;

import {Test, console2} from "forge-std/Test.sol";
import {Elock} from "../src/Elock.sol";

contract ElockTest is Test {
    Elock public elock;
    address owner = 0x7FA9385bE102ac3EAc297483Dd6233D62b3e1496;
    address validator1 = 0xe0cAd8f1Deee00329A2437fCe982b90Dc9e03abd;
    address validator2 = 0xA2Cd57002cD089b7166ad40Bb1402664afc64067;
    address foreign = address(0xdeadbeef);
    address user2 = address(0x200);
    address user3 = address(0x300);
    uint256 startGlockBlock = 0x1ace7b2f05e684509d1e93c045eff589aa2c1f27477f9bbdce66b2f0ff8746a1;

    event WithdrawRejected(uint256 indexed proposalKey, address indexed voter, uint8 reason);
    event WithdrawExecuted(uint256 indexed proposalKey);
    event Withdrawal(address indexed recepient, uint256 value);

    function setUp() public {
        address[] memory validators = new address[](2);
        validators[0] = owner;
        validators[1] = validator1;
        elock = new Elock(startGlockBlock, validators);
    }

    function test_constructorRun() public {
        assertEq(elock.lastProcessedL2Block(), startGlockBlock);
        address[] memory validators = elock.getValidators();
        assertEq(validators.length, 2);
        assertEq(validators[0], owner);
        assertEq(validators[1], validator1);
    }

    function test_totalSupply() public {
        assertEq(elock.totalSupply(), 0);

        elock.deposit{value: 1 ether}(100);
        assertEq(elock.totalSupply(), 1 ether);

        payable(address(elock)).transfer(1 ether);
        assertEq(elock.totalSupply(), 1 ether);
        assertEq(address(elock).balance, 2 ether);

        vm.expectRevert();
        elock.deposit{value: 0 ether}(100);
        assertEq(elock.totalSupply(), 1 ether);
    }

    function test_trxDepositCount() public {
        assertEq(elock.trxDepositCount(), 0);

        elock.deposit{value: 1 ether}(100);
        assertEq(elock.trxDepositCount(), 1);

        elock.deposit{value: 1 ether}(100);
        assertEq(elock.trxDepositCount(), 2);

        payable(address(elock)).transfer(1 ether);
        assertEq(elock.trxDepositCount(), 2);

        vm.expectRevert();
        elock.deposit{value: 0 ether}(100);
        assertEq(elock.trxDepositCount(), 2);
    }

    function test_proposeWithdrawal_creation() public {
        uint256 fromBlock = 1;
        uint256 tillBlock = 2;
        Elock.Transfer memory transfer1 = Elock.Transfer({
            to: payable(address(this)),
            value: 0.5 ether,
            trxHash: 0xbeef
        });
        Elock.Transfer[] memory transfers = new Elock.Transfer[](1);
        transfers[0] = transfer1;
        elock.proposeWithdrawal(fromBlock, tillBlock, transfers);

        uint256[] memory proposalKeys = elock.getProposalList();
        assertEq(proposalKeys.length, 1);
        uint256 BlockA;
        uint256 BlockB;
        Elock.Transfer[] memory _transfers;
        (BlockA, BlockB, _transfers) = elock.getProposal(proposalKeys[0]);
        assertEq(BlockA, fromBlock);
        assertEq(BlockB, tillBlock);
        assertEq(_transfers.length, 1);
    }

    function test_proposeWithdrawal_1() public {
        uint blockA = elock.lastProcessedL2Block();
        uint blockB1 = 0xb10cb1;
        uint blockB2 = 0xb10cb2;

        Elock.Transfer[] memory transfers = create_transfers(4);
        Elock.Transfer[] memory transfers1 = new Elock.Transfer[](1);
        transfers1[0] = transfers[0];
        Elock.Transfer[] memory transfers2 = new Elock.Transfer[](4);
        transfers2 = transfers;

        // propose 1st withdrawals
        elock.proposeWithdrawal(blockA, blockB1, transfers1);
        uint256[] memory proposalKeys = elock.getProposalList();
        assertEq(proposalKeys.length, 1);

        // propose 2nd withdrawals
        elock.proposeWithdrawal(blockA, blockB2, transfers2);
        proposalKeys = elock.getProposalList();
        assertEq(proposalKeys.length, 2);

        // ensure that aren't votes yet
        uint proposalKey = proposalKeys[1];
        assertEq(elock.getVotesForWithdrawal(proposalKey), 0);

        // vote for 2nd proposal as `validator1`
        vm.prank(validator1);
        elock.voteForWithdrawal(proposalKey);
        assertEq(elock.getVotesForWithdrawal(proposalKey), 1);

        // ensure that voting for 2nd proposal as `validator1` will be failed
        // vm.expectRevert("Already voted");
        vm.prank(validator1);
        elock.voteForWithdrawal(proposalKey);
        assertEq(elock.getVotesForWithdrawal(proposalKey), 1);

        // vote for 2nd proposal as `owner`
        vm.expectEmit(true, true, false, true);
        emit WithdrawRejected(proposalKey, owner, 3);
        elock.voteForWithdrawal(proposalKey);
        // vote counted but withdraws aren't executed because not enough funds
        assertEq(elock.getVotesForWithdrawal(proposalKey), 2);
        assertEq(elock.trxWithdrawCount(), 0);

        elock.deposit{value: 10 ether}(uint(0xdeadbeef));
        assertEq(elock.totalSupply(), 10 ether);

        // finalization
        for (uint256 i = 0; i < transfers.length; i++) {
            // vm.expectEmit();
            // emit Withdrawal(transfers[i].to, transfers[i].value);
        }
        vm.expectEmit(true, false, false, true);
        emit WithdrawExecuted(proposalKey);

        elock.voteForWithdrawal(proposalKey);

        assertEq(elock.trxWithdrawCount(), 4);
        assertEq(elock.getVotesForWithdrawal(proposalKey), 0); // withdrawal was executed, all propsals were deleted
        assertEq(elock.getProposalList().length, 0);
        assertEq(elock.lastProcessedL2Block(), blockB2);
        uint totalWithdrawn;
        for (uint256 i = 0; i < transfers.length; i++) {
            totalWithdrawn += transfers[i].value;
        }
        assertEq(elock.totalSupply() + totalWithdrawn, 10 ether);

    }

    // function test_voteForWithdrawal() public {
    //     assertTrue(false, "todo!");
    // }

    // function test_proposeChangeValidators() public {
    //     address[] memory currentValidators = elock.getValidators();
    //     assertEq(currentValidators.length, 2);

    //     address[] memory newValidators = new address[](2);
    //     newValidators[0] = validator1;
    //     newValidators[1] = validator2;
    //     elock.proposeChangeValidators(newValidators);
    //     address[] memory proposedValidators = elock.getProposedValidators();
    //     for (uint256 i = 0; i < proposedValidators.length; i++) {
    //         console2.log(proposedValidators[i]);
    //     }
    //     assertTrue(false, "todo!");
    // }

    // function test_voteForChangeValidators() public {
    //     assertTrue(false, "todo!");
    // }


    function create_transfers(uint count)
        private view
        returns (Elock.Transfer[] memory transfers)
    {
        address payable[5] memory users = [
            payable(owner/* address(0xd00d1) */),
            payable(validator1/* address(0xd00d2) */),
            payable(validator2/* address(0xd00d3) */),
            payable(foreign/* address(0xd00d4) */),
            payable(address(0xd00d5))
        ];
        uint256[5] memory values = [
            uint256(0xff00000f00d1),
            0xff00000f00d2,
            0xff00000f00d3,
            0xff00000f00d4,
            0xff00000f00d5
        ];

        uint256[5] memory txn = [
            uint256(0xbee1),
            0xbee2,
            0xbee3,
            0xbee4,
            0xbee5
        ];

        if (count > users.length) {
            count = users.length;
        }

        transfers = new Elock.Transfer[](count);
        for (uint256 i = 0; i < count; i++) {
            transfers[i] = Elock.Transfer(users[i], values[i], txn[i]);
        }

        return transfers;
    }
}
