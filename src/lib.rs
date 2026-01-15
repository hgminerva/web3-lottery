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
    use ink::env::hash;
    use ink::prelude::vec::Vec;

    use crate::errors::{Error, RuntimeError, ContractError};
    use crate::assets::{AssetsCall, RuntimeCall};

    /// Success messages
    #[derive(scale::Encode, scale::Decode, Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Success {
        LotterySetup,
        LotteryStarted,
        LotteryStopped,
        DrawAdded,
        DrawRemoved,
        DrawOpened,
        DrawProcessed,
        DrawClosed,
        BetAdded,
        JackpotAdded,
    }
    
    /// Emit messages
    #[derive(scale::Encode, scale::Decode, Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum LotteryStatus {
        EmitSuccess(Success),
        EmitError(Error),
    }

    /// Contract event emitter
    #[ink(event)]
    pub struct LotteryEvent {
        #[ink(topic)]
        operator: AccountId,
        status: LotteryStatus,
    } 

    /// Draw status
    #[derive(scale::Encode, scale::Decode, Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub enum DrawStatus {
        Open,
        Processing,
        Close,
    }

    impl Default for DrawStatus {
        fn default() -> Self {
            Self::Open
        }
    }

    /// Lottery Setup 
    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct LotterySetup {
        // Admin settings:
        // The operator of the lottery. This account starts and stops the lottery and manages
        // the draws. Through this account, the operator monitors asset transfer hashes sent
        // to the contract and adds bets once they are verified.
        pub operator: AccountId,
        // The developer of the lottery.  This account set up the lottery and the draws tailored
        // to the requirements of the operator.
        pub dev: AccountId,
        // Asset id of the token, e.g., USDT
        pub asset_id: u128,
        // Used for off-chain lottery job:
        // Once this block has been reached the job will start the lottery at the same time
        // calculate the next starting block based on the daily (cycle) total blocks.
        pub starting_block: u32,
        // Total blocks every day/cycle.  Used to calculate the next starting block of the
        // lottery.  In production it must constitute to a 24-hour cycle.
        pub daily_total_blocks: u32,
        // Once all draws has been closed and the lottery is closing, this value will transfer
        // to the starting block and immediately computes for the new value for the nex
        // starting block.
        pub next_starting_block: u32,
        // Maximum draws allowed per lottery
        pub maximum_draws: u8,
        // Maximum bets allowed per draw per lottery
        pub maximum_bets: u16,
        // Starts and stops the lottery
        pub is_started: bool,
    }

    /// Bet
    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Bet {
        pub bettor: AccountId,
        pub upline: AccountId,
        pub bet_number: u16,
        pub tx_hash: Vec<u8>,
    }

    /// Winner
    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Winner {
        pub draw_number: u32,
        pub bettor: AccountId,
        pub upline: AccountId,
        pub bet_number: u16,
        pub tx_hash: Vec<u8>,
        pub bettor_share: u128,
        pub upline_share: u128,
    }

    /// Draw meta data 
    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq, Default)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Draw {
        pub draw_number: u32,
        // Total blocks prior to draw opening from the lottery starting block
        pub opening_blocks: u32,
        // Total blocks before processing from the lottery’s starting block.  
        // The total processing blocks must be greater than the opening blocks.  
        // The difference between the opening blocks and the total processing  
        // blocks determines the time window during which the draw accepts bets.
        pub processing_blocks: u32,
        // Total blocks before closing from the lottery's starting block.
        // The total closing blocks must be greater than the processing blocks.
        // The difference between the processing blocks and the total closing
        // blocks determines the time window during which the draw processed the
        // winners.
        pub closing_blocks: u32,
        // Fixed amount for all bet in the draw.
        pub bet_amount: u128,
        // Total accumulated jackpot 
        pub jackpot: u128,
        // Total accumulated rebate. 10% of the jackpot share will go to the rebate
        pub rebate: u128,
        // Bets
        pub bets: Vec<Bet>,
        // Winning number will be generated during the processed period of the draw.
        pub winning_number: u16,
        // Winners are bets that matches the winning number.
        pub winners: Vec<Winner>,
        // Status of the draw, e.g., Open, Process, Close
        pub status: DrawStatus,
        // True (accepts bets otherwise bets are denied)
        pub is_open: bool,
    }    

    /// Lottery
    #[ink(storage)]
    pub struct Lottery {
        // Lottery Meta-data
        pub lottery_setup: LotterySetup,
        // Multiple draws
        pub draws: Vec<Draw>,
        // Randomizer salt
        pub salt: u64,
    }

    /// Implementation
    impl Lottery {

        /// Lottery setup 
        /// -------------
        /// Setup, start and stop the lottery
        
        /// Constructor
        #[ink(constructor)]
        pub fn new(asset_id: u128,
                   starting_block: u32,
                   daily_total_blocks: u32,
                   maximum_draws: u8,
                   maximum_bets: u16,
                   init_start: bool) -> Self 
        {
            let caller = Self::env().caller();
            Self { 
                lottery_setup: LotterySetup {
                    operator: caller,
                    dev: caller,
                    asset_id: asset_id,
                    starting_block: starting_block,
                    daily_total_blocks: daily_total_blocks,
                    next_starting_block: (starting_block + daily_total_blocks),
                    maximum_draws: maximum_draws,
                    maximum_bets: maximum_bets,
                    is_started: init_start, 
                },
                draws: Vec::new(),
                salt: 0,
            }
        }

        /// Make a default instantiation
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(0u128,
                      0u32,
                      0u32,
                      0u8,
                      0u16,
                      false)
        }

        /// Only the dev can setup the lottery smart contract
        #[ink(message)]
        pub fn setup(&mut self, 
                     operator: AccountId,
                     asset_id: u128,
                     starting_block: u32,
                     daily_total_blocks: u32,
                     maximum_draws: u8,
                     maximum_bets: u16) -> Result<(), Error> {

            // Only the dev (the account that deployed the contract) can change the 
            // lottery setup.  The operator handles the functional activities of the 
            // lottery while the dev handles all technical issues.
            if self.env().caller() != self.lottery_setup.dev {
                self.env().emit_event(LotteryEvent {
                    operator: self.lottery_setup.operator,
                    status: LotteryStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            self.lottery_setup.operator = operator;
            self.lottery_setup.asset_id = asset_id;
            self.lottery_setup.starting_block = starting_block;
            self.lottery_setup.daily_total_blocks = daily_total_blocks;
            self.lottery_setup.next_starting_block = starting_block + daily_total_blocks;
            self.lottery_setup.maximum_draws = maximum_draws;
            self.lottery_setup.maximum_bets = maximum_bets;
            self.lottery_setup.is_started = false;

            self.env().emit_event(LotteryEvent {
                operator: self.lottery_setup.operator,
                status: LotteryStatus::EmitSuccess(Success::LotterySetup),
            });
            Ok(())
        }

        /// Start the lottery
        /// 
        /// 1. Only the operator can start the lottery
        /// 2. The current block must be greater than the starting block
        #[ink(message)]
        pub fn start(&mut self) -> Result<(), Error>  {
            
            // The caller must be the operator
            let caller = self.env().caller();
            if caller != self.lottery_setup.operator {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Check of already started
            if self.lottery_setup.is_started {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::AlreadyStarted),
                });
                return Ok(());
            }

            // Check block
            let current_block: u32 = self.env().block_number();
            if current_block < self.lottery_setup.starting_block {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::InvalidBlock),
                });
                return Ok(());
            }

            self.lottery_setup.is_started = true;

            self.env().emit_event(LotteryEvent {
                operator: caller,
                status: LotteryStatus::EmitSuccess(Success::LotteryStarted),
            });
            Ok(())
        }

        /// Stop the lottery
        /// 
        /// 1. The lottery can be stop only if all draws are closed
        /// 2. Stopping the lottery if the block passes the starting block is also invalid.
        ///    You must correct the setup of the lottery before stopping.
        /// 3. Only the operator can stop the lottery.
        /// 4. 
        #[ink(message)]
        pub fn stop(&mut self) -> Result<(), Error> {

            // Check operator
            let caller = self.env().caller();
            if caller != self.lottery_setup.operator {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Check if all draws are closed
            for draw in self.draws.clone() {
                if draw.is_open || draw.status == DrawStatus::Open {
                    self.env().emit_event(LotteryEvent {
                        operator: caller,
                        status: LotteryStatus::EmitError(Error::DrawOpen),
                    });
                    return Ok(());
                }
            }

            // Check if the current block did not pass the next lottery starting block
            let current_block: u32 = self.env().block_number();
            let next_lottery_starting_block: u32 = self.lottery_setup.next_starting_block;
            if next_lottery_starting_block > current_block  {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::InvalidBlock),
                });
                return Ok(());
            }

            self.lottery_setup.is_started = false;
            self.lottery_setup.starting_block = self.lottery_setup.next_starting_block;
            self.lottery_setup.next_starting_block = self.lottery_setup.next_starting_block + self.lottery_setup.daily_total_blocks;

            self.env().emit_event(LotteryEvent {
                operator: caller,
                status: LotteryStatus::EmitSuccess(Success::LotteryStopped),
            });
            Ok(())
        }

        /// Lottery draws
        /// -------------
        /// All functions related to draws
        
        /// Add draw:
        /// 
        /// 1. Only the operator can add a draw.
        /// 2. The draw can only be added if the lottery is stopped.
        /// 3. It must be important that the following hierarchy of value must be followed.
        ///    lottery.daily_total_blocks > closing_blocks > processing_blocks > opening_blocks
        #[ink(message)]
        pub fn add_draw(&mut self, 
            opening_blocks: u32,
            processing_blocks: u32,
            closing_blocks: u32,
            bet_amount: u128) -> Result<(), Error>  {
            
            // Only the operator can add a draw
            let caller = self.env().caller();      
            if caller != self.lottery_setup.operator {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Must not exceed the maximum number of draws setup in the lottery
            if self.draws.len() >= self.lottery_setup.maximum_draws.into() {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::TooManyDraws),
                });
                return Ok(());
            }

            // Blocks must follow hierarchy order.
            if self.lottery_setup.daily_total_blocks > closing_blocks && 
               closing_blocks > processing_blocks && 
               processing_blocks > opening_blocks {
                // Do nothing and continue
            } else {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::InvalidBlocksHierarchy),
                });
                return Ok(());
            }

            // Check if the lottery is stopped
            if self.lottery_setup.is_started == true {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::AlreadyStarted),
                });
                return Ok(());
            }

            let next_draw_number = self.draws
                                            .iter()
                                            .map(|d| d.draw_number)
                                            .max()
                                            .unwrap_or(0)
                                            .saturating_add(1);

            let new_draw = Draw {
                draw_number: next_draw_number,
                opening_blocks: opening_blocks,
                processing_blocks: processing_blocks,
                closing_blocks: closing_blocks,
                bet_amount: bet_amount,
                jackpot: 0,
                rebate: 0,
                bets: Vec::new(),
                winning_number: 0,
                winners: Vec::new(),
                status: DrawStatus::Close,
                is_open: false,
            };

            self.draws.push(new_draw);

            self.env().emit_event(LotteryEvent {
                operator: caller,
                status: LotteryStatus::EmitSuccess(Success::DrawAdded),
            });
            Ok(())
        }

        /// Remove draw:
        /// 
        /// 1. Only the operator can remove a draw.
        /// 2. The lottery must be stopped before removing a draw.
        /// 3. The removal is last-in-first-out sequence
        #[ink(message)]
        pub fn remove_draw(&mut self) -> Result<(), Error> {
            // Only the operator can add a draw
            let caller = self.env().caller();      
            if caller != self.lottery_setup.operator {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // No more draw record
            if self.draws.len() == 0 {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::NoRecords),
                });
                return Ok(());
            }

            // Check if the lottery is stopped
            if self.lottery_setup.is_started == true {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::AlreadyStarted),
                });
                return Ok(());
            }

            self.draws.pop();

            self.env().emit_event(LotteryEvent {
                operator: caller,
                status: LotteryStatus::EmitSuccess(Success::DrawRemoved),
            });
            Ok(())
        }

        /// Open draw
        /// 
        /// 1. Only the operator can open a draw
        /// 2. The draw status must be close and the is_open flag must be false before
        ///    you can open a draw.
        /// 3. The block number must be greater than the lottery starting block plus the
        ///    draw blocks opening.
        #[ink(message)]
        pub fn open_draw(&mut self, draw_number: u32) -> Result<(), Error> {
            // Only the operator can add a draw
            let caller = self.env().caller();      
            if caller != self.lottery_setup.operator {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Check if draw exist
            let draw = match self.draws.iter().find(|d| d.draw_number == draw_number) {
                Some(d) => d,
                None => {
                    self.env().emit_event(LotteryEvent {
                        operator: caller,
                        status: LotteryStatus::EmitError(Error::DrawNotFound),
                    });
                    return Ok(());
                }
            };

            // The current block must be greater or equal to the draw opening blocks.
            let current_block: u32 = self.env().block_number();
            let draw_opening_blocks: u32 = self.lottery_setup.starting_block + draw.opening_blocks;
            if draw_opening_blocks > current_block  {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::InvalidBlock),
                });
                return Ok(());
            }

            // Open the draw for betting
            for draw in &mut self.draws {
                if draw.draw_number == draw_number {
                    // Check if the draw is close to open
                    if !draw.is_open && draw.status == DrawStatus::Close {
                        draw.is_open = true;
                        draw.status = DrawStatus::Open;
                    } else {
                        self.env().emit_event(LotteryEvent {
                            operator: caller,
                            status: LotteryStatus::EmitError(Error::DrawOpen),
                        });
                        return Ok(());
                    }
                }
            }

            self.env().emit_event(LotteryEvent {
                operator: caller,
                status: LotteryStatus::EmitSuccess(Success::DrawOpened),
            });
            Ok(())
        }

        /// Process draw
        /// 
        /// 1. Processing means that stopping the lottery draw in accepting bets.
        /// 2. At the same time it calculates in random the winning number.
        /// 3. It will also gives the operator the opportunity to override the winning 
        ///    number.
        /// 4. It will also checks of the current block is greater than the sum of the
        ///    lottery starting block and the processing blocks of the draw.
        #[ink(message)]
        pub fn process_draw(&mut self, draw_number: u32) -> Result<(), Error> {
            // Check if operator
            let caller = self.env().caller();
            if caller != self.lottery_setup.operator {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Check if draw exist
            let draw = match self.draws.iter().find(|d| d.draw_number == draw_number) {
                Some(d) => d,
                None => {
                    self.env().emit_event(LotteryEvent {
                        operator: caller,
                        status: LotteryStatus::EmitError(Error::DrawNotFound),
                    });
                    return Ok(());
                }
            };

            // Check if draw is open
            if !draw.is_open {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::DrawClosed),
                });
                return Ok(());
            }

            // Check if draw status is processing.  We can only process open draws
            if draw.status == DrawStatus::Processing {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::DrawProcessing),
                });
                return Ok(());
            }

            // The current block must be greater or equal to the draw processing blocks.
            let current_block: u32 = self.env().block_number();
            let draw_processing_blocks: u32 = self.lottery_setup.starting_block + draw.processing_blocks;
            if draw_processing_blocks > current_block  {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::InvalidBlock),
                });
                return Ok(());
            }

            // Generate random number
            let max_value: u16 = 999;
            let seed = self.env().block_timestamp();

            let mut input: Vec<u8> = Vec::new();
            input.extend_from_slice(&seed.to_be_bytes());
            input.extend_from_slice(&self.salt.to_be_bytes());

            let mut output = <hash::Keccak256 as hash::HashOutput>::Type::default();
            ink::env::hash_bytes::<hash::Keccak256>(&input, &mut output);

            self.salt += 1;

            let raw = u16::from_le_bytes([output[0], output[1]]);
            let random_num: u16 = (raw % max_value) + 1;

            // Close the draw (No one can bet anymore)
            let draw = match self.draws.iter_mut().find(|d| d.draw_number == draw_number) {
                Some(d) => d,
                None => {
                    self.env().emit_event(LotteryEvent {
                        operator: caller,
                        status: LotteryStatus::EmitError(Error::DrawNotFound),
                    });
                    return Ok(());
                }
            };

            draw.is_open = false;            
            draw.status = DrawStatus::Processing;
            draw.winning_number = random_num;

            self.env().emit_event(LotteryEvent {
                operator: caller,
                status: LotteryStatus::EmitSuccess(Success::DrawProcessed),
            });
            Ok(())
        }

        /// Override draw
        /// 
        /// 1. The operator can override the winning number of the draw during the processing period.
        #[ink(message)]
        pub fn override_draw(&mut self, draw_number: u32,
            winning_number: u16) -> Result<(), Error> {

            // Check if operator
            let caller = self.env().caller();
            if caller != self.lottery_setup.operator {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Check if draw exist
            let draw = match self.draws.iter_mut().find(|d| d.draw_number == draw_number) {
                Some(d) => d,
                None => {
                    self.env().emit_event(LotteryEvent {
                        operator: caller,
                        status: LotteryStatus::EmitError(Error::DrawNotFound),
                    });
                    return Ok(());
                }
            };

            // Check if draw status is Processing (Override is only after random winning number is generated)
            if draw.status == DrawStatus::Processing {

                 // Change the random winning number
                draw.winning_number = winning_number;

            } else {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::DrawNotProcessing),
                });
                return Ok(());
            }

            self.env().emit_event(LotteryEvent {
                operator: caller,
                status: LotteryStatus::EmitSuccess(Success::DrawProcessed),
            });
            Ok(())
        }        

        /// Add to the draw's jackpot balance
        /// 
        /// 1. Make sure to transfer the equivalent asset balance to the contract address
        /// 2. Can only be called by the operator
        /// 3. The draw must be closed.
        #[ink(message)]
        pub fn add_draw_jackpot(&mut self, draw_number: u32,
            jackpot: u128) -> Result<(), Error> {

            // Check if operator
            let caller = self.env().caller();
            if caller != self.lottery_setup.operator {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Check if draw exist
            let draw = match self.draws.iter_mut().find(|d| d.draw_number == draw_number) {
                Some(d) => d,
                None => {
                    self.env().emit_event(LotteryEvent {
                        operator: caller,
                        status: LotteryStatus::EmitError(Error::DrawNotFound),
                    });
                    return Ok(());
                }
            };

            // Check if draw status is Close
            if draw.status == DrawStatus::Close {
                // Add the transferred value to the existing jackpot
                draw.jackpot += jackpot;
            } else {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::DrawNotClosed),
                });
                return Ok(());
            }

            self.env().emit_event(LotteryEvent {
                operator: caller,
                status: LotteryStatus::EmitSuccess(Success::JackpotAdded),
            });

            Ok(())
        }

        /// Close draw
        /// 
        /// 1. Only the operator can close the draw.
        /// 2. Only processed draws can be closed.
        /// 3. The block number must be greater than the lottery starting block plus the
        ///    draw blocks closing.
        /// 4. The closing of the draw calls on the following process:
        ///    4.1. Search for the winners
        ///    4.2. Calculate the shares of the jackpot and upline percentage.  Only given
        ///         to upline that bets on the current draw.
        ///    4.3. Transfer the balance to the bettors and its upline who actively bets
        ///    4.4. Update the status of the draw.
        ///    4.5. Delete all bets
        /// 5. During only this period (closing) the app should display the winning number
        #[ink(message)]
        pub fn close_draw(&mut self, draw_number: u32) -> Result<(), ContractError> {

            // Check if operator
            let caller = self.env().caller();
            if caller != self.lottery_setup.operator {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Check if the draw exist
            let draw = match self.draws.iter().find(|d| d.draw_number == draw_number) {
                Some(d) => d,
                None => {
                    self.env().emit_event(LotteryEvent {
                        operator: caller,
                        status: LotteryStatus::EmitError(Error::DrawNotFound),
                    });
                    return Ok(());
                }
            };

            // The current block must be greater or equal to the draw closing blocks.
            let current_block: u32 = self.env().block_number();
            let draw_closing_blocks: u32 = self.lottery_setup.starting_block + draw.opening_blocks;
            if draw_closing_blocks > current_block  {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::InvalidBlock),
                });
                return Ok(());
            }  

            // Get draw for editing
            let draw = match self.draws.iter_mut().find(|d| d.draw_number == draw_number) {
                Some(d) => d,
                None => {
                    self.env().emit_event(LotteryEvent {
                        operator: caller,
                        status: LotteryStatus::EmitError(Error::DrawNotFound),
                    });
                    return Ok(());
                }
            };
            
            // Get the winners
            let mut winners: Vec<Winner> = draw
                .bets
                .iter()
                .filter(|b| b.bet_number == draw.winning_number)
                .map(|b| Winner {
                    draw_number: draw.draw_number,
                    bettor: b.bettor,
                    upline: b.upline,
                    bet_number: b.bet_number,
                    tx_hash: b.tx_hash.clone(),
                    bettor_share: 0,
                    upline_share: 0,
                })
                .collect();         
            
            // Count the number of winners
            let count_winners = winners.len() as u128;

            // Distribute the share of the jackpot to the winners
            if count_winners > 0 {
                let jackpot_share   = draw.jackpot * 90 / 100;
                let upline_share   = draw.jackpot * 10 / 100;

                for w in winners.iter_mut() {
                    w.bettor_share = jackpot_share / count_winners;
                    w.upline_share = upline_share / count_winners;
                }  

                // Save the winners here
                draw.winners = winners;           

                // Drop the mutable draw to start the transfer
                let draw = self.draws.iter()
                    .find(|d| d.draw_number == draw_number)
                    .ok_or(ContractError::Internal(Error::DrawNotFound))?; 

                // Transfer the balances of the winners and the upline
                for winner in draw.winners.iter() {
                    // Winners
                    self.env()
                        .call_runtime(&RuntimeCall::Assets(AssetsCall::Transfer {
                            id: self.lottery_setup.asset_id,
                            target: winner.bettor.into(),
                            amount: winner.bettor_share,
                        }))
                        .map_err(|_| RuntimeError::CallRuntimeFailed)?;                

                    // Upline
                    if draw.bets.iter().find(|b| b.bettor == winner.upline).is_none() {
                        // If the upline is not actively betting the share will go to the operator
                        self.env()
                            .call_runtime(&RuntimeCall::Assets(AssetsCall::Transfer {
                                id: self.lottery_setup.asset_id,
                                target: self.lottery_setup.operator.into(),
                                amount: winner.upline_share,
                            }))
                            .map_err(|_| RuntimeError::CallRuntimeFailed)?;    
                    } else {
                        // If the upline is actively betting
                        self.env()
                            .call_runtime(&RuntimeCall::Assets(AssetsCall::Transfer {
                                id: self.lottery_setup.asset_id,
                                target: winner.upline.into(),
                                amount: winner.upline_share,
                            }))
                            .map_err(|_| RuntimeError::CallRuntimeFailed)?;       
                    }
                } 
            } else {
                // If there are no winners in the current draw make sure to clean up the winner array
                draw.winners = Vec::new();
            }

            // Distribute the shares of the rebate to the bettors.
            //
            // Drop the mutable draw to start the transfer
            let draw = self.draws.iter()
                .find(|d| d.draw_number == draw_number)
                .ok_or(ContractError::Internal(Error::DrawNotFound))?;             

            // Count the bettors
            let count_bettors = draw.bets.len() as u128;

            if count_bettors > 0 {
                // Rebate share per bet
                let bettor_share = draw.rebate / count_bettors;

                for bet in draw.bets.iter() {
                    // Bettors
                    self.env()
                        .call_runtime(&RuntimeCall::Assets(AssetsCall::Transfer {
                            id: self.lottery_setup.asset_id,
                            target: bet.bettor.into(),
                            amount: bettor_share,
                        }))
                        .map_err(|_| RuntimeError::CallRuntimeFailed)?;   
                }
            }

            // Change the status of the draw from open to close
            let draw = match self.draws.iter_mut().find(|d| d.draw_number == draw_number) {
                Some(d) => d,
                None => {
                    self.env().emit_event(LotteryEvent {
                        operator: caller,
                        status: LotteryStatus::EmitError(Error::DrawNotFound),
                    });
                    return Ok(());
                }
            };

            // Clean the jackpot after we distribute it to the winners of the current draw
            if draw.winners.len() > 0 {
                draw.jackpot = 0;
            }
            // All rebate will be distributed to all bettors as we close the draw 
            draw.rebate = 0;
            // Clean up the bets
            draw.bets = Vec::new();
            // Close the draw
            draw.status = DrawStatus::Close;
            draw.is_open = false;

            self.env().emit_event(LotteryEvent {
                operator: caller,
                status: LotteryStatus::EmitSuccess(Success::DrawClosed),
            });
            Ok(())

        }

        /// Bets
        /// ----
        /// All functions related to bets.
        
        /// Add a bet
        /// 
        /// 1. Anyone can place a bet on an open draw
        /// 2. Upon betting the bet amount is already distributed and transferred to the following:
        ///    2.1. 50% will go to the jackpot where it will be split into the following:
        ///         2.1.1. Jackpot share is 90%
        ///         2.1.2. Upline share of the jackpot is 10%
        ///    2.2. 20% will go to the operator
        ///    2.3. 10% will go to the developer
        ///    2.4. 10% will go to the rebate (all bettors)
        ///    2.5. 10% will go to the affiliate (immediately the active upline will get 10%)
        #[ink(message)]
        pub fn add_bet(&mut self, draw_number: u32, 
            bet_number: u16, 
            bettor: AccountId, 
            upline: AccountId, 
            tx_hash: Vec<u8>) -> Result<(), ContractError> {

            let caller = self.env().caller();

            // Add bet is called at the server by the operator as soon as tx_hash transfer 
            // of bet has been verified.
            if caller != self.lottery_setup.operator {
                self.env().emit_event(LotteryEvent {
                    operator: self.lottery_setup.operator,
                    status: LotteryStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Find the draw number
            let draw = self.draws.iter()
                .find(|d| d.draw_number == draw_number)
                .ok_or(ContractError::Internal(Error::DrawNotFound))?;        

            // A draw that the status is not open and the flag is false is considered close draw.
            if draw.status != DrawStatus::Open && !draw.is_open {
                self.env().emit_event(LotteryEvent {
                    operator: self.lottery_setup.operator,
                    status: LotteryStatus::EmitError(Error::DrawClosed),
                });
                return Ok(());
            }

            // Shares
            let jackpot_share   = draw.bet_amount * 50 / 100;
            let dev_share       = draw.bet_amount * 10 / 100;
            let operator_share  = draw.bet_amount * 20 / 100;
            let rebate_share    = draw.bet_amount * 10 / 100;
            let affiliate_share = draw.bet_amount * 10 / 100;

            // Transfer operator's share
            self.env()
                .call_runtime(&RuntimeCall::Assets(AssetsCall::Transfer {
                    id: self.lottery_setup.asset_id,
                    target: self.lottery_setup.operator.into(),
                    amount: operator_share,
                }))
                .map_err(|_| RuntimeError::CallRuntimeFailed)?;

            // Transfer dev's share
            self.env()
                .call_runtime(&RuntimeCall::Assets(AssetsCall::Transfer {
                    id: self.lottery_setup.asset_id,
                    target: self.lottery_setup.dev.into(),
                    amount: dev_share,
                }))
                .map_err(|_| RuntimeError::CallRuntimeFailed)?;


            // Transfer affiliate share.
            // This will require that the affiliate upline already betted, if not
            // the share will be sent to the operator.
            let mut upline_found: Option<AccountId> = None;

            for b in &draw.bets {
                if b.bettor == upline {
                    upline_found = Some(b.bettor);
                    break;
                }
            }

            match upline_found {
                Some(valid_upline) => {
                    // Upline exists, send affiliate share to the upline
                    self.env()
                        .call_runtime(&RuntimeCall::Assets(AssetsCall::Transfer {
                            id: self.lottery_setup.asset_id,
                            target: valid_upline.into(),
                            amount: affiliate_share,
                        }))
                        .map_err(|_| RuntimeError::CallRuntimeFailed)?;
                }
                None => {
                    // Upline not found, send affiliate share to the operator
                    self.env()
                        .call_runtime(&RuntimeCall::Assets(AssetsCall::Transfer {
                            id: self.lottery_setup.asset_id,
                            target: self.lottery_setup.operator.into(),
                            amount: affiliate_share,
                        }))
                        .map_err(|_| RuntimeError::CallRuntimeFailed)?;
                }
            };

            // Add the bet
            let draw = self.draws.iter_mut()
                .find(|d| d.draw_number == draw_number)
                .ok_or(ContractError::Internal(Error::DrawNotFound))?;
            
            let new_bet = Bet {
                bettor: bettor,
                upline: upline,
                bet_number: bet_number,
                tx_hash: tx_hash,
            };
            
            draw.bets.push(new_bet);

            // Compute for jackpot and rebate, these shares are distributed during closing 
            // 1. jackpot are given to the winners in equal shares
            // 2. rebate are given to all bettors in equal shares 
            draw.jackpot += jackpot_share;
            draw.rebate += rebate_share; 

            self.env().emit_event(LotteryEvent {
                operator: self.lottery_setup.operator,
                status: LotteryStatus::EmitSuccess(Success::BetAdded),
            });

            Ok(())
        }        

        /// Getter functions
        /// 
        /// These functions returns storage data 

        /// Returns lottery setup
        #[ink(message)]
        pub fn get_lottery_setup(&self) -> LotterySetup {
            self.lottery_setup.clone()
        }

        /// Return all the draws
        #[ink(message)]
        pub fn get_draws(&self) -> Vec<Draw> {
            self.draws.clone()
        }

        /// Return all the bets
        #[ink(message)]
        pub fn get_bets(&self, draw_number:u32) -> Vec<Bet> {
            self.draws
                .iter()
                .find(|d| d.draw_number == draw_number)
                .map(|d| d.bets.clone())
                .unwrap_or_default()
        }
        
    }

}
