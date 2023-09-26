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
    uint256 startGlockBlock = 1;

    function setUp() public {
        address[] memory validators = new address[](2);
        validators[0] = owner;
        validators[1] = validator1;
        elock = new Elock(startGlockBlock, validators);
    }

    function test_constructorRun() public {
        assertEq(elock.lastProcessedBlock(), startGlockBlock);
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

    function test_proposeWithdrawal() public {
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

        vm.expectRevert("Proposal already exists");
        elock.proposeWithdrawal(fromBlock, tillBlock, transfers);
        proposalKeys = elock.getProposalList();
        assertEq(proposalKeys.length, 1);

        vm.prank(address(0xdeadbeef));
        vm.expectRevert(Elock.Unauthorized.selector);
        elock.proposeWithdrawal(fromBlock, tillBlock, transfers);
    }

    function test_voteForWithdrawal() public {
        assertTrue(false, "todo!");
    }

    function test_proposeChangeValidators() public {
        address[] memory currentValidators = elock.getValidators();
        assertEq(currentValidators.length, 2);

        address[] memory newValidators = new address[](2);
        newValidators[0] = validator1;
        newValidators[1] = validator2;
        elock.proposeChangeValidators(newValidators);
        address[] memory proposedValidators = elock.getProposedValidators();
        for (uint256 i = 0; i < proposedValidators.length; i++) {
            console2.log(proposedValidators[i]);
        }
        assertTrue(false, "todo!");
    }

    function test_voteForChangeValidators() public {
        assertTrue(false, "todo!");
    }
}
