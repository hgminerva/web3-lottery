#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[cfg(test)]
mod tests;

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests;

#[ink::contract]
mod lottery {
    use ink::prelude::vec::Vec; 
    
    /// Lottery error messages
    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum LotteryError {
        AlreadyStarted,
        StartingBlockPassed,
        BadOrigin,
        TooManyDraws,
    }

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
        pub jackpot: Balance,
        pub bets: Vec<Bet>,
        pub winning_number: u16,
        pub winners: Vec<Winner>,
        pub is_open: bool,
    }

    /// Lottery
    #[ink(storage)]
    pub struct Lottery {
        pub operator: AccountId,
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
        #[ink(message)]
        pub fn setup(&mut self, 
                     starting_block: u32,
                     daily_total_blocks: u16,
                     maximum_draws: u8,
                     maximum_bets: u16) -> Result<(), LotteryError> {

            if self.env().caller() != self.operator {
                return Err(LotteryError::BadOrigin);
            } 

            self.lottery_setup.starting_block = starting_block;
            self.lottery_setup.daily_total_blocks = daily_total_blocks;
            self.lottery_setup.maximum_draws = maximum_draws;
            self.lottery_setup.maximum_bets = maximum_bets;

            Ok(())
        }

        /// Add draw
        #[ink(message)]
        pub fn add_draw(&mut self, block_interval: u16) -> Result<(), LotteryError>  {
            // Only the operator can add a draw
            if self.env().caller() != self.operator {
                return Err(LotteryError::BadOrigin);
            } 

            // Must not exceed the maximum number of draws setup in the lottery
            if self.draws.len() >= self.lottery_setup.maximum_draws.into() {
                return Err(LotteryError::TooManyDraws);
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
                jackpot: 0,
                bets: Vec::new(),
                winning_number: 0,
                winners: Vec::new(),
                is_open: false,
            };

            self.draws.push(new_draw);

            Ok(())
        }


        /// Start the lottery
        #[ink(message)]
        pub fn start(&mut self) -> Result<(), LotteryError> {
            let current_block: u32 = self.env().block_number();

            if self.env().caller() != self.operator {
                return Err(LotteryError::BadOrigin);
            } 

            if self.lottery_setup.is_started {
                return Err(LotteryError::AlreadyStarted);
            }

            if current_block > self.lottery_setup.starting_block {
                return Err(LotteryError::StartingBlockPassed);
            }

            self.lottery_setup.is_started = true;

            Ok(())
        }

        /// Stop the lottery
        #[ink(message)]
        pub fn stop(&mut self) -> Result<(), LotteryError> {
            if self.env().caller() != self.operator {
                return Err(LotteryError::BadOrigin);
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
