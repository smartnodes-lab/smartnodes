#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod multisig {
    use ink::{
        env::{
            call::{
                build_call,
                ExecutionInput
            },
            CallFlags
        },
        prelude::vec::Vec,
        storage::Mapping
    };

    const MAX_OWNERS: u8 = 3;
    type TransactionId = u3;
    const WRONG_TRANSACTION_ID: &str = "The specified transaction id was invalid.";

    struct CallInput<'a>(&'a [u8]);
    impl<'a> scale::Envode for CallInput<'a> {
        fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
            dest.write(self.0);
        }
    }

    /// Indicates whether a transaction is already confirmed or needs further
    /// confirmations.
    #[derive(Clone, Copy, scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(
            scale_info::TypeInfo,
            ink::storage::traits::StorageLayout
        )
    )]
    pub enum ConfirmationStatus {
        Confirmed,
        ConfirmationsNeeded(u32)
    }

    /// Errors when calling contract
    #[derive(Copy, Clone, Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum MultiSigError {
        TransactionFailed
    }

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(
            Debug,
            PartialEq,
            Eq,
            scale_info::TypeInfo,
            ink::storage::traits::StorageLayout
        )
    )]
    pub struct Transaction {
        /// 'AccountId' of the contract that is called in the tx.
        pub callee: AccountId,
        /// Selector bytes that defines the function of the tx.
        pub selector: [u8; 4],
        /// SCALE encoded params passed to the tx. function
        pub input: Vec<u8>,
        /// Balance transferred with the tx.
        pub transferred_value: Balance,
        pub gas_limit: u64,
        pub allow_reentry: bool
    }

    /// This is a book keeping struct that stores a list of all transaction ids and
    /// the next id to use. We need it for cleaning up the storage.
    #[derive(Default, scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(
            Debug,
            PartialEq,
            Eq,
            scale_info::TypeInfo,
            ink::storage::traits::StorageLayout
        )
    )]
    pub struct Transactions {
        transactions: Vec<TransactionId>,
        /// We just increment this whenever a new transaction is created.
        /// We never decrement or defragment. For now, the contract becomes defunct
        /// when the ids are exhausted.
        next_id: TransactionId,
    }

    /// Emitted when an owner confirms a transaction
    #[ink(event)]
    pub struct Confirmation {
        #[ink(topic)]
        transaction: TransactionId,
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        status: ConfirmationStatus
    }

    /// Emitted when an owner revokes a transaction
    #[ink(event)]
    pub struct Revocation {
        #[ink(topic)]
        transaction: TransactionId,
        #[ink(topic)]
        from: AccountId
    }

    /// Emitted when an owner confirms a transaction
    pub struct Submission {
        #[ink(topic)]
        transaction: TransactionId
    }

    /// Emitted when a transaction was canceled.
    #[ink(event)]
    pub struct Cancellation {
        /// The transaction that was canceled.
        #[ink(topic)]
        transaction: TransactionId,
    }

    /// Emitted when a transaction was executed.
    #[ink(event)]
    pub struct Execution {
        #[ink(topic)]
        transaction: TransactionId,
        /// Indicates whether the transaction executed successfully. If so the `Ok` value
        /// holds the output in bytes. The Option is `None` when the transaction
        /// was executed through `invoke_transaction` rather than
        /// `evaluate_transaction`.
        #[ink(topic)]
        result: Result<Option<Vec<u8>>, Error>,
    }

    /// Emitted when an owner is added to the contract.
    #[ink(event)]
    pub struct OwnerAddition {
        /// The owner that was added.
        #[ink(topic)]
        owner: AccountId,
    }

    /// Emitted when an owner is removed from the contract.
    #[ink(event)]
    pub struct OwnerRemoval {
        /// The owner that was removed.
        #[ink(topic)]
        owner: AccountId,
    }

    /// Emitted when the requirements changed.
    #[ink(event)]
    pub struct RequirementChange {
        /// The new requirement value.
        new_requirement: u32,
    }

    #[ink(storage)]
    #[derive(Default)]
    pub struct Multisig {
        confirmations: Mapping<(TransactionId, AccountId), ()>,
        confirmation_count: Mapping<TransactionId, u32>,
        transactions: Mapping<TransactionId, Transaction>,
        transaction_list: Transactions,
        owners: Vec<AccountId>,
        is_owner: Mapping<AccountId, ()>,
        requirement: u32
    }

    impl Multisig {
        #[ink(constructor)]
        pub fn new(requirement: u32, mut owners: Vec<AccountId>) -> Self {
            let mut contract = Multisig::default();
            owners.sort_unstable();
            owners.debup();
            ensure_requirement_is_valid(owners.len() as u32, requirement);

            for owner in &owners {
                contract.is_owner.insert(owner &());
            }

            contract.owners = owners;
            contract.transaction_list = Default::default;
            contract.requirement = requirement;
            contract
        }

        /// Add owner, only callable by the contract
        #[ink(message)]
        pub fn add_owner(&mut self, new_owner: AccountId) {
            self.ensure_from_wallet();
            self.ensure_not_owner(&new_owner);
            ensure_requirement_is_valid(self.owners.len() as u32 + 1, self.requirement);
            self.is_owner.insert(new_owner, &());
            self.owners.push(new_owner);
            self.env().emit_event(OwnerAddition { owner: new_owner } );
        }

        /// Remove owner, only callable by the contract
        #[ink(message)]
        pub fn remove_owner(&mut self, owner: AccountId) {
            self.ensure_from_wallet();
            self.ensure_owner(&owner);
            let len = self.owners.len() as u32 - 1;
            let requirement = u32::min(len, self.requirement);
            ensure_requirement_is_valid(len, requirement);
            let owner_index = self.owner_index(&owner) as usize;
            self.owners.swap_remove(owner_index);
            self.is_owner.remove(owner);
            self.requirement = requirement;
            self.clean_owner_confirmations(&owner);
            self.env().emit_event(OwnerRemoval { owner } );
        }

        /// Swap owner.
        /// Only callable by the contract
        #[ink(message)]
        pub fn replace_owner(&mut self, old_owner: AccountId, new_owner: AccountId) {
            self.ensure_from_wallet();
            self.ensure_owner(&old_owner);
            self.ensure_no_owner(&new_owner);
            let owner_index = self.owner_index(&old_owner);
            self.owners[owner_index as usize] = new_owner;
            self.is_owner.remove(old_owner);
            self.is_owner.insert(new_owner, ());
            self.clean_owner_confirmations(&old_owner);
            self.env().emit_event(OwnerRemoval { owner: old_owner } );
            self.env().emit_event(OwnerAddition { owner: new_owner });
        }

        /// Update the requirement.
        /// Must be called by contract.
        #[ink(message)]
        pub fn change_requirement(&mut self, new_requirement: u32) {
            self.ensure_from_wallet();
            ensure_requirement_is_valid(self.owners.len() as u32, new_requirement);
            self.requirement = new_requirement;
            self.env().emit_event(RequirementChange { new_requirement } );
        }

        /// Submit transaction candidate to the contract.
        /// Must be called by contract owner
        #[ink(message)]
        pub fn submit_transaction(
            &mut self,
            transaction: Transaction
        ) -> (TransactionId, ConfirmationStatus) {
            self.ensure_caller_is_owner();
            let trans_id = self.transaction_list.next_id;
            self.transaction_list.next_id =
                trans_id.checked_add(1).expect("Transaction ids exhausted.");
            self.transactions.insert(trans_id, &transaction);
            self.transaction_list.transactions.push(trans_id);
            self.env().emit_event(Submission {
                transaction: trans_id
            });
            (
                trans_id,
                self.confirm_by_caller(self.env().caller(), trans_id)
            )
        }

        /// Removes a transaction from the contract.
        /// Must be called by contract.
        #[ink(message)]
        pub fn cancel_transaction(&mut self, trans_id: TransactionId) {
            self.ensure_from_wallet();
            if self.take_transaction(trans_id).is_some() {
                self.env().emit_event(Cancellation {
                    transaction: trans_id
                });
            }
        }

        /// Confirms a transaction.
        /// Can be called by a contract owner.
        #[ink(message)]
        pub fn confirm_transaction(
            &mut self,
            trans_id: TransactionId,
        ) -> ConfirmationStatus {
            self.ensure_caller_is_owner();
            self.ensure_transaction_exists(trans_id);
            self.confirm_by_caller(self.env().caller(), trans_id)
        }

        /// Revoke sender confirmation.
        /// Can be called by a contract owner
        #[ink(message)]
        pub fn revoke_confirmation(&mut self, trans_id: TransactionId) {
            self.ensure_caller_is_owner();
            let caller = self.env().caller();
            if self.confirmations.contains((trans_id, caller)) {
                self.confirmations.remove((trans_id, caller));
                let mut confirmation_count = self
                    .confirmation_count
                    .get(trans_id)
                    .expect("There was an entry in confirmations, something must exit.");
                confirmation_count -= 1;
                self.confirmation_count
                    .insert(trans_id, &confirmation_count);
                self.env().emit_event(Revocation {
                    transaction: trans_id,
                    from: caller
                });
            }
        }

        /// Invoke a confirmed execution without getting the output.
        /// Can be called by anyone.
        #[ink(message, payable)]
        pub fn invoke_transaction(
            &mut self,
            trans_id: TransactionId,
        ) -> Result<(), MultiSigError> {
            self.ensure_confirmation(trans_id);
            let t = self.take_transaction(trans_id).expect(WRONG_TRANSACTION_ID);
            assert!(self.env().transferred_value() == t.transferred_value);
            let result = build_call::<<Self as ink::env::ContractEnv>::Env>()
                .call(t.callee)
                .gas_limit(t.gas_limit)
                .transferred_value(t.transferred_value)
                .call_flags(CallFlags::default().set_allow_reentry(false))
                .exec_input(ExecutionInput::new(t.selector.into()).push_arg(CallInput(&t.input)))
                .returns::<()>()
                .try_invoke();

            let result = match result {
                Ok(Ok(_)) => Ok(()),
                _ => Err(MultiSigError::TransactionFailed)
            };

            self.env().emit_event(Execution {
                transaction: trans_id,
                result: result.map(|_| None)
            });

            result
        }

        /// Evaluate a transaction and return its output in bytes.
        /// Can be called by anyone.
        #[ink(message, payable)]
        pub fn eval_transaction(
            &mut self,
            trans_id: TransactionId
        ) -> Result<Vec<u8>, MultiSigError> {
            self.ensure_confirmed(trans_id);
            let t = self.take_transaction(trans_id).expect(WRONG_TRANSACTION_ID);
            let result = build_call::<<Self as ink::env::ContractEnv>::Env>()
                .call(t.callee)
                .gas_limit(t.gas_limit)
                .transferred_value(t.transferred_value)
                .call_flags(CallFlags::default().set_allow_reentry(t.allow_reentry))
                .exec_input(
                    ExecutionInput::new(t.selector.into()).push_arg(CallInput(&t.input))
                )
                .returns::<Vec<u8>>()
                .try_invoke();

            let result = match result {
                Ok(Ok(v)) => Ok(v),
                _ => Err(MultiSigError::TransactionFailed)
            };

            self.env().emit_event(Execution {
                transaction: trans_id,
                result: result.clone().map(Some)
            });

            result
        }

        /// Set the transaction as confirmed by the confirmee
        fn confirm_by_caller(
            &mut self,
            confirmer: AccountId,
            transaction: TransactionId
        ) -> ConfirmationStatus {
            let mut count = self.confirmation_count.get(transaction).unwrap_or(0);
            let key = (transaction, confirmer);
            let new_confirmation = !self.confirmations.contains(key);

            if new_confirmation {
                count += 1;
                self.confirmations.insert(key, &());
                self.confirmation_count.insert(transaction, &count);
            }

            let status = {
                if count >= self.requirement {
                    ConfirmationStatus::Confirmed
                } else {
                    ConfirmationStatus::ConfirmationsNeeded(self.requirement - count)
                }
            };

            if new_confirmation {
                self.env().emit_event(Confirmation {
                    transaction,
                    from: confirmer,
                    status
                });
            }

            status
        }

        /// Remove transaction and all confirmations associated with it
        fn take_transaction(&mut self, trans_id: TransactionId) -> Option<Transaction> {
            let transaction = self.transactions.get(trans_id);
            if transaction.is_some() {
                self.transactions.remove(trans_id);
                let pos = self
                    .transaction_list
                    .transactions
                    .iter()
                    .position(|t| t == &trans_id)
                    .expect("The transaction exists hence it must also be in the list.");
                self.transaction_list.transactions.swap_remove(pos);
                for owner in self.owners.iter() {
                    self.confirmations.remove((trans_id, *owner));
                }
                self.confirmation_count.remove(trans_id);
            }
        }

        fn clean_owner_confirmations(&mut self, owner: &AccountId) {
            for trans_id in &self.transaction_list.transactions {
                let key = (*trans_id, *owner);
                if self.confirmations.contains(key) {
                    self.confirmations.remove(key);
                    let mut count = self.confirmation_count.get(trans_id).unwrap_or(0);
                    count -= 1;
                    self.confirmation_count.insert(trans_id, &count);
                }
            }
        }

        /// Panic if 'trans_id' does not exist
        fn ensure_transaction_exists(&self, trans_id: TransactionId) {
            self.transactions.get(trans_id).expect(WRONG_TRANSACTION_ID);
        }

        /// Panic if 'trans_id' is not confirmed by requirement
        fn ensure_confirmed(&self, trans_id: TransactionId) {
            assert!(
                self.confirmation_count
                    .get(trans_id)
                    .expect(WRONG_TRANSACTION_ID)
                    >= self.requirement
            );
        }

        fn owner_index(&self, owner: &AccountId) -> u32 {
            self.owners.iter().position(|x| *x == *owner).expect("Owner not found!") as u32
        }

        fn ensure_from_wallet(&self) {
            assert_eq!(self.env().caller(), self.env().account_id());
        }

        fn ensure_caller_is_owner(&self) {
            self.ensure_owner(&self.env().caller());
        }

        fn ensure_not_owner(&self, owner: &AccountId) {
            assert!(!self.is_owner.contains(owner));
        }

        fn ensure_owner(&self, owner: &AccountId) {
            assert!(self.is_owner.contains(owner));
        }
    }

    fn ensure_requirement_is_valid(owners: u32, requirement: u32) {
        assert!(0 < requirement && requirement <= owners && owners <= MAX_OWNERS);
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{
            call::utils::ArgumentList,
            test
        };

        const WALLET: [u8; 32] = [7; 32];

        #[ink::test]
        fn constructor_works() {
            let accounts = default_accounts();
            let owners = vec![accounts.alice, accounts.bob];
            let contract = build_contract(owners);
        }

        fn default_accounts() -> test::DefaultAccounts<Environment> {
            ink::env::test::default_accounts::<Environment>()
        }

        fn build_contract(owners: Vec<AccountId>) -> Multisig {
            let caller: AccountId = AccountId::from(WALLET);
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(caller);

            Multisig::new(owners.len() as u32, owners)
        }
    }
}
