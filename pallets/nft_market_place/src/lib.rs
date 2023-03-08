#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
#[frame_support::pallet]
pub mod pallet {
	use codec::MaxEncodedLen;
	use frame_support::pallet_prelude::*;
	use frame_support::sp_runtime::SaturatedConversion;
	use frame_support::traits::UnixTime;
	use frame_support::traits::{Currency, ExistenceRequirement};
	use frame_system::pallet_prelude::*;
	use pallet_nft::{Classes, Id, Nft, Nfts};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_nft::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Currency: Currency<Self::AccountId>;
		type UnixTime: UnixTime;
	}
	pub type OfferOf<T> = Offer<<T as frame_system::Config>::AccountId, BalanceOf<T>>;
	pub type AuctionOf<T> = Auction<<T as frame_system::Config>::AccountId, BalanceOf<T>>;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	pub type Balance = u128;
	pub type Time = u64;
	#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub struct Offer<AccountId, Balance> {
		pub nft_id: Id,
		pub class_id: Id,
		pub amount: Balance,
		pub owner: AccountId,
	}

	#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub struct Auction<AccountId, Balance> {
		pub owner: AccountId,
		pub nft_id: Id,
		pub spender: AccountId,
		pub class_id: Id,
		pub min_amount: Balance,
		pub amount: Balance,
		pub start_time: Time,
		pub end_time: Time,
	}

	#[pallet::storage]
	#[pallet::getter(fn offers)]
	pub type Offers<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		Id, // class id
		Twox64Concat,
		Id,         // nft id
		OfferOf<T>, // offers
	>;

	#[pallet::storage]
	#[pallet::getter(fn auctions)]
	pub type Auctions<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		Id, // class id
		Twox64Concat,
		Id,           // nft id
		AuctionOf<T>, // auctions
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		OfferCreated {
			class_id: Id,
			nft_id: Id,
			owner: T::AccountId,
			amount: BalanceOf<T>,
		},
		OfferBuyed {
			buyer: T::AccountId,
			sender: T::AccountId,
			amount: BalanceOf<T>,
			nft_id: Id,
		},
		AuctionCreated {
			class_id: Id,
			nft_id: Id,
			owner: T::AccountId,
			min_amount: BalanceOf<T>,
			amount: BalanceOf<T>,
			start_time: Time,
			end_time: Time,
		},
		NewUserAuctioner {
			nft_id: Id,
			class_id: Id,
			auctioner: T::AccountId,
			amount: BalanceOf<T>,
		},
		AuctionFinished {
			nft_id: Id,
			class_id: Id,
			spender: T::AccountId,
			amount: BalanceOf<T>,
		},
		AuctionCanceled {
			class_id: Id,
			nft_id: Id,
		},
		OfferCanceled {
			class_id: Id,
			nft_id: Id,
		},
		TransferNft {
			from: T::AccountId,
			to: T::AccountId,
			nft_id: Id,
			class_id: Id,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		ClassNotFound,
		NotOwner,
		NotMinted,
		AmountToLow,
		NftBuyed,
		OfferIsRun,
		TimeOut,
		TimeNotStarted,
		NoBodyParticipateWithThisAuction,
		CantCloseAuction,
		UnderTime,
		EndTimeUnderStartTime,
		WaitWhenAuctionFinish,
		OnlyOwner,
		OfferNotFound,
		NftNotFound,
		AuctionNotFound,
	}
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		///the name listing instead of offer there is another features called offer for next update
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn create_offer(
			origin: OriginFor<T>,
			class_id: Id,
			nft_id: Id,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			/// make it function instead if using ut like this
			let nft = <Nfts<T>>::get(class_id, nft_id).unwrap();
			ensure!(nft.owner == who, Error::<T>::NotOwner);
			///create function to transfer number from balance into u128 instead of doing it like this
			ensure!(amount > 0_u128.saturated_into::<BalanceOf<T>>(), Error::<T>::AmountToLow);
			let offer = Offer { class_id, nft_id, owner: who.clone(), amount };
			<Offers<T>>::insert(class_id, nft_id, offer);
			Self::deposit_event(Event::OfferCreated { class_id, nft_id, owner: who, amount });
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn buy_offer(
			origin: OriginFor<T>,
			nft_id: Id,
			class_id: Id,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let from = ensure_signed(origin.clone())?;
			let offer = <Offers<T>>::get(class_id, nft_id).unwrap();
			/// why contains key for class then nft and then get nft getting nft if not found  will throw error
			ensure!(<Classes<T>>::contains_key(class_id), Error::<T>::ClassNotFound);
			ensure!(<Nfts<T>>::contains_key(class_id, nft_id), Error::<T>::NftNotFound);
			let nft = <Nfts<T>>::get(offer.class_id, offer.nft_id).unwrap();
			///you get the offer above what is the meaning of checking it here
			ensure!(<Offers<T>>::contains_key(class_id, nft_id), Error::<T>::OfferNotFound);
			ensure!(offer.owner == nft.owner, Error::<T>::NftBuyed);
			ensure!(amount >= offer.amount, Error::<T>::AmountToLow);
			///create function for the transfer instead of using it like this
			<T as Config>::Currency::transfer(
				&from,
				&nft.owner,
				offer.amount,
				ExistenceRequirement::KeepAlive,
			)?;
			///you created new nft you need to transfer the nft to the new owner
			<Nfts<T>>::insert(
				class_id,
				nft_id,
				Nft {
					owner: from.clone(),
					class_id,
					description: nft.description.clone(),
					name: nft.name.clone(),
					url: nft.url.clone(),
					external_url: nft.external_url.clone(),
				},
			);
			<Offers<T>>::remove(class_id, nft_id);
			///we need to know the price we sell nft on in the events
			Self::deposit_event(Event::TransferNft { from: nft.owner, to: from, nft_id, class_id });
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn create_auction(
			origin: OriginFor<T>,
			class_id: Id,
			nft_id: Id,
			amount: BalanceOf<T>,
			start_time: Time,
			end_time: Time,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			///dont use contains for the nft and class if you will get the key after
			ensure!(<Classes<T>>::contains_key(class_id), Error::<T>::ClassNotFound);
			ensure!(<Nfts<T>>::contains_key(class_id, nft_id), Error::<T>::NftNotFound);
			///create function for getting owner
			let nft = <Nfts<T>>::get(class_id, nft_id).unwrap();
			ensure!(nft.owner == who, Error::<T>::NotOwner);
			let now = Self::now();
			/// create function for transfer 
			ensure!(amount > 0_u128.saturated_into::<BalanceOf<T>>(), Error::<T>::AmountToLow);
			ensure!(start_time >= now, Error::<T>::UnderTime);
			ensure!(end_time > start_time, Error::<T>::EndTimeUnderStartTime);
			///who is the owner the spender and we dont have current highest bidder and change amount name into higgest bid
			///the amount params it is meant to be the min amount and amount in auction is 0
			let auction = Auction {
				spender: who.clone(),
				class_id,
				nft_id,
				owner: who.clone(),
				amount,
				min_amount: 0_u128.saturated_into::<BalanceOf<T>>(),
				start_time,
				end_time,
			};
			///why clone the whole auction if it is the last time you will use it for the auction min amount you can use 0
			/// why send the amount in the event if it is meaned to be always 0 
			<Auctions<T>>::insert(class_id, nft_id, auction.clone());
			Self::deposit_event(Event::AuctionCreated {
				class_id,
				nft_id,
				owner: who,
				min_amount: auction.min_amount,
				start_time,
				end_time,
				amount,
			});
			Ok(())
		}

		// #[pallet::call_index(3)]
		// #[pallet::weight(0)]
		// pub fn buy_auction(
		// 	origin: OriginFor<T>,
		// 	nft_id: Id,
		// 	class_id: Id,
		// 	amount: BalanceOf<T>,
		// ) -> DispatchResult {
		// 	let who = ensure_signed(origin)?;
		// 	let now = Self::now();
		// 	ensure!(<Classes<T>>::contains_key(class_id), Error::<T>::ClassNotFound);
		// 	ensure!(<Nfts<T>>::contains_key(class_id, nft_id), Error::<T>::NftNotFound);
		// 	ensure!(<Auctions<T>>::contains_key(class_id, nft_id), Error::<T>::AuctionNotFound);
		// 	let nft = <Nfts<T>>::get(class_id, nft_id).unwrap();
		// 	let auction = <Auctions<T>>::get(class_id, nft_id).unwrap();
		// 	ensure!(now >= auction.start_time, Error::<T>::TimeNotStarted);
		// 	ensure!(now < auction.end_time, Error::<T>::TimeOut);
		// 	ensure!(nft.owner != who, Error::<T>::NotOwner);
		// 	ensure!(!<Offers<T>>::contains_key(class_id, nft_id), Error::<T>::OfferIsRun);
		// 	ensure!(amount > auction.amount, Error::<T>::AmountToLow);
		// 	ensure!(nft.owner == auction.owner, Error::<T>::NftBuyed);

		// 	if auction.spender == nft.owner {
		// 		<T as Config>::Currency::transfer(
		// 			&who,
		// 			&auction.owner,
		// 			amount,
		// 			ExistenceRequirement::KeepAlive,
		// 		)?;
		// 		<Auctions<T>>::insert(
		// 			class_id,
		// 			nft_id,
		// 			Auction {
		// 				owner: auction.owner,
		// 				nft_id,
		// 				spender: who.clone(),
		// 				class_id,
		// 				min_amount: amount,
		// 				start_time: auction.start_time,
		// 				end_time: auction.end_time,
		// 				amount: auction.amount,
		// 			},
		// 		);
		// 		return Ok(());
		// 	}
		
		// 	<T as Config>::Currency::transfer(
		// 		&who,
		// 		&auction.owner,
		// 		amount,
		// 		ExistenceRequirement::KeepAlive,
		// 	)?;
		// 	<T as Config>::Currency::transfer(
		// 		&auction.owner,
		// 		&auction.spender,
		// 		auction.min_amount,
		// 		ExistenceRequirement::KeepAlive,
		// 	)?;
		// 	<Auctions<T>>::insert(
		// 		class_id,
		// 		nft_id,
		// 		Auction {
		// 			owner: auction.owner,
		// 			nft_id,
		// 			spender: who.clone(),
		// 			class_id,
		// 			min_amount: amount,
		// 			start_time: auction.start_time,
		// 			end_time: auction.end_time,
		// 			amount: auction.amount,
		// 		},
		// 	);
		// 	Self::deposit_event(Event::NewUserAuctioner {
		// 		nft_id,
		// 		class_id,
		// 		auctioner: who,
		// 		amount,
		// 	});
		// 	Ok(())
		// }

		#[pallet::call_index(4)]
		#[pallet::weight(0)]
		pub fn auction_finish(origin: OriginFor<T>, nft_id: Id, class_id: Id) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			let now = Self::now();
			///stop using contains key
			ensure!(<Classes<T>>::contains_key(class_id), Error::<T>::ClassNotFound);
			ensure!(<Nfts<T>>::contains_key(class_id, nft_id), Error::<T>::NftNotFound);
			ensure!(<Auctions<T>>::contains_key(class_id, nft_id), Error::<T>::AuctionNotFound);
			let nft = <Nfts<T>>::get(class_id, nft_id).unwrap();
			let auction = <Auctions<T>>::get(class_id, nft_id).unwrap();
			///you need to let the winner and the owner finish the auction
			ensure!(who == auction.owner, Error::<T>::OnlyOwner);
			ensure!(now >= auction.end_time, Error::<T>::WaitWhenAuctionFinish);
			ensure!(auction.owner != auction.spender, Error::<T>::NoBodyParticipateWithThisAuction);
			///again transfer the nft using created  function called transfer nft 
			<Nfts<T>>::insert(
				class_id,
				nft_id,
				Nft {
					owner: auction.spender.clone(),
					class_id,
					description: nft.description.clone(),
					name: nft.name.clone(),
					url: nft.url.clone(),
					external_url: nft.external_url.clone(),
				},
			);
			<Auctions<T>>::remove(class_id, nft_id);
			Self::deposit_event(Event::AuctionFinished {
				spender: auction.owner,
				class_id: nft.class_id,
				nft_id,
				amount: auction.min_amount,
			});
			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(0)]
		pub fn cancel_auction(origin: OriginFor<T>, nft_id: Id, class_id: Id) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let auction = <Auctions<T>>::get(class_id, nft_id).unwrap();
			ensure!(auction.owner == who, Error::<T>::NotOwner);
			ensure!(auction.owner == auction.spender, Error::<T>::CantCloseAuction);
			<Auctions<T>>::remove(class_id, nft_id);
			///you need to supply the auction canceler
			Self::deposit_event(Event::AuctionCanceled { nft_id, class_id });
			Ok(())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(0)]
		pub fn cancel_offer(origin: OriginFor<T>, nft_id: Id, class_id: Id) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let offer = <Offers<T>>::get(class_id, nft_id).unwrap();
			ensure!(offer.owner == who, Error::<T>::NotOwner);
			///you already have the class id 
			let class_id = offer.class_id;
			<Offers<T>>::remove(class_id, nft_id);
			Self::deposit_event(Event::OfferCanceled { nft_id, class_id });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn now() -> u64 {
			T::UnixTime::now().as_millis().saturated_into::<u64>()
		}
	}
}
