mod channel;
mod misc;

pub use self::channel::Channel;
pub use crate::contract::properties::dynamic::apply_to_channel_id;

use crate::contract::properties::dynamic::TpPropertyType;
use crate::contract::properties::dynamic::__macro::DynTpPropId;
use crate::contract::properties::traits::ITpProperty;
use crate::contract::ContractDataHandle;

use std::any::TypeId;
use std::marker::PhantomData;
use typemap::ShareMap;

pub type ChannelHandle<T> = arena::Index<Channel<T>>;

/// A `TypeMap` key to access the arena containing `State<T>`s.
pub struct ChannelArenaHandle<T: ITpProperty>(PhantomData<T>);
impl<T: ITpProperty> typemap::Key for ChannelArenaHandle<T> {
    type Value = arena::Arena<Channel<T>>;
}

pub type ChannelArenaMap = ShareMap;

/// Represents a particular channel field of a contract. For actual channel data
/// of a specific object, see [`ChannelHandle`].
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct ChannelId<T: ITpProperty> {
    idx: usize, // idx into an object's channel properties
    contract: ContractDataHandle,
    _phantom: PhantomData<T>,
}
impl<T: ITpProperty> ChannelId<T> {
    pub fn contract(&self) -> ContractDataHandle {
        self.contract
    }

    pub(crate) fn idx(&self) -> usize {
        self.idx
    }

    pub fn new(idx: usize, contract: ContractDataHandle) -> Self {
        Self {
            idx,
            contract,
            _phantom: PhantomData,
        }
    }
}

pub trait IChannels {
    fn type_ids() -> &'static [TypeId];
    fn enumerate_types() -> &'static [TpPropertyType];
}

impl IChannels for () {
    fn type_ids() -> &'static [TypeId] {
        &[]
    }

    fn enumerate_types() -> &'static [TpPropertyType] {
        &[]
    }
}

DynTpPropId!(DynChannelId, ChannelId);

super::prop_iter!(ChannelsIter, IChannels, DynChannelId);