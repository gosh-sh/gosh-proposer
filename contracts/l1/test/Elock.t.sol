pragma solidity ^0.8.13;

import {Test, Vm, console, console2} from "forge-std/Test.sol";
import {Elock} from "../src/Elock.sol";

contract ElockTest is Test {
    Elock public elock;
    address owner = 0x7FA9385bE102ac3EAc297483Dd6233D62b3e1496;

    Vm.Wallet commissionWallet = vm.createWallet("commission");
    Vm.Wallet validator1 = vm.createWallet("validator1");
    Vm.Wallet validator2 = vm.createWallet("validator2");
    Vm.Wallet foreigner = vm.createWallet("foreigner");
    Vm.Wallet user0 = vm.createWallet("user0");
    Vm.Wallet user1 = vm.createWallet("user1");
    Vm.Wallet user2 = vm.createWallet("user2");

    uint256 startGlockBlock;
    uint256 gasPrice;

    address constant ETH = address(0);

    event Deposited(address indexed token, address from, uint256 pubkey, uint256 value);
    event Withdrawal(address indexed token, address indexed to, uint256 value, uint256 commission);
    event WithdrawExecuted(uint256 indexed proposalKey);
    event WithdrawRejected(uint256 indexed proposalKey, address indexed voter, uint8 reason);

    function setUp() public {
        startGlockBlock = 0x1ace7b2f05e684509d1e93c045eff589aa2c1f27477f9bbdce66b2f0ff8746a1;
        gasPrice = tx.gasprice;

        address[] memory validators = new address[](2);
        validators[0] = validator1.addr;
        validators[1] = validator2.addr;

        elock = new Elock(startGlockBlock, payable(commissionWallet.addr), validators);
    }

    function test_constructorRun() public {
        assertEq(elock.lastProcessedL2Block(), startGlockBlock);
        address[] memory validators = elock.getValidators();
        assertEq(validators.length, 2);
        assertEq(validators[0], validator1.addr);
        assertEq(validators[1], validator2.addr);
    }

    function test_depositedEth() public {
        assertEq(elock.totalSupply(), 0);

        vm.expectEmit(true, true, true, true);
        emit Deposited(address(0), owner, 100, 1 ether);
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
        uint256 fromBlock = elock.lastProcessedL2Block();
        uint256 tillBlock = 2;
        Elock.Transfer memory transfer1 = Elock.Transfer({
            token: address(0),
            to: payable(address(this)),
            value: 0.5 ether,
            trxHash: 0xbeef
        });
        Elock.Transfer[] memory transfers = new Elock.Transfer[](1);
        transfers[0] = transfer1;

        vm.startPrank(validator1.addr);
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

        vm.deal(user0.addr, 10 ether);
        uint user_deposit = 1 ether;
        uint user_goshPubkey = uint(0xdeadbeef);

        vm.prank(user0.addr);
        elock.deposit{value: user_deposit}(user_goshPubkey);
        assertEq(elock.totalSupply(), 1 ether);

        // propose withdrawal
        Elock.Transfer[] memory transfers = new Elock.Transfer[](1);
        transfers[0] = Elock.Transfer(
            address(0),
            payable(user2.addr),
            user_deposit,
            user_goshPubkey
        );
        vm.prank(validator1.addr);
        elock.proposeWithdrawal(blockA, blockB1, transfers);
        uint256[] memory proposalKeys = elock.getProposalList();
        assertEq(proposalKeys.length, 1);

        // ensure that aren't votes yet
        uint proposalKey = proposalKeys[0];
        assertEq(elock.getVotesForWithdrawal(proposalKey), 0);

        // vote for proposal as `validator2`
        vm.prank(validator2.addr);
        elock.voteForWithdrawal(proposalKey);
        assertEq(elock.getVotesForWithdrawal(proposalKey), 1);

        // ensure that voting for proposal as `validator2` will be failed
        vm.prank(validator2.addr);
        elock.voteForWithdrawal(proposalKey);
        assertEq(elock.getVotesForWithdrawal(proposalKey), 1);

        // user balance before withdrawal
        assertEq(address(user2.addr).balance, 0);

        uint expectedCommission = calcExpectedCommission(transfers);
        vm.expectEmit(true, true, false, true);
        emit Withdrawal(address(0), user2.addr, user_deposit - expectedCommission, expectedCommission);
        vm.expectEmit(true, true, false, false);
        emit WithdrawExecuted(proposalKey);

        // vote for proposal as `validator1`
        vm.prank(validator1.addr);
        elock.voteForWithdrawal(proposalKey);
        assertEq(elock.getVotesForWithdrawal(proposalKey), 0);
        assertEq(elock.getProposalList().length, 0);
        assertEq(elock.trxWithdrawCount(), 1);
        assertEq(elock.totalSupply(), 0);

        // the cost of all transfers in the proposal
        uint gasCorrection = transfers.length * 21_000 * gasPrice;
        uint collectedCommission = elockCollectedCommission();
        assertEq(collectedCommission, 400_000 * elock.gasPrice());
        assertEq(address(elock).balance - gasCorrection, collectedCommission);
        // user balance after withdrawal
        assertEq(address(user2.addr).balance, user_deposit - collectedCommission - gasCorrection);
    }

    function test_proposeWithdrawal_2() public {
        uint blockA = elock.lastProcessedL2Block();
        uint blockB = 0xb10cB;
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
    function elockCollectedCommission() internal view returns (uint) {
        bytes32 comm = vm.load(address(elock), bytes32(uint(0x13)));
        return uint(comm);
    }

    function calcExpectedCommission(Elock.Transfer[] memory transfers)
        internal view
        returns (uint)
    {
        return transfers.length * 21_000 * gasPrice + 400_000 * gasPrice;
    }

    function create_transfers(uint count)
        private view
        returns (Elock.Transfer[] memory transfers)
    {
        address payable[5] memory users = [
            payable(owner),
            payable(user1.addr),
            payable(user2.addr),
            payable(foreigner.addr),
            payable(address(0xd00d5))
        ];
        uint256[5] memory values = [
            uint256(1 ether),
            uint256(1 ether),
            uint256(1 ether),
            uint256(1 ether),
            uint256(1 ether)
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
            transfers[i] = Elock.Transfer(address(0), users[i], values[i], txn[i]);
        }

        return transfers;
    }

    function dump_logs() internal {
        Vm.Log[] memory logs = vm.getRecordedLogs();

        for (uint256 i = 0; i < logs.length; i++) {
            for (uint256 j = 0; j < logs[i].topics.length; j++) {
                console.logBytes32(logs[i].topics[j]);
            }
            console.logBytes(logs[i].data);
        }
    }
}
