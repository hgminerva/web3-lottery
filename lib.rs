#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod lottery {
    use ink::storage::Mapping;

    /// Lottery error messages
    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum LotteryError {
        AlreadyStarted,
        StartingBlockPassed,
    }

    /// Lottery Setup 
    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq, Default)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct LotterySetup {
        starting_block: u32,
        daily_total_blocks: u32,
        last_draw_number: u32,
        is_started: bool,
    }

    /// Lottery
    #[ink(storage)]
    pub struct Lottery {
        lottery_setup: LotterySetup,
    }

    /// Draw(s)
    #[ink::storage_item]
    pub struct Draw {
        draw_number: u32,
        jackpot: Balance,
        bets: Mapping<AccountId, Balance>,
        is_open: bool,
    }

    impl Lottery {
        /// Constructor
        #[ink(constructor)]
        pub fn new(init_start: bool) -> Self {
            Self { 
                lottery_setup: LotterySetup {
                    starting_block: 0u32,
                    daily_total_blocks: 14_400u32,
                    last_draw_number: 0u32,
                    is_started: init_start, 
                }
            }
        }

        /// Make a default instantiation
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(false)
        }

        /// Start the lottery
        #[ink(message)]
        pub fn start(&mut self) -> Result<(), LotteryError> {
            let current_block = self.env().block_number();

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
        pub fn stop(&mut self) {
            self.lottery_setup.is_started = false;
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
        use ink::env;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let lottery = Lottery::default();
            let lottery_setup = LotterySetup {
				starting_block: 0u32,
                daily_total_blocks: 14_400u32,
                last_draw_number: 032,
                is_started: false,
			};
            assert_eq!(lottery.get_lottery_setup(), lottery_setup);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut lottery = Lottery::new(false);
            let lottery_setup = LotterySetup {
				starting_block: 0u32,
                daily_total_blocks: 14_400u32,
                last_draw_number: 0u32,
                is_started: false,
			};
            assert_eq!(lottery.get_lottery_setup(), lottery_setup);
            let _ = lottery.start();
            let lottery_setup = LotterySetup {
				starting_block: 0u32,
                daily_total_blocks: 14_400u32,
                last_draw_number: 0u32,
                is_started: true,
			};
            assert_eq!(lottery.get_lottery_setup(), lottery_setup);
        }

        #[ink::test]
        fn start_lottery_works() {
            let mut lottery = Lottery::new(false);
            let _ = lottery.start();
            let result = lottery.start();
            assert!(matches!(result, Err(LotteryError::AlreadyStarted)));       
        }    
    }


    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
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
