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
	use frame_support::traits::Currency;
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;
	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);
///create weight for every function instead of supplie 0 as weight
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Currency: Currency<Self::AccountId>;
	}
	///id must be supplied from the runtime like currency
	pub type Id = u64;
	pub type NftOf<T> = Nft<<T as frame_system::Config>::AccountId, Vec<u8>>;
	pub type ClassOf<T> = Class<<T as frame_system::Config>::AccountId, Vec<u8>>;

	#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub struct Nft<AccountId, Body> {
		pub owner: AccountId,
		pub description: Body,
		pub name: Body,
		pub url: Body,
		pub external_url: Body,
		pub class_id: Id,
	}

	#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub struct Class<AccountId, Body> {
		pub owner: AccountId,
		pub description: Body,
		pub name: Body,
		pub url: Body,
		pub external_url: Body,
		///nft number
	}

	// collection => nft => classId

	#[pallet::storage]
	#[pallet::getter(fn next_class_id)]
	pub type NextClassId<T> = StorageValue<_, Id>;

	#[pallet::storage]
	#[pallet::getter(fn next_nft_id)]
	pub type NextNftId<T> = StorageValue<_, Id>;

	#[pallet::storage]
	#[pallet::getter(fn classes)]
	pub type Classes<T: Config> = StorageMap<_, Twox64Concat, Id, ClassOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn nfts)]
	pub type Nfts<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		Id, // class id
		Twox64Concat,
		Id,       // nft id
		NftOf<T>, // nfts
	>;
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClassCreated {
			owner: T::AccountId,
			name: Vec<u8>,
			url: Vec<u8>,
			external_url: Vec<u8>,
			class_id: Id,
			description: Vec<u8>,
		},
		Minted {
			class_id: Id,
			owner: T::AccountId,
			name: Vec<u8>,
			url: Vec<u8>,
			external_url: Vec<u8>,
			description: Vec<u8>,
		},
		///must know who the one who burn the nft
		Burned {
			nft_id: Id,
			class_id: Id,
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
		NftNotFound,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn create_class(
			origin: OriginFor<T>,
			name: Vec<u8>,
			url: Vec<u8>,
			description: Vec<u8>,
			external_url: Vec<u8>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let class = Class {
				name: name.clone(),
				owner: owner.clone(),
				description: description.clone(),
				url: url.clone(),
				external_url: external_url.clone(),
			};
			let id = <NextClassId<T>>::get().unwrap_or(0);
			<Classes<T>>::insert(id, class);
			<NextClassId<T>>::put(id + 1);
			Self::deposit_event(Event::ClassCreated {
				name,
				owner,
				description,
				url,
				external_url,
				class_id: id,
			});
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn mint(
			origin: OriginFor<T>,
			class_id: Id,
			name: Vec<u8>,
			description: Vec<u8>,
			url: Vec<u8>,
			external_url: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			///you need to make sure the owner of the class is the one minting it
			ensure!(<Classes<T>>::contains_key(class_id), Error::<T>::ClassNotFound);
			// let nft_id = Self::match_nft_id();
			///each class nft should start from 1 should start from 
			let nft_id = <NextNftId<T>>::get().unwrap_or(0);
			let nft = Nft {
				owner: who,
				class_id,
				name: name.clone(),
				description: description.clone(),
				url: url.clone(),
				external_url: external_url.clone(),
			};
			<Nfts<T>>::insert(class_id, nft_id, nft.clone());
			<NextNftId<T>>::put(nft_id + 1);
			///name is availiable from the params
			Self::deposit_event(Event::Minted {
				name: nft.name,
				owner: nft.owner,
				description: nft.description,
				url: nft.url,
				external_url: nft.external_url,
				class_id,
			});
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			nft_id: Id,
			class_id: Id,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(<Classes<T>>::contains_key(class_id), Error::<T>::ClassNotFound);
			ensure!(<Nfts<T>>::contains_key(class_id, nft_id), Error::<T>::NftNotFound);
			let mut nft_info = <Nfts<T>>::get(class_id, nft_id).unwrap();
			/// we need to check if the owner is the one who make the transfer
			nft_info.owner = to.clone();
			<Nfts<T>>::insert(class_id, nft_id, nft_info);
			Self::deposit_event(Event::TransferNft { from: who, to, nft_id, class_id });
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(0)]
		pub fn burn(origin: OriginFor<T>, nft_id: Id, class_id: Id) -> DispatchResult {
			let who = ensure_signed(origin)?;
			///why check if key is availiable then getting data just get the data and see if it it the owner
			ensure!(<Nfts<T>>::contains_key(class_id, nft_id), Error::<T>::NftNotFound);
			let nft_info = <Nfts<T>>::get(class_id, nft_id).unwrap();
			ensure!(nft_info.owner == who, Error::<T>::NotOwner);
			<Nfts<T>>::remove(class_id, nft_id);
			Self::deposit_event(Event::Burned { nft_id, class_id });
			Ok(())
		}
	}
}
