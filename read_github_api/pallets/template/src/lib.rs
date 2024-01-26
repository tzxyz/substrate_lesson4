#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

mod github;

use sp_core::crypto::KeyTypeId;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"btc!");
// 这个模块是签名需要用到的
pub mod crypto {
	use super::KEY_TYPE;
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify,
		MultiSignature, MultiSigner,
	};
	app_crypto!(sr25519, KEY_TYPE);

	pub struct TestAuthId;

	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	// implemented for mock runtime in test
	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
		for TestAuthId
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

#[frame_support::pallet]
pub mod pallet {

	use frame_support::pallet_prelude::*;
	use frame_system::{
		offchain::{
			AppCrypto, CreateSignedTransaction, SendUnsignedTransaction, SignedPayload, Signer,
			SigningTypes,
		},
		pallet_prelude::*,
	};

	use crate::github::GithubUser;
	use frame_support::inherent::Vec;
	use sp_runtime::{
		offchain::{http, Duration},
		transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction},
		RuntimeDebug,
	};

	const ONCHAIN_TX_KEY: &[u8] = b"kuaidi100::indexing_parcel_weight";

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct Payload<Public> {
		user: GithubUser,
		public: Public,
	}

	impl<T: SigningTypes> SignedPayload<T> for Payload<T::Public> {
		fn public(&self) -> T::Public {
			self.public.clone()
		}
	}

	#[derive(Debug, Encode, Decode, Default)]
	struct IndexingData(BoundedVec<u8, ConstU32<4>>);

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	// pub trait Config: frame_system::Config +
	// frame_system::offchain::SendTransactionTypes<Call<Self>> {
	pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
		/// The identifier type for an offchain worker.
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		ParcelWeightStored { parcel_weight: BoundedVec<u8, ConstU32<4>>, who: T::AccountId },
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn unsigned_extrinsic_with_signed_payload(
			origin: OriginFor<T>,
			payload: Payload<T::Public>,
			_signature: T::Signature,
		) -> DispatchResult {
			ensure_none(origin)?;

			log::info!(
				"OCW ==> in call unsigned_extrinsic_with_signed_payload: {:?}",
				payload.user
			);
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn set_parcel_weight(
			origin: OriginFor<T>,
			parcel_weight: BoundedVec<u8, ConstU32<4>>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			log::info!("EXTRINSIC ==> set_parcel_weight: {:?}", parcel_weight);
			let data = IndexingData(parcel_weight.clone());

			log::info!("EXTRINSIC ==> set key: {:?}", ONCHAIN_TX_KEY);
			log::info!(
				"EXTRINSIC ==> set value: {:?}",
				sp_std::str::from_utf8(&parcel_weight).unwrap()
			);
			sp_io::offchain_index::set(&ONCHAIN_TX_KEY, &data.encode());

			Self::deposit_event(Event::ParcelWeightStored { parcel_weight, who: _who });
			Ok(())
		}
	}

	// 发送未签名交易时需要实现的 trait
	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		/// Validate unsigned call to this module.
		///
		/// By default unsigned transactions are disallowed, but implementing the validator
		/// here we make sure that some particular calls (the ones produced by offchain worker)
		/// are being whitelisted and marked as valid.
		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			const UNSIGNED_TXS_PRIORITY: u64 = 100;
			let valid_tx = |provide| {
				ValidTransaction::with_tag_prefix("my-pallet")
					.priority(UNSIGNED_TXS_PRIORITY) // please define `UNSIGNED_TXS_PRIORITY` before this line
					.and_provides([&provide])
					.longevity(3)
					.propagate(true)
					.build()
			};

			match call {
				Call::unsigned_extrinsic_with_signed_payload { ref payload, ref signature } => {
					if !SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone()) {
						return InvalidTransaction::BadProof.into()
					}
					valid_tx(b"unsigned_extrinsic_with_signed_payload".to_vec())
				},
				_ => InvalidTransaction::Call.into(),
			}
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: T::BlockNumber) {
			let parcel_weight = Self::get_parcel_weight_from_storage();
			if let Ok(user) = Self::get_github_user(parcel_weight) {
				log::info!("OCW ==> Github User: {:?}", user);

				// Retrieve the signer to sign the payload
				let signer = Signer::<T, T::AuthorityId>::any_account();

				// `send_unsigned_transaction` is returning a type of `Option<(Account<T>,
				// Result<(), ()>)>`. 	 The returned result means:
				// 	 - `None`: no account is available for sending transaction
				// 	 - `Some((account, Ok(())))`: transaction is successfully sent
				// 	 - `Some((account, Err(())))`: error occurred when sending the transaction
				if let Some((_, res)) = signer.send_unsigned_transaction(
					// this line is to prepare and return payload
					|acct| Payload { user: user.clone(), public: acct.public.clone() },
					|payload, signature| Call::unsigned_extrinsic_with_signed_payload {
						payload,
						signature,
					},
				) {
					match res {
						Ok(()) => {
							log::info!(
								"OCW ==> unsigned tx with signed payload successfully sent."
							);
						},
						Err(()) => {
							log::error!("OCW ==> sending unsigned tx with signed payload failed.");
						},
					};
				} else {
					// The case of `None`: no account is available for sending
					log::error!("OCW ==> No local account available");
				}
			} else {
				log::info!("OCW ==> Error while fetch kuaidi100 price info!");
			}

			log::info!("OCW ==> Leave from offchain workers!: {:?}", block_number);
		}
	}

	impl<T: Config> Pallet<T> {
		fn get_github_user(
			parcel_weight: BoundedVec<u8, ConstU32<4>>,
		) -> Result<GithubUser, http::Error> {
			// prepare for send request
			let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(8_000));
			let url = Self::get_url(parcel_weight);
			let url = sp_std::str::from_utf8(&url).map_err(|_| {
				log::warn!("No UTF8 body");
				http::Error::Unknown
			})?;
			let request = http::Request::get(url);
			let pending = request
				.add_header("User-Agent", "Substrate-Offchain-Worker")
				.deadline(deadline)
				.send()
				.map_err(|_| http::Error::IoError)?;
			let response =
				pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;
			if response.code != 200 {
				log::warn!("Unexpected status code: {}", response.code);
				return Err(http::Error::Unknown)
			}
			let body = response.body().collect::<Vec<u8>>();
			let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
				log::warn!("No UTF8 body");
				http::Error::Unknown
			})?;

			// parse the response str
			let user: GithubUser =
				serde_json::from_str(body_str).map_err(|_| http::Error::Unknown)?;

			Ok(user)
		}

		/// 获取Github User的url
		fn get_url(_parcel_weight: BoundedVec<u8, ConstU32<4>>) -> Vec<u8> {
			Vec::from("https://api.github.com/users/tzxyz".as_bytes())
		}

		fn get_parcel_weight_from_storage() -> BoundedVec<u8, ConstU32<4>> {
			let mut result = BoundedVec::<u8, ConstU32<4>>::try_from(b"1".to_vec()).unwrap();
			if let Some(parcel_weight) =
				sp_runtime::offchain::storage::StorageValueRef::persistent(ONCHAIN_TX_KEY)
					.get::<IndexingData>()
					.unwrap_or_else(|_| {
						log::info!("OCW ==> Error while fetching data from offchain storage!");
						None
					}) {
				result = parcel_weight.0;
			}
			result
		}
	}
}
