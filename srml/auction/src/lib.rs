use parity_codec::Codec;
/// sample of simple auction contract in rust
use rstd::{cmp, result};
use rstd::prelude::*;
use runtime_primitives::traits::{Zero, As, Hash, CheckedAdd, CheckedSub, Member, SimpleArithmetic};
use support::{decl_event, decl_module, decl_storage, dispatch::Result, ensure, Parameter, StorageMap, StorageValue};
use system::{self, ensure_signed};
use parity_codec::{Encode, Decode};
use timestamp;
use core::ops::Mul;

use crate::erc20;
use crate::erc721;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Auction<AccountId, Moment, TokenBalance> {
    seller: AccountId,
    startAt: Moment,
    duration: u64,
    startingPrice: TokenBalance,
    endingPrice: TokenBalance,
    lastRecord: TokenBalance,
    lastBidder: AccountId,
    lastBidStartAt: Moment,
}


pub trait Trait: timestamp::Trait + erc20::Trait + erc721::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}



decl_event!(
    pub enum Event<T>
    where
        <T as system::Trait>::AccountId,
        <T as system::Trait>::Hash,
        <T as timestamp::Trait>::Moment,
        <T as erc20::Trait>::TokenBalance
    {
        // event AuctionCreated(
        //        uint256 tokenId, address seller, uint256 startingPriceInToken, uint256 endingPriceInToken, uint256 duration);
        AuctionCreated(Hash, AccountId, TokenBalance, TokenBalance, u64),
        // event AuctionSuccessful(uint256 tokenId, uint256 totalPrice, address winner);
        AuctionSuccessful(Hash, TokenBalance, AccountId),
        // event NewBid(uint256 tokenId, address lastBidder, uint256 lastRecord, uint256 bidStartAt, uint256 returnToLastBidder, uint256 price
        //    );
        NewBid(Hash, AccountId, TokenBalance, Moment, TokenBalance, TokenBalance),
       // event AuctionCancelled(uint256 tokenId);
       AuctionCancelled(Hash),
       // for debug
       Test(u64),
       Price(TokenBalance),
       PriceInternal(u64, TokenBalance),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as AuctionStorage {
        Init get(is_init): bool;
        TokenToAuction get(token_to_auction): map T::Hash => Auction<T::AccountId, T::Moment, T::TokenBalance>;
        Owner get(owner): T::AccountId;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;

        fn init(origin) -> Result {
          let sender = ensure_signed(origin)?;
          ensure!(Self::is_init() == false, "Already initialized.");
          ensure!(Self::owner() == sender, "Only owner can initialize.");

          <Init<T>>::put(true);

          Ok(())
      }

        fn create_auction(origin,
            _token_id: T::Hash,
            _starting_price: T::TokenBalance,
            _ending_price: T::TokenBalance,
            _duration: u64
            ) -> Result {

            let sender = ensure_signed(origin)?;
            let owner = <erc721::Module<T>>::owner_of(_token_id).ok_or("Not the owner")?;
            ensure!(owner == sender, "you do not own this token.");
            ensure!(_duration > 0, "duration must be larger than zero.");

            let new_auction = Auction {
                seller: sender.clone(),
                startAt: timestamp::Module::<T>::now(),
                duration: _duration.clone(),
                startingPrice: _starting_price,
                endingPrice: _ending_price,
                lastRecord: <T::TokenBalance as As<u64>>::sa(0),
                lastBidder: <T::AccountId as Default>::default(),
                lastBidStartAt: T::Moment::sa(0),
            };

            <TokenToAuction<T>>::insert(_token_id, new_auction);
            <erc721::Module<T>>::_burn(_token_id)?;

            Self::deposit_event(RawEvent::AuctionCreated(_token_id, sender, _starting_price, _ending_price, _duration));

            Ok(())
        }

        fn cancel_auction(origin, _token_id: T::Hash) -> Result {
            let sender = ensure_signed(origin)?;
            let auction = Self::token_to_auction(_token_id);
            let seller = auction.seller;
            ensure!(sender.clone() == seller.clone() || sender.clone() == Self::owner(), "you have no right.");

            <TokenToAuction<T>>::remove(_token_id);
            <erc721::Module<T>>::_mint(seller.clone(), _token_id)?;

            Self::deposit_event(RawEvent::AuctionCancelled(_token_id));

            Ok(())
        }


        fn buy_token(origin, _token_id: T::Hash) -> Result {
            Self::deposit_event(RawEvent::Test(1));
            let sender = ensure_signed(origin)?;
            let price = Self::_current_price(_token_id)?;
            Self::deposit_event(RawEvent::Price(price));
            ensure!(erc20::Module::<T>::balance_of(sender.clone()) >= price, "no more money.");
            Self::deposit_event(RawEvent::Test(2));
            let mut auction = Self::token_to_auction(_token_id);

            let now = timestamp::Module::<T>::now();
            let last_bid_at = auction.lastBidStartAt;
            let start_at = auction.startAt.clone();
            ensure!(now.clone() >= start_at || now.clone() - last_bid_at <= T::Moment::sa(1800), "time out.");
            Self::deposit_event(RawEvent::Test(1));

            let last_price = auction.lastRecord;
            if last_price != <T::TokenBalance as As<u64>>::sa(0) {
                <erc20::Module<T>>::_transfer(sender.clone(), auction.lastBidder.clone(), last_price)?;
                let margin = price.checked_sub(&last_price).ok_or("last price is higher.")?;
                <erc20::Module<T>>::_transfer(sender.clone(), auction.seller.clone(), margin)?;
            } else {
                <erc20::Module<T>>::_transfer(sender.clone(), auction.seller.clone(), price)?;
            }

             Self::deposit_event(RawEvent::NewBid(_token_id, auction.lastBidder.clone(), auction.lastRecord.clone(), now.clone(), last_price, price));


            auction.lastRecord = price;
            auction.lastBidder = sender;
            auction.lastBidStartAt = now;

            <TokenToAuction<T>>::insert(_token_id, auction);

             Ok(())
        }

        fn claim_token(_token_id: T::Hash) -> Result {
             ensure!(<TokenToAuction<T>>::exists(_token_id), "the auction does not exist.");
             let auction = Self::token_to_auction(_token_id);
             let last_bid_at = auction.lastBidStartAt;
             let now = timestamp::Module::<T>::now();
             let bidder = auction.lastBidder;

             if now - last_bid_at > T::Moment::sa(1800) {
                <erc721::Module<T>>::_mint(bidder.clone(), _token_id)?;
                <TokenToAuction<T>>::remove(_token_id);
                Self::deposit_event(RawEvent::AuctionSuccessful(_token_id, auction.lastRecord, bidder));
             }

             Ok(())
        }

    }
}


impl<T: Trait> Module<T> {

    fn _calculate_price(_starting_price: T::TokenBalance, _ending_price: T::TokenBalance, _duration: u64, _second_pass: u64) -> result::Result<T::TokenBalance, &'static str> {
        if _second_pass >= _duration {
            return Ok(_ending_price);
        } else {
            let max = cmp::max(_starting_price, _ending_price);
            Self::deposit_event(RawEvent::PriceInternal(1, max));
            let min = cmp::min(_starting_price, _ending_price);
            Self::deposit_event(RawEvent::PriceInternal(2, min));

            let total_changed = max - min;

            let duration_in_ring : T::TokenBalance =  As::sa(_duration);
            let second_passed_in_ring: T::TokenBalance = As::sa(_second_pass);
            let current_price_changed = total_changed * second_passed_in_ring / duration_in_ring;
            let mut current_price: T::TokenBalance = As::sa(0);

            if max == _starting_price {
                current_price = max - current_price_changed;
            } else {
                current_price = min + current_price_changed;
            }


            return Ok(current_price);
        }

    }


    fn _current_price(_token_id: T::Hash) -> result::Result<T::TokenBalance, &'static str> {
        ensure!(<TokenToAuction<T>>::exists(_token_id), "the auction does not exist.");
        let auction = Self::token_to_auction(_token_id);
        let start_at: T::Moment = auction.startAt;
        let second_passed = timestamp::Module::<T>::now() - start_at;

        if auction.lastRecord == <T::TokenBalance as As<u64>>::sa(0) {
            return Self::_calculate_price(auction.startingPrice, auction.endingPrice, auction.duration, <T::Moment as As<u64>>::as_(second_passed));
        } else {
            return Ok(auction.lastRecord * As::sa(110) / As::sa(100));
        }

    }
}