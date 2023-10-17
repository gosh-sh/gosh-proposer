pragma solidity ^0.8.13;

import {Test, Vm, console, console2} from "forge-std/Test.sol";
import {Elock} from "../src/Elock.sol";

contract ElockFreezeTest is Test {
    Elock public elock;
    address owner = 0x7FA9385bE102ac3EAc297483Dd6233D62b3e1496;

    Vm.Wallet commissionWallet = vm.createWallet("commission");
    Vm.Wallet validator1 = vm.createWallet("validator1");
    Vm.Wallet validator2 = vm.createWallet("validator2");
    Vm.Wallet foreigner = vm.createWallet("foreigner");
    Vm.Wallet user1 = vm.createWallet("user1");
    Vm.Wallet user2 = vm.createWallet("user2");

    uint256 startGlockBlock;

    function setUp() public {
        startGlockBlock = 0x1ace7b2f05e684509d1e93c045eff589aa2c1f27477f9bbdce66b2f0ff8746a1;

        address[] memory validators = new address[](2);
        validators[0] = validator1.addr;
        validators[1] = validator2.addr;

        elock = new Elock(startGlockBlock, payable(commissionWallet.addr), validators);
    }

    function test_ensureUnfreezedByDefault() public {
        assert(_isUnfreezed());

        elock.deposit{value: 1 ether}(100);
        assertEq(elock.totalSupply(), 1 ether);
    }

    function test_voteForFreeze() public {
        vm.skip(true);
        vm.prank(validator1.addr);
        elock.freezeDeposits();
        console.log();

        vm.prank(validator2.addr);
        elock.freezeDeposits();
        // console.log(elock.votedForFreeze(1));

        assert(_isFreezed());
    }

    function _getFreezeStatus() internal view returns (bool) {
        return uint(vm.load(address(elock), "12")) != 0;
    }

    function _isFreezed() internal view returns (bool) {
        return _getFreezeStatus() == true;
    }

    function _isUnfreezed() internal view returns (bool) {
        return _getFreezeStatus() == false;
    }

    // function _dumpVoted() internal view {
    //     address[] memory voted = elock.votedForFreeze();

    //     for (uint256 i = 0; i < voted.length; i++) {
    //         console.log(voted[i]);
    //     }
    // }
}