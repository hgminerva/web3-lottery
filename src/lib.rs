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
    use ink::prelude::string::String;
    use scale::{Decode, Encode};
    use ink::env::Error as EnvError;

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
        BetAdded,
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
        // Admin settings
        // --------------
        // Operator of the lottery
        pub operator: AccountId,
        // Developer of the lottery
        pub dev: AccountId,
        // Asset id of the token, e.g., USDT
        pub asset_id: u128,
        
        // Used for off-chain lottery job
        // ------------------------------
        // Starting block of the lottery (this may change of the a daily/cycle basis)
        pub starting_block: u32,
        // Total blocks every day/cycle
        pub daily_total_blocks: u32,
        // Next starting block
        pub next_starting_block: u32,

        // Controls
        // --------
        // Maximum draws
        pub maximum_draws: u8,
        // Maximum bet
        pub maximum_bets: u16,
        // Started
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
        pub bettor: AccountId,
        pub winning_amount: Balance,
    }

    /// Draw meta data 
    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq, Default)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Draw {
        pub draw_number: u32,
        pub block_interval: u16,
        pub bet_amount: u128,
        pub jackpot: u128,
        pub rebate: u128,
        pub bets: Vec<Bet>,
        pub winning_number: u16,
        pub winners: Vec<Winner>,
        pub status: DrawStatus,
        pub is_open: bool,
    }    

    /// Lottery
    #[ink(storage)]
    pub struct Lottery {
        // Lottery Meta-data
        pub lottery_setup: LotterySetup,
        // Multiple draws
        pub draws: Vec<Draw>,
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
                     dev: AccountId,
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

            self.env().emit_event(LotteryEvent {
                operator: self.lottery_setup.operator,
                status: LotteryStatus::EmitSuccess(Success::LotterySetup),
            });
            Ok(())
        }

        /// Start the lottery
        #[ink(message)]
        pub fn start(&mut self) -> Result<(), Error>  {
            let current_block: u32 = self.env().block_number();
            let caller = self.env().caller();

            if caller != self.lottery_setup.operator {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            if self.lottery_setup.is_started {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::AlreadyStarted),
                });
                return Ok(());
            }

            if current_block > self.lottery_setup.starting_block {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::StartingBlockPassed),
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
        #[ink(message)]
        pub fn stop(&mut self) -> Result<(), Error> {
            let caller = self.env().caller();

            if caller != self.lottery_setup.operator {
                self.env().emit_event(LotteryEvent {
                    operator: caller,
                    status: LotteryStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            self.lottery_setup.is_started = false;

            self.env().emit_event(LotteryEvent {
                operator: caller,
                status: LotteryStatus::EmitSuccess(Success::LotteryStopped),
            });
            Ok(())
        }

        /// Lottery draws
        /// -------------
        /// All functions related to draws
        
        /// Add draw
        #[ink(message)]
        pub fn add_draw(&mut self, block_interval: u16, 
            bet_amount: u128) -> Result<(), Error>  {
            let caller = self.env().caller();

            // Only the operator can add a draw
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
                status: DrawStatus::Open,
                is_open: false,
            };

            self.draws.push(new_draw);

            self.env().emit_event(LotteryEvent {
                operator: caller,
                status: LotteryStatus::EmitSuccess(Success::DrawAdded),
            });
            Ok(())
        }

        /// Remove draw
        #[ink(message)]
        pub fn remove_draw(&mut self) -> Result<(), Error> {
            // Only the operator can add a draw
            if self.env().caller() != self.lottery_setup.operator {
                return Err(Error::BadOrigin);
            } 

            // No more draw record
            if self.draws.len() == 0 {
                return Err(Error::NoRecords);
            }

            self.draws.pop();

            Ok(())
        }

        /// Open draw
        #[ink(message)]
        pub fn open_draw(&mut self, draw_number: u32) -> Result<(), Error> {
            // Check if operator
            let caller = self.env().caller();
            if caller != self.lottery_setup.operator {
                return Err(Error::BadOrigin);
            } 

            // Check if draw exist
            let draw_exists = self.draws.iter().any(|d| d.draw_number == draw_number);
            if !draw_exists {
                return Err(Error::DrawNotFound);
            }

            // Open the draw for betting
            for draw in &mut self.draws {
                if draw.draw_number == draw_number {
                    // Check if the draw is close to open
                    if draw.is_open {
                        return Err(Error::DrawOpen);
                    } else {
                        draw.is_open = true;
                        draw.status = DrawStatus::Open;
                    }
                }
            }

            Ok(())
        }

        /// Process draw
        #[ink(message)]
        pub fn process_draw(&mut self, draw_number: u32) -> Result<(), Error> {
            // Check if operator
            let caller = self.env().caller();
            if caller != self.lottery_setup.operator {
                return Err(Error::BadOrigin);
            } 

            // Check if draw exist
            let draw = self.draws.iter()
                .find(|d| d.draw_number == draw_number)
                .ok_or(Error::DrawNotFound)?;

            // Check if draw is open
            if !draw.is_open {
                return Err(Error::DrawClosed);
            }

            // Check if draw status is processing.  We can only process open draws
            if draw.status == DrawStatus::Processing {
                return Err(Error::DrawProcessed);
            }

            // Generate random number
            let seed = self.env().block_timestamp();
            let mut input: Vec<u8> = Vec::new();
            input.extend_from_slice(&seed.to_be_bytes());
            input.extend_from_slice(&draw.draw_number.to_be_bytes());

            let mut output = <hash::Keccak256 as hash::HashOutput>::Type::default();
            ink::env::hash_bytes::<hash::Keccak256>(&input, &mut output);

            let raw = u16::from_le_bytes([output[0], output[1]]);
            let random_num = (raw % 999) + 1;

            // Close the draw (No one can bet anymore)
            let draw = self.draws.iter_mut()
                .find(|d| d.draw_number == draw_number)
                .ok_or(Error::DrawNotFound)?;

            draw.is_open = false;            
            draw.status = DrawStatus::Processing;
            draw.winning_number = random_num;

            Ok(())
        }

        /// Override draw
        #[ink(message)]
        pub fn override_draw(&mut self, draw_number: u32,
            winning_number: u16) -> Result<(), Error> {
            // Check if operator
            // Check if draw exist
            // Check if draw status is Processing (Override is only after random winning number is generated)
            // Change the random winning number

            Ok(())
        }        

        /// Close draw
        #[ink(message)]
        pub fn close_draw(&mut self, draw_number: u32) -> Result<(), Error> {
            // Check if operator
            let caller = self.env().caller();
            if caller != self.lottery_setup.operator {
                return Err(Error::BadOrigin);
            } 

            // Check if draw exist and other statuses
            let draw_exists = self.draws.iter().any(|d| d.draw_number == draw_number);
            if !draw_exists {
                return Err(Error::DrawNotFound);
            } else {
                // Check if draw already closed
                // Check if draw status is Processing
            }

            // Get winners
            
            // Get winner's upline who are also betting

            // If there are winners calculate the prices and upline percentage

            // Distribute the price to the winners

            // Distribute the upline percentage of the price to the upline who bet.
            // If there is no upline it will be given to the operator by default.
            
            // Calculate rebate for all bettors

            // Distribute the rebate to all bettors

            Ok(())
        }

        /// Bets
        /// ----
        /// All functions related to bets.
        
        /// Add a bet
        #[ink(message)]
        pub fn add_bet(&mut self, draw_number: u32, 
            bet_number: u16, 
            bettor: AccountId, 
            upline: AccountId, 
            tx_hash: Vec<u8>) -> Result<(), ContractError> {

            let caller = self.env().caller();

            /// Add bet is called at the server by the operator as soon as tx_hash transfer 
            /// of bet has been verified.
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
                .ok_or(ContractError::Internal(Error::DrawNotFound))?;        /// Logs any message or error in the lottery contract (10 logs max)

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
        /// ----------------
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
