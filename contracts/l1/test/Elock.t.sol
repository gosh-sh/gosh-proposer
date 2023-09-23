pragma solidity ^0.8.13;

import {Test, console2} from "forge-std/Test.sol";
import {Elock} from "../src/Elock.sol";

contract ElockTest is Test {
    Elock public elock;
    address owner = 0x7FA9385bE102ac3EAc297483Dd6233D62b3e1496;
    address user2 = address(0x200);
    address user3 = address(0x300);

    function setUp() public {
        elock = new Elock();
    }

    function test_totalSupply() public {
        assertEq(elock.totalSupply(), 0);

        elock.deposit{value: 1 ether}(100);
        assertEq(elock.totalSupply(), 1 ether);
    }

    function test_trxCount() public {
        assertEq(elock.trxCount(), 0);

        elock.deposit{value: 1 ether}(100);
        assertEq(elock.trxCount(), 1);

        elock.deposit{value: 1 ether}(100);
        assertEq(elock.trxCount(), 2);

        vm.expectRevert();
        payable(address(elock)).transfer(1 ether);
        assertEq(elock.trxCount(), 2);
        assertEq(elock.totalSupply(), 2 ether);

        vm.expectRevert();
        elock.deposit{value: 0 ether}(100);
        assertEq(elock.trxCount(), 2);
        assertEq(elock.totalSupply(), 2 ether);
    }
}
