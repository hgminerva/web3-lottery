use sp_runtime::MultiAddress;
use ink::prelude::*;
use ink::env::DefaultEnvironment;

type AccountId = <DefaultEnvironment as ink::env::Environment>::AccountId;
type Balance = <DefaultEnvironment as ink::env::Environment>::Balance;

#[ink::scale_derive(Encode)]
enum RuntimeCall {
    /// Dispatches a call to the `Assets` pallet.
    #[codec(index = 50)]
    Assets(AssetsCall),
}

/// Defines relevant `Assets` pallet calls for vesting management.
#[ink::scale_derive(Encode)]
enum AssetsCall {
    /// Freezes an account’s balance of a specific asset.
    ///
    /// Used to lock tokens during the vesting period.
    #[codec(index = 11)]
    Freeze {
        #[codec(compact)]
        id: u128,
        who: MultiAddress<AccountId, ()>,
    },

    /// Thaws a previously frozen asset balance, unlocking it for transfer.
    ///
    /// Called when vesting is completed.
    #[codec(index = 12)]
    Thaw {
        #[codec(compact)]
        id: u128,
        who: MultiAddress<AccountId, ()>,
    },

    /// Transfers approved assets from an owner to another account.
    ///
    /// Used to transfer vested tokens into the recipient’s frozen balance.
    #[codec(index = 25)]
    TransferApproved {
        #[codec(compact)]
        id: u128,
        owner: MultiAddress<AccountId, ()>,
        destination: MultiAddress<AccountId, ()>,
        #[codec(compact)]
        amount: Balance,
    },
}