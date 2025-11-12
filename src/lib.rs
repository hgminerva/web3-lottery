#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// Unit test
#[cfg(test)]
mod tests;

/// End-to-end test
#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests;

/// pallet_assets runtime calls
pub mod assets;

/// Errors
pub mod errors;

#[ink::contract]
mod lottery {
    use ink::prelude::vec::Vec;
    use crate::errors::Error;
   
    /// Lottery Setup 
    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct LotterySetup {
        pub starting_block: u32,
        pub daily_total_blocks: u16,
        pub last_draw_number: u32,
        pub maximum_draws: u8,
        pub maximum_bets: u16,
        pub is_started: bool,
    }

    /// Bet
    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Bet {
        pub bettor: AccountId,
        pub bet_number: u16,
    }

    /// Winner
    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Winner {
        pub bettor: AccountId,
        pub winning_amount: Balance,
    }

    /// Draw meta data 
    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq, Default)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Draw {
        pub draw_number: u32,
        pub block_interval: u16,
        pub bet_amount: Balance,
        pub jackpot: Balance,
        pub rebate: Balance,
        pub bets: Vec<Bet>,
        pub winning_number: u16,
        pub winners: Vec<Winner>,
        pub is_open: bool,
    }

    /// Lottery
    #[ink(storage)]
    pub struct Lottery {
        pub operator: AccountId,
        pub dev: AccountId,
        pub lottery_setup: LotterySetup,
        pub draws: Vec<Draw>,
    }

    /// Implementation
    impl Lottery {

        /// Constructor
        #[ink(constructor)]
        pub fn new(starting_block: u32,
                   daily_total_blocks: u16,
                   last_draw_number: u32,
                   maximum_draws: u8,
                   maximum_bets: u16,
                   init_start: bool) -> Self 
        {
            let caller = Self::env().caller();
            Self { 
                operator: caller,
                dev: caller,
                lottery_setup: LotterySetup {
                    starting_block: starting_block,
                    daily_total_blocks: daily_total_blocks,
                    last_draw_number: last_draw_number,
                    maximum_draws: maximum_draws,
                    maximum_bets: maximum_bets,
                    is_started: init_start, 
                },
                draws: Vec::new(),
            }
        }

        /// Make a default instantiation
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(0u32,
                      14_400u16,
                      0u32,
                      2u8,
                      1_000u16,
                      false)
        }

        /// Setup the lottery
        /// 
        /// starting_block - determines the starting block of the 14-hour cycle.
        #[ink(message)]
        pub fn setup(&mut self, 
                     starting_block: u32,
                     daily_total_blocks: u16,
                     maximum_draws: u8,
                     maximum_bets: u16) -> Result<(), Error> {

            if self.env().caller() != self.operator {
                return Err(Error::BadOrigin);
            } 

            self.lottery_setup.starting_block = starting_block;
            self.lottery_setup.daily_total_blocks = daily_total_blocks;
            self.lottery_setup.maximum_draws = maximum_draws;
            self.lottery_setup.maximum_bets = maximum_bets;

            Ok(())
        }

        /// Add draw
        #[ink(message)]
        pub fn add_draw(&mut self, block_interval: u16, bet_amount: Balance) -> Result<(), Error>  {
            // Only the operator can add a draw
            if self.env().caller() != self.operator {
                return Err(Error::BadOrigin);
            } 

            // Must not exceed the maximum number of draws setup in the lottery
            if self.draws.len() >= self.lottery_setup.maximum_draws.into() {
                return Err(Error::TooManyDraws);
            }

            let next_draw_number = self.draws
                                            .iter()
                                            .map(|d| d.draw_number)
                                            .max()
                                            .unwrap_or(0)
                                            .saturating_add(1);

            let new_draw = Draw {
                draw_number: next_draw_number,
                block_interval: block_interval,
                bet_amount: bet_amount,
                jackpot: 0,
                rebate: 0,
                bets: Vec::new(),
                winning_number: 0,
                winners: Vec::new(),
                is_open: false,
            };

            self.draws.push(new_draw);

            Ok(())
        }

        #[ink(message)]
        pub fn remove_draw(&mut self) -> Result<(), Error> {
            // Only the operator can add a draw
            if self.env().caller() != self.operator {
                return Err(Error::BadOrigin);
            } 

            // No more draw record
            if self.draws.len() == 0 {
                return Err(Error::NoRecords);
            }

            self.draws.pop();

            Ok(())
        }

        /// Add a bet in a draw
        #[ink(message, payable)]
        pub fn add_bet(&mut self, draw_number: u32, bet_number: u16, affiliate: AccountId) -> Result<(), Error> {
            let caller = self.env().caller();
            let transferred = self.env().transferred_value();

            // Check the draw number exist
            let draw_exists = self.draws.iter().any(|d| d.draw_number == draw_number);
            if !draw_exists {
                return Err(Error::DrawNotFound);
            }

            for draw in &mut self.draws {
                if draw.draw_number == draw_number {
                    // Check if the draw is open
                    if !draw.is_open {
                        return Err(Error::DrawStillClose);
                    }

                    // Check if the transferred amount is equal to the bet amount
                    if transferred != draw.bet_amount {
                        return Err(Error::InvalidBetAmount);
                    }

                    let jackpot_share = transferred * 50 / 100;
                    let rebate_share = transferred * 10 / 100;
                    let operator_share = transferred * 20 / 100;
                    let dev_share = transferred * 10 / 100;
                    let affiliate_share = transferred * 10 / 100;

                    draw.jackpot += jackpot_share;

                    draw.rebate += rebate_share;   
                }
            }

            Ok(())
        }        


        /// Start the lottery
        #[ink(message)]
        pub fn start(&mut self) -> Result<(), Error> {
            let current_block: u32 = self.env().block_number();

            if self.env().caller() != self.operator {
                return Err(Error::BadOrigin);
            } 

            if self.lottery_setup.is_started {
                return Err(Error::AlreadyStarted);
            }

            if current_block > self.lottery_setup.starting_block {
                return Err(Error::StartingBlockPassed);
            }

            self.lottery_setup.is_started = true;

            Ok(())
        }

        /// Stop the lottery
        #[ink(message)]
        pub fn stop(&mut self) -> Result<(), Error> {
            
            if self.env().caller() != self.operator {
                return Err(Error::BadOrigin);
            } 

            self.lottery_setup.is_started = false;

            Ok(())
        }

        /// Returns lottery setup
        #[ink(message)]
        pub fn get_lottery_setup(&self) -> LotterySetup {
            self.lottery_setup.clone()
        }
        
    }

}
