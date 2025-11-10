/// Imports all the definitions from the outer scope so we can use them here.
use super::*;
use crate::lottery::{Lottery, LotterySetup, LotteryError, Draw};

/// We test if the default constructor does its job.
#[ink::test]
fn default_works() {
    let lottery = Lottery::default();
    let lottery_setup = LotterySetup {
        starting_block: 0u32,
        daily_total_blocks: 14_400u16,
        last_draw_number: 0u32,
        maximum_draws: 2u8,
        maximum_bets: 1_000u16,
        is_started: false,
    };
    assert_eq!(lottery.get_lottery_setup(), lottery_setup);
}

#[ink::test]
fn start_lottery_works() {
    let mut lottery = Lottery::new(0u32,
                                14_400u16,
                                0u32,
                                2u8,
                                1_000u16,
                                false);
    let _ = lottery.start();
    let result = lottery.start();
    assert!(matches!(result, Err(LotteryError::AlreadyStarted)));       
}    

#[ink::test]
fn setup_lottery_works() {
    let mut lottery = Lottery::new(0u32,
                                14_400u16,
                                0u32,
                                2u8,
                                1_000u16,
                                false);
    let _ = lottery.start();
    let _ = lottery.setup(1_000_000u32, 14_400u16, 2u8, 1_000u16);

    let lottery_setup = LotterySetup {
        starting_block: 1_000_000u32,
        daily_total_blocks: 14_400u16,
        last_draw_number: 0u32,
        maximum_draws: 2u8,
        maximum_bets: 1_000u16,
        is_started: true,
    };
    assert_eq!(lottery.get_lottery_setup(), lottery_setup);
}

#[ink::test]
fn adding_draw_works() {
    let mut lottery = Lottery::new(0u32,
                                14_400u16,
                                0u32,
                                2u8,
                                1_000u16,
                                false);
    
    let _ = lottery.add_draw(1_000u16);
    assert_eq!(lottery.draws.len(), 1);
    
    let new_draw = Draw {
        draw_number: 1,
        block_interval: 1_000u16,
        jackpot: 0,
        bets: Vec::new(),
        winning_number: 0,
        winners: Vec::new(),
        is_open: false,
    };
    assert_eq!(lottery.draws[0], new_draw);
}