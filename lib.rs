#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod lottery {

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
        starting_block: u32,
        daily_total_blocks: u16,
        last_draw_number: u32,
        maximum_draws: u8,
        maximum_bets: u16,
        is_started: bool,
    }

    /// Bet
    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Bet {
        bettor: AccountId,
        bet_number: u16,
    }

    /// Winner
    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Winner {
        bettor: AccountId,
        winning_amount: Balance,
    }

    /// Draw meta data 
    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq, Default)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Draw {
        draw_number: u32,
        block_interval: u16,
        jackpot: Balance,
        bets: Vec<Bet>,
        winning_number: u16,
        winners: Vec<Winner>,
        is_open: bool,
    }

    /// Lottery
    #[ink(storage)]
    pub struct Lottery {
        operator: AccountId,
        lottery_setup: LotterySetup,
        draws: Vec<Draw>,
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

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

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

    }


    /// End-to-end (E2E) or integration tests for ink! contracts.
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = LotteryRef::default();

            // When
            let contract_account_id = client
                .instantiate("lottery", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let get = build_message::<LotteryRef>(contract_account_id.clone())
                .call(|lottery| lottery.get());
            let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = LotteryRef::new(false);
            let contract_account_id = client
                .instantiate("lottery", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<LotteryRef>(contract_account_id.clone())
                .call(|lottery| lottery.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = build_message::<LotteryRef>(contract_account_id.clone())
                .call(|lottery| lottery.flip());
            let _flip_result = client
                .call(&ink_e2e::bob(), flip, 0, None)
                .await
                .expect("flip failed");

            // Then
            let get = build_message::<LotteryRef>(contract_account_id.clone())
                .call(|lottery| lottery.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}
