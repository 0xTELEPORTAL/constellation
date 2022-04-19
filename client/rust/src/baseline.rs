// Teleportal Platform v3
// Copyright 2021 WiTag Inc. dba Teleportal

use crate::contract::properties::channels::{
    apply_to_channel, apply_to_channel_id, Channel, ChannelArenaHandle, ChannelArenaMap,
    ChannelHandle, ChannelId, ChannelsIter, DynChannel, IChannels,
};
use crate::contract::properties::dynamic::{apply_to_prop, DynTpProperty};
use crate::contract::properties::states::{
    apply_to_state_id, IStateHandle, IStates, State, StateArenaHandle, StateArenaMap, StateHandle,
    StateId, StatesIter,
};

use crate::contract::properties::traits::ITpPropertyStatic;
use crate::contract::{Contract, ContractData, ContractDataHandle};
use crate::object::{Object, ObjectHandle};
use crate::time::TimeWarp;

use arena::Arena;
use eyre::{eyre, Result};
use itertools::EitherOrBoth;
use itertools::Itertools;

#[cfg(feature = "safer-ffi")]
use safer_ffi::derive_ReprC;

#[cfg_attr(feature = "safer-ffi", derive_ReprC)]
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum BaselineKind {
    Main,
    Fork,
}

#[cfg_attr(feature = "safer-ffi", derive_ReprC, ReprC::opaque)]
pub struct Baseline {
    kind: BaselineKind,
    objects: Arena<Object>,
    contracts: Arena<ContractData>,
    pub(crate) states: StateArenaMap, // maps from T to Arena<State<T>>
    pub(crate) channels: ChannelArenaMap, // maps from T to Arena<Channel<T>>
}

impl Baseline {
    pub fn new(kind: BaselineKind) -> Self {
        let objects = Arena::new();
        let contracts = Arena::new();
        let states = StateArenaMap::new();
        let channels = ChannelArenaMap::new();

        Self {
            kind,
            objects,
            contracts,
            states,
            channels,
        }
    }

    pub fn kind(&self) -> BaselineKind {
        self.kind
    }

    // ---- Called by the Baseline on its fork ----

    // TODO[SER-259]: determine method for notifying Baseline fork.

    fn on_state_change<T: ITpPropertyStatic>(&self, state: StateHandle<T>) {
        todo!("Notify fork");
    }

    fn on_channel_change<T: ITpPropertyStatic>(&self, channel: ChannelHandle<T>) {
        todo!("Notify fork");
    }

    // ---- Object and Contract Acessors ----

    pub fn register_contract<C: Contract>(&mut self) -> Result<C> {
        for (_, c_data) in self.contracts.iter() {
            let c_id = c_data.id();
            if c_id == C::ID {
                return Err(eyre!("Contract already added!"));
            }
        }
        let handle = self.contracts.insert(ContractData::new(C::ID));
        Ok(C::new(handle))
    }

    pub fn unregister_contract<C: Contract>(&mut self, handle: ContractDataHandle) -> Result<()> {
        let c_data = self
            .contracts
            .get_mut(handle)
            .ok_or_else(|| eyre!("There is no contract with that id to unregister!"))?;

        if c_data.id() != C::ID {
            return Err(eyre!("Handle did not match the provided contract type!"));
        }

        // Its ok to steal the hashmap because c_data will be deleted soon anyway
        let objs = std::mem::take(c_data.objects_mut());
        for o in objs {
            self.object_remove::<C>(o)
                .expect("Failed to remove object!")
        }
        self.contracts.remove(handle);
        Ok(())
    }

    pub fn contract_data(&self, handle: ContractDataHandle) -> Result<&ContractData> {
        self.contracts
            .get(handle)
            .ok_or_else(|| eyre!("No contract exists for that handle!"))
    }

    pub fn iter_objects(&self) -> impl Iterator<Item = (ObjectHandle, &Object)> {
        self.objects.iter()
    }

    pub fn object(&self, obj: ObjectHandle) -> Result<&Object> {
        self.objects
            .get(obj)
            .ok_or_else(|| eyre!("The given handle doesn't exist in the Arena"))
    }

    pub fn object_mut(&mut self, obj: ObjectHandle) -> Result<&mut Object> {
        self.objects
            .get_mut(obj)
            .ok_or_else(|| eyre!("The given handle doesn't exist in the Arena"))
    }

    /// Create an object with the given `states` and `channels`, corresponding
    /// to contract `C`
    ///
    /// # Errors
    /// Will error if the types of any of the states and channels don't match
    /// the contract.
    pub fn object_create<C: Contract>(
        &mut self,
        contract: &C,
        states: impl Iterator<Item = DynTpProperty>,
        channels: impl Iterator<Item = DynChannel>,
    ) -> Result<ObjectHandle> {
        if !self.contracts.contains(contract.handle()) {
            return Err(eyre!("No such contract for that handle"));
        }

        let state_types = C::States::enumerate_types();
        let channel_types = C::Channels::enumerate_types();

        // Check that all types match before attempting to create properties
        macro_rules! check_types {
            ($prop:ident, $types:ident) => {{
                let size = $prop.size_hint().0;
                $prop.zip_longest($types).enumerate().try_fold(
                    Vec::with_capacity(size),
                    |mut acc, (i, either)| {
                        if let EitherOrBoth::Both(p, t) = either {
                            if p.prop_type() != *t {
                                return Err(eyre!(
                                    "Property at field index {} did not match contract type",
                                    i
                                ));
                            }
                            acc.push(p);
                            Ok(acc)
                        } else {
                            return Err(eyre!(
                                "Properties did not match the number of fields in contract"
                            ));
                        }
                    },
                )
            }};
        }

        let states: Vec<DynTpProperty> = check_types!(states, state_types)?;
        let channels: Vec<DynChannel> = check_types!(channels, channel_types)?;

        // actually do the creation
        let mut state_handles: Vec<arena::generational_arena::Index> = Vec::new();
        let mut channel_handles: Vec<arena::generational_arena::Index> = Vec::new();

        for s in states {
            apply_to_prop!(s, |s| state_handles.push(self.state_create(s).into()));
        }
        for c in channels {
            apply_to_channel!(c, |c| channel_handles.push(self.channel_create(c).into()));
        }

        let object = Object::new(
            state_handles,
            channel_handles,
            contract.handle(),
            TimeWarp::default(),
        );
        let obj_handle = self.objects.insert(object);
        self.contracts
            .get_mut(contract.handle())
            .expect("We already checked this")
            .objects_mut()
            .insert(obj_handle);
        Ok(obj_handle)
    }

    pub fn object_remove<C: Contract>(&mut self, obj: ObjectHandle) -> Result<()> {
        let o = if let Some(o) = self.objects.remove(obj) {
            o
        } else {
            return Err(eyre!("Object did not exist, so it could not be removed"));
        };

        // remove all fields of the object
        let states = StatesIter::<C::States>::new(o.contract());
        let channels = ChannelsIter::<C::Channels>::new(o.contract());

        for s in states {
            apply_to_state_id!(s, |id| {
                let handle = self.bind_state(id, obj)?;
                if let Err(e) = self.state_remove(handle) {
                    log::warn!("Failed to remove state, state has been leaked: {}", e);
                }
                Ok::<(), eyre::Report>(())
            })?;
        }

        for c in channels {
            apply_to_channel_id!(c, |id| {
                let handle = self.bind_channel(id, obj)?;
                if let Err(e) = self.channel_remove(handle) {
                    log::warn!("Failed to remove channel, channel has been leaked: {}", e);
                }
                Ok::<(), eyre::Report>(())
            })?;
        }

        Ok(())
    }

    // ---- Property accessors ----

    pub fn state<H: IStateHandle>(&self, state: H) -> Result<H::OutputRef<'_>> {
        state.get(self)
    }

    pub fn state_mut<H: IStateHandle>(&mut self, state: H) -> Result<H::OutputMut<'_>> {
        state.get_mut(self)
    }

    pub fn channel<T: ITpPropertyStatic>(&self, chan: ChannelHandle<T>) -> Result<&Channel<T>> {
        let arena = self
            .channels
            .get()
            .ok_or_else(|| eyre!("The given handle doesn't have an associated Arena"))?;

        arena
            .get(chan)
            .ok_or_else(|| eyre!("The given handle doesn't exist in the Arena"))
    }

    pub fn channel_mut<T: ITpPropertyStatic>(
        &mut self,
        chan: ChannelHandle<T>,
    ) -> Result<&mut Channel<T>> {
        let arena = self
            .channels
            .get_mut()
            .ok_or_else(|| eyre!("The given handle doesn't have an associated Arena"))?;

        arena
            .get_mut(chan)
            .ok_or_else(|| eyre!("The given handle doesn't exist in the Arena"))
    }

    fn state_remove<T: ITpPropertyStatic>(&mut self, state: StateHandle<T>) -> Result<State<T>> {
        let arena = self
            .states
            .get_mut()
            .ok_or_else(|| eyre!("The given handle doesn't have an associated Arena"))?;

        arena
            .remove(state)
            .ok_or_else(|| eyre!("The given handle doesn't exist in the Arena"))
    }

    fn channel_remove<T: ITpPropertyStatic>(
        &mut self,
        channel: ChannelHandle<T>,
    ) -> Result<Channel<T>> {
        let arena = self
            .channels
            .get_mut()
            .ok_or_else(|| eyre!("The given handle doesn't have an associated Arena"))?;

        arena
            .remove(channel)
            .ok_or_else(|| eyre!("The given handle doesn't exist in the Arena"))
    }

    fn state_create<T: ITpPropertyStatic>(&mut self, value: T) -> StateHandle<T> {
        let arena = self
            .states
            .0
            .entry::<StateArenaHandle<T>>()
            .or_insert_with(|| Arena::new());

        arena.insert(State(value))
    }

    fn channel_create<T: ITpPropertyStatic>(&mut self, channel: Channel<T>) -> ChannelHandle<T> {
        let arena = self
            .channels
            .0
            .entry::<ChannelArenaHandle<T>>()
            .or_insert_with(|| Arena::new());

        arena.insert(channel)
    }

    // ---- State and Channel bindings ----

    pub fn bind_state<T: ITpPropertyStatic>(
        &self,
        id: StateId<T>,
        obj: ObjectHandle,
    ) -> Result<StateHandle<T>> {
        let obj = self
            .objects
            .get(obj)
            .ok_or_else(|| eyre!("The given ObjectHandle doesn't exist in the Arena"))?;
        obj.bind_state(id)
    }

    pub fn bind_channel<T: ITpPropertyStatic>(
        &self,
        id: ChannelId<T>,
        obj: ObjectHandle,
    ) -> Result<ChannelHandle<T>> {
        let obj = self
            .objects
            .get(obj)
            .ok_or_else(|| eyre!("The given ObjectHandle doesn't exist in the Arena"))?;
        obj.bind_channel(id)
    }
}

// ---- Index traits ----

impl core::ops::Index<ObjectHandle> for Baseline {
    type Output = Object;

    fn index(&self, index: ObjectHandle) -> &Self::Output {
        &self.objects[index]
    }
}
impl core::ops::IndexMut<ObjectHandle> for Baseline {
    fn index_mut(&mut self, index: ObjectHandle) -> &mut Self::Output {
        &mut self.objects[index]
    }
}

impl<T: ITpPropertyStatic> core::ops::Index<StateHandle<T>> for Baseline {
    type Output = State<T>;

    fn index(&self, index: StateHandle<T>) -> &Self::Output {
        self.state(index).expect("Invalid handle")
    }
}
impl<T: ITpPropertyStatic> core::ops::IndexMut<StateHandle<T>> for Baseline {
    fn index_mut(&mut self, index: StateHandle<T>) -> &mut Self::Output {
        self.state_mut(index).expect("Invalid handle")
    }
}

impl<T: ITpPropertyStatic> core::ops::Index<ChannelHandle<T>> for Baseline {
    type Output = Channel<T>;

    fn index(&self, index: ChannelHandle<T>) -> &Self::Output {
        self.channel(index).expect("Invalid handle")
    }
}
impl<T: ITpPropertyStatic> core::ops::IndexMut<ChannelHandle<T>> for Baseline {
    fn index_mut(&mut self, index: ChannelHandle<T>) -> &mut Self::Output {
        self.channel_mut(index).expect("Invalid handle")
    }
}