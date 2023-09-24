pragma solidity ^0.8.13;

import {Test, console2} from "forge-std/Test.sol";
import {Elock} from "../src/Elock.sol";

contract ElockTest is Test {
    Elock public elock;
    address owner = 0x7FA9385bE102ac3EAc297483Dd6233D62b3e1496;
    address user2 = address(0x200);
    address user3 = address(0x300);

    function setUp() public {
        elock = new Elock(1);
    }

    function test_totalSupply() public {
        assertEq(elock.totalSupply(), 0);

        elock.deposit{value: 1 ether}(100);
        assertEq(elock.totalSupply(), 1 ether);

        vm.expectRevert();
        payable(address(elock)).transfer(1 ether);
        assertEq(elock.totalSupply(), 1 ether);

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

        vm.expectRevert();
        payable(address(elock)).transfer(1 ether);
        assertEq(elock.trxDepositCount(), 2);

        vm.expectRevert();
        elock.deposit{value: 0 ether}(100);
        assertEq(elock.trxDepositCount(), 2);
    }

    function test_startWithdrawalProposal() public {
        uint256 fromBlock = 1;
        uint256 tillBlock = 2;
        Elock.Transfer memory transfer1 = Elock.Transfer({
            to: payable(address(this)),
            value: 0.5 ether,
            trxHash: 0xbeef
        });
        Elock.Transfer[] memory transfers = new Elock.Transfer[](1);
        transfers[0] = transfer1;
        elock.startWithdrawalProposal(fromBlock, tillBlock, transfers);

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
        elock.startWithdrawalProposal(fromBlock, tillBlock, transfers);
    }
}
