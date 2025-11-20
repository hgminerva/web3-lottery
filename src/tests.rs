/// Imports all the definitions from the outer scope so we can use them here.
use crate::lottery::{Lottery, LotterySetup, Draw, DrawStatus};
use crate::errors::Error;
use ink::env::test::{default_accounts, set_caller};

/// We test if the default constructor does its job.
#[ink::test]
fn default_works() {
    let accounts = default_accounts::<ink::env::DefaultEnvironment>();
    let lottery = Lottery::default();
    let lottery_setup = LotterySetup {
        operator: accounts.alice,
        dev: accounts.alice,
        asset_id: 1984u128,
        starting_block: 0u32,
        daily_total_blocks: 14_400u32,
        next_starting_block: 0u32,
        maximum_draws: 2u8,
        maximum_bets: 1_000u16,
        is_started: false,
    };
    assert_eq!(lottery.get_lottery_setup(), lottery_setup);
}

#[ink::test]
fn start_lottery_works() {
    let mut lottery = Lottery::new(
                                1984u128,
                                14_400u32,
                                14_400u32,
                                2u8,
                                1_000u16,
                                false
    );
    let _ = lottery.start();
    let result = lottery.start();
    assert!(matches!(result, Err(Error::AlreadyStarted)));       
}    

#[ink::test]
fn setup_lottery_works() {
    let accounts = default_accounts::<ink::env::DefaultEnvironment>();
    set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

    let mut lottery = Lottery::new(1984u128,
                                14_400u32,
                                14_400u32,
                                2u8,
                                1_000u16,
                                false);

    let _ = lottery.setup(
        accounts.alice,
        1984u128,
        14_400u32,
        14_400u32,
        2u8,
        1_000u16,
    );

    let lottery_setup = LotterySetup {
        operator: accounts.alice,
        dev: accounts.alice,
        asset_id: 1984u128,
        starting_block: 14_400u32,
        daily_total_blocks: 14_400u32,
        next_starting_block:28_800u32,
        maximum_draws: 2u8,
        maximum_bets: 1_000u16,
        is_started: true,
    };
    assert_eq!(lottery.get_lottery_setup(), lottery_setup);
    assert_eq!(lottery.lottery_setup.operator, accounts.alice);

    set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
    assert_eq!(
        lottery.setup(
            accounts.alice,
            1984u128,
            14_400u32,
            14_400u32,
            2u8,
            1_000u16,
        ),
        Err(Error::BadOrigin)
    );

}

#[ink::test]
fn adding_and_removing_draw_works() {
    let mut lottery = Lottery::new(
                                1984u128,
                                14_400u32,
                                14_400u32,
                                2u8,
                                1_000u16,
                                false
    );

    let _ = lottery.add_draw(
        1_000u32,
        3_000u32,
        3_500u32,
        500_000,
    );
    
    assert_eq!(lottery.draws.len(), 1);
    
    let new_draw = Draw {
        draw_number: 1,
        opening_blocks: 1_000u32,
        processing_blocks: 3_000u32,
        closing_blocks: 3_500u32,
        bet_amount: 500_000,
        jackpot: 0,
        rebate: 0,
        bets: Vec::new(),
        winning_number: 0,
        winners: Vec::new(),
        status: DrawStatus::Open,
        is_open: false,
    };
    assert_eq!(lottery.draws[0], new_draw);

    let _ = lottery.add_draw(
        1_000u32,
        3_000u32,
        3_500u32,
        500_000,
    );
    assert_eq!(lottery.draws.len(), 2);

    let new_draw = Draw {
        draw_number: 2,
        opening_blocks: 1_000u32,
        processing_blocks: 3_000u32,
        closing_blocks: 3_500u32,
        bet_amount: 500_000,
        jackpot: 0,
        rebate: 0,
        bets: Vec::new(),
        winning_number: 0,
        winners: Vec::new(),
        status: DrawStatus::Open,
        is_open: false,
    };
    assert_eq!(lottery.draws[1], new_draw);

    let _ = lottery.remove_draw();
    assert_eq!(lottery.draws.len(), 1);

    let new_draw = Draw {
        draw_number: 1,
        opening_blocks: 1_000u32,
        processing_blocks: 3_000u32,
        closing_blocks: 3_500u32,
        bet_amount: 500_000,
        jackpot: 0,
        rebate: 0,
        bets: Vec::new(),
        winning_number: 0,
        winners: Vec::new(),
        status: DrawStatus::Open,
        is_open: false,
    };
    assert_eq!(lottery.draws[0], new_draw);
}