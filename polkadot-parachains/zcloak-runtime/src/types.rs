use sp_std::{prelude::*, result::Result, convert::{TryInto, TryFrom}, result, marker::PhantomData, borrow::Borrow};
use xcm::v0::{Error as XcmError, MultiAsset, MultiLocation, Junction};
use frame_support::traits::{Get, tokens::fungibles, Contains};
use xcm_executor::traits::{TransactAsset, Convert, MatchesFungibles, Error as MatchError};
use xcm_executor::traits::FilterAssetLocation;

pub struct SimpleAssetIdConverter<AssetId>(PhantomData<AssetId>);
impl<AssetId: Clone + TryFrom<u128>> Convert<u128, AssetId> for SimpleAssetIdConverter<AssetId> {
	fn convert_ref(id: impl Borrow<u128>) -> Result<AssetId, ()>{
		(*id.borrow()).try_into().map_err(|_e| ())
	}
}


// asset id conversion
pub struct AsPrefixedGeneralIndex<Prefix, AssetId, ConvertAssetId>(PhantomData<(Prefix, AssetId, ConvertAssetId)>);
impl<
	Prefix: Get<MultiLocation>,
	AssetId: Clone,
	ConvertAssetId: Convert<u128, AssetId>,
> Convert<MultiLocation, AssetId> for AsPrefixedGeneralIndex<Prefix, AssetId, ConvertAssetId> {
	fn convert_ref(id: impl Borrow<MultiLocation>) -> result::Result<AssetId, ()> {
		let prefix = Prefix::get();
		let id = id.borrow();
		if !prefix.iter().enumerate().all(|(index, item)| id.at(index) == Some(item)) {
			return Err(())
		}
		match id.at(prefix.len()) {
			Some(Junction::GeneralIndex { id }) => ConvertAssetId::convert_ref(id),
			_ => Err(()),
		}
	}
	fn reverse_ref(what: impl Borrow<AssetId>) -> result::Result<MultiLocation, ()> {
		let mut location = Prefix::get();
		let id = ConvertAssetId::reverse_ref(what)?;
		location.push(Junction::GeneralIndex { id }).map_err(|_| ())?;
		Ok(location)
	}
}



pub struct SimpleBalanceConverter<Balance>(PhantomData<Balance>);
impl<Balance: Clone + TryFrom<u128>> Convert<u128, Balance> for SimpleBalanceConverter<Balance> {
	fn convert_ref(amount: impl Borrow<u128>) -> Result<Balance, ()> {
		Balance::try_from(*amount.borrow()).map_err(|_e| ())
	}
}



pub struct ConvertedConcreteAssetId<AssetId, Balance, ConvertAssetId, ConvertBalance>(
	PhantomData<(AssetId, Balance, ConvertAssetId, ConvertBalance)>
);
impl<
	AssetId: Clone,
	Balance: Clone,
	ConvertAssetId: Convert<MultiLocation, AssetId>,
	ConvertBalance: Convert<u128, Balance>,
> MatchesFungibles<AssetId, Balance> for
	ConvertedConcreteAssetId<AssetId, Balance, ConvertAssetId, ConvertBalance>
{
	fn matches_fungibles(a: &MultiAsset) -> result::Result<(AssetId, Balance), MatchError> {
		let (id, amount) = match a {
			MultiAsset::ConcreteFungible { id, amount } => (id, amount),
			_ => return Err(MatchError::AssetNotFound),
		};
		let what = ConvertAssetId::convert_ref(id).map_err(|_| MatchError::AssetIdConversionFailed)?;
		let amount = ConvertBalance::convert_ref(amount).map_err(|_| MatchError::AmountToBalanceConversionFailed)?;
		Ok((what, amount))
	}
}

pub struct TrustedReserve;

impl FilterAssetLocation for TrustedReserve {
	fn filter_asset_location(asset: &MultiAsset, origin: &MultiLocation) -> bool {
		true
	}
}