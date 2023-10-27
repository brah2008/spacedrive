use std::{
	collections::HashMap,
	marker::PhantomData,
	sync::{Arc, PoisonError, RwLock},
};

use thiserror::Error;
use tokio::sync::{broadcast, Notify};
use tracing::warn;

use crate::{
	spacetime::UnicastStream, spacetunnel::RemoteIdentity, DiscoveredPeer, DiscoveryManagerState,
	Manager, Metadata,
};

/// A Service represents a thing your application exposes to the network that can be discovered and connected to.
pub struct Service<TMeta> {
	name: String,
	state: Arc<RwLock<DiscoveryManagerState>>,
	do_broadcast: Arc<Notify>,
	manager: Arc<Manager>,
	phantom: PhantomData<fn() -> TMeta>,
}

impl<TMeta: Metadata> Service<TMeta> {
	pub fn new(
		name: impl Into<String>,
		manager: Arc<Manager>,
	) -> Result<Self, ErrDuplicateServiceName> {
		let name = name.into();
		let state = manager.discovery_state.clone();
		let do_broadcast = {
			let mut state = state.write().unwrap_or_else(PoisonError::into_inner);
			if state.services.contains_key(&name) {
				return Err(ErrDuplicateServiceName);
			}
			state
				.services
				.insert(name.clone(), (broadcast::channel(20).0, Default::default()));
			state.do_broadcast.clone()
		};

		Ok(Self {
			name,
			state,
			do_broadcast,
			manager,
			phantom: PhantomData,
		})
	}

	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn update(&self, meta: TMeta) {
		if let Some((_, services_meta)) = self
			.state
			.write()
			.unwrap_or_else(PoisonError::into_inner)
			.services
			.get_mut(&self.name)
		{
			*services_meta = meta.to_hashmap();
			self.do_broadcast.notify_waiters();
		} else {
			warn!(
				"Service::update called on non-existent service '{}'. This indicates a major bug in P2P!",
				self.name
			);
		}
	}

	pub fn get_state(&self) -> HashMap<RemoteIdentity, PeerStatus> {
		let connected = self
			.manager
			.state
			.read()
			.unwrap_or_else(PoisonError::into_inner)
			.connected
			.iter()
			.map(|(_, remote_identity)| (remote_identity.clone(), PeerStatus::Connected))
			.collect::<Vec<_>>();

		let state = self.state.read().unwrap_or_else(PoisonError::into_inner);
		state
			.known
			.get(&self.name)
			.into_iter()
			.flatten()
			.map(|remote_identity| (remote_identity.clone(), PeerStatus::Unavailable))
			// We do these after the `Unavailable` to replace the keys that are in both
			.chain(connected)
			.chain(
				state
					.discovered
					.get(&self.name)
					.into_iter()
					.flatten()
					.map(|(remote_identity, _)| (remote_identity.clone(), PeerStatus::Discovered)),
			)
			.collect::<HashMap<RemoteIdentity, PeerStatus>>()
	}

	// TODO: Remove in favor of `get_state` maybe???
	pub fn get_discovered(&self) -> Vec<DiscoveredPeer<TMeta>> {
		self.state
			.read()
			.unwrap_or_else(PoisonError::into_inner)
			.discovered
			.get(&self.name)
			.into_iter()
			.flatten()
			.map(|(i, p)| DiscoveredPeer {
				identity: i.clone(),
				peer_id: p.peer_id,
				metadata: TMeta::from_hashmap(&p.meta).unwrap(),
				addresses: p.addresses.clone(),
			})
			.collect::<Vec<_>>()
	}

	pub async fn connect(
		&self,
		manager: Arc<Manager>,
		identity: &RemoteIdentity,
	) -> Result<UnicastStream, ()> {
		let candidate = {
			let state = self.state.read().unwrap_or_else(PoisonError::into_inner);
			let (_, candidate) = state
				.discovered
				.get(&self.name)
				.ok_or(())?
				.into_iter()
				.find(|(i, _)| *i == identity)
				.ok_or(())?;
			candidate.clone()
		};

		let stream = manager.stream(candidate.peer_id).await.unwrap(); // TODO: handle providing incorrect peer id
		Ok(stream)
	}

	pub fn listen(&self) -> broadcast::Receiver<()> {
		// TODO: Filtering of events -> Discover and expire events only???
		// self.chan.subscribe()
		todo!();
	}
}

impl<Meta> Drop for Service<Meta> {
	fn drop(&mut self) {
		// TODO: Remove from manager + do rebroadcast
	}
}

#[derive(Debug, Error)]
#[error("a service has already been mounted with this name")]
pub struct ErrDuplicateServiceName;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum PeerStatus {
	Unavailable,
	Discovered,
	Connected,
}