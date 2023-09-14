#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod xcm_execute {
    // use xcm::VersionedXcm;
    use xcm::{
        v3::prelude::*,
        VersionedXcm,
    };

    use ink::{
        env::Error as EnvError,
        prelude::*,
    };

    /// A trivial contract with a single message, that uses `call-runtime` API for
    /// performing native token transfer.
    #[ink(storage)]
    #[derive(Default)]
    pub struct XcmExecute;

    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum RuntimeError {
        CallRuntimeFailed,
    }

    impl From<EnvError> for RuntimeError {
        fn from(e: EnvError) -> Self {
            match e {
                EnvError::CallRuntimeFailed => RuntimeError::CallRuntimeFailed,
                _ => panic!("Unexpected error from `pallet-contracts`."),
            }
        }
    }

    impl XcmExecute {
        /// The constructor is `payable`, so that during instantiation it can be given
        /// some tokens that will be further transferred when transferring funds through
        /// XCM.
        #[ink(constructor, payable)]
        pub fn new() -> Self {
            Default::default()
        }

        /// Tries to transfer `value` from the contract's balance to `receiver`.
        ///
        /// Fails if:
        ///  - called in the off-chain environment
        ///  - the chain is not configured with Xcm
        ///  - the XCM program executed failed (e.g contract doesn't have enough balance)
        #[ink(message)]
        pub fn transfer_through_xcm(
            &mut self,
            receiver: AccountId,
            value: Balance,
        ) -> Result<(), RuntimeError> {
            self.env()
                .xcm_execute(&VersionedXcm::V3(Xcm::<()>(vec![
                    WithdrawAsset(vec![(Here, value).into()].into()),
                    DepositAsset {
                        assets: All.into(),
                        beneficiary: AccountId32 {
                            network: None,
                            id: *receiver.as_ref(),
                        }
                        .into(),
                    },
                ])))
                .map_err(Into::into)
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::{
            ChainBackend,
            ContractsBackend,
        };

        use ink::{
            env::{
                test::default_accounts,
                DefaultEnvironment,
            },
            primitives::AccountId,
        };

        type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

        /// The base number of indivisible units for balances on the
        /// `substrate-contracts-node`.
        const UNIT: Balance = 1_000_000_000_000;

        /// The contract will be given 1000 tokens during instantiation.
        const CONTRACT_BALANCE: Balance = 1_000 * UNIT;

        /// The receiver will get enough funds to have the required existential deposit.
        ///
        /// If your chain has this threshold higher, increase the transfer value.
        const TRANSFER_VALUE: Balance = 1 / 10 * UNIT;

        /// An amount that is below the existential deposit, so that a transfer to an
        /// empty account fails.
        ///
        /// Must not be zero, because such an operation would be a successful no-op.
        const INSUFFICIENT_TRANSFER_VALUE: Balance = 1;

        /// Positive case scenario:
        #[ink_e2e::test]
        async fn transfer_with_xcm_execute_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let constructor = XcmExecuteRef::new();
            let contract = client
                .instantiate(
                    "xcm-execute",
                    &ink_e2e::alice(),
                    constructor,
                    CONTRACT_BALANCE,
                    None,
                )
                .await
                .expect("instantiate failed");
            let mut call = contract.call::<XcmExecute>();

            let receiver: AccountId = default_accounts::<DefaultEnvironment>().bob;

            let contract_balance_before = client
                .balance(contract.account_id)
                .await
                .expect("Failed to get account balance");
            let receiver_balance_before = client
                .balance(receiver)
                .await
                .expect("Failed to get account balance");

            // when
            let transfer_message = call.transfer_through_xcm(receiver, TRANSFER_VALUE);

            let call_res = client
                .call(&ink_e2e::alice(), &transfer_message, 0, None)
                .await
                .expect("call failed");

            assert!(call_res.return_value().is_ok());

            // then
            let contract_balance_after = client
                .balance(contract.account_id)
                .await
                .expect("Failed to get account balance");
            let receiver_balance_after = client
                .balance(receiver)
                .await
                .expect("Failed to get account balance");

            assert_eq!(
                contract_balance_before,
                contract_balance_after + TRANSFER_VALUE
            );
            assert_eq!(
                receiver_balance_before,
                receiver_balance_after - TRANSFER_VALUE
            );

            Ok(())
        }

        /// Negative case scenario:
        #[ink_e2e::test]
        async fn transfer_with_xcm_execute_fails_when_execution_fails<
            Client: E2EBackend,
        >(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let constructor = XcmExecuteRef::new();
            let contract = client
                .instantiate(
                    "xcm-execute",
                    &ink_e2e::alice(),
                    constructor,
                    CONTRACT_BALANCE,
                    None,
                )
                .await
                .expect("instantiate failed");
            let mut call = contract.call::<XcmExecute>();

            let receiver: AccountId = default_accounts::<DefaultEnvironment>().bob;

            // when
            let transfer_message = call.transfer_through_xcm(receiver, TRANSFER_VALUE);

            let call_res = client
                .call_dry_run(&ink_e2e::alice(), &transfer_message, 0, None)
                .await
                .return_value();

            // then
            assert!(matches!(call_res, Err(RuntimeError::CallRuntimeFailed)));

            Ok(())
        }
    }
}
