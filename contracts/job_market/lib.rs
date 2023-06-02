#![cfg_attr(not(feature = "std"), no_std)]

pub use self::job_market::JobMarketRef;

#[ink::contract]
mod job_market {

    use ink::primitives::AccountId;
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct JobMarket {
        jobs: Mapping<AccountId, Job>,
    }
}
