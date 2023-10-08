macro_rules! abi {
    ($file: expr) => {
        // NOTE: Run `make copy_abi` in case of `No such file...` error
        include_str!($file)
    };
}

pub static CHECKER_ABI: &str = abi!("../../../contracts/l2/checker.abi.json");
pub static PROPOSAL_ABI: &str = abi!("../../../contracts/l2/proposal.abi.json");
pub static ROOT_ABI: &str = abi!("../../../contracts/l2/RootTokenContract.abi");
pub static TOKEN_WALLET_ABI: &str = abi!("../../../contracts/l2/TONTokenWallet.abi");
pub static ELOCK_ABI: &str = abi!("../../../resources/elock.abi.json");
pub static ELOCK_IDS: &str = abi!("../../../resources/identifiers.json");
