use std::{
	collections::HashMap,
	fmt,
	sync::{Arc, PoisonError, RwLock},
};

use sd_p2p::{Manager, Service};
use tokio::sync::broadcast;
use tracing::error;
use uuid::Uuid;

use crate::library::{Libraries, Library, LibraryManagerEvent};

use super::{P2PManager, PeerMetadata};

pub struct LibraryServices {
	services: RwLock<HashMap<Uuid, Arc<Service<PeerMetadata>>>>, // TODO: probs don't use `PeerMetadata` here
	tx: broadcast::Sender<()>,
}

impl fmt::Debug for LibraryServices {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("LibraryServices")
			.field("services", &self.services.read().unwrap().keys())
			.finish()
	}
}

impl LibraryServices {
	pub fn new(tx: broadcast::Sender<()>) -> Self {
		Self {
			services: Default::default(),
			tx,
		}
	}

	pub(crate) async fn start(manager: Arc<P2PManager>, libraries: Arc<Libraries>) {
		if let Err(err) = libraries
			.rx
			.clone()
			.subscribe(|msg| {
				let manager = manager.clone();
				async move {
					match msg {
						LibraryManagerEvent::Load(library) => {
							manager.libraries.load_library(&library).await
						}
						LibraryManagerEvent::Edit(library) => {
							manager.libraries.edit_library(&library).await
						}
						LibraryManagerEvent::InstancesModified(library) => {
							manager.libraries.load_library(&library).await
						}
						LibraryManagerEvent::Delete(library) => {
							manager.libraries.delete_library(&library).await
						}
					}
				}
			})
			.await
		{
			error!("Core may become unstable! `networked_libraries_v2` manager aborted with error: {err:?}");
		}
	}

	pub fn get(&self, id: &Uuid) -> Option<Arc<Service<PeerMetadata>>> {
		self.services
			.read()
			.unwrap_or_else(PoisonError::into_inner)
			.get(id)
			.cloned()
	}

	pub fn libraries(&self) -> Vec<(Uuid, Arc<Service<PeerMetadata>>)> {
		self.services
			.read()
			.unwrap_or_else(PoisonError::into_inner)
			.iter()
			.map(|(k, v)| (*k, v.clone()))
			.collect::<Vec<_>>()
	}

	pub(crate) async fn load_library(&self, library: &Library) {
		// let (db_owned_instances, db_instances): (Vec<_>, Vec<_>) = library
		// 	.db
		// 	.instance()
		// 	.find_many(vec![])
		// 	.exec()
		// 	.await
		// 	.unwrap()
		// 	.into_iter()
		// 	.partition_map(
		// 		// TODO: Error handling
		// 		|i| match IdentityOrRemoteIdentity::from_bytes(&i.identity).unwrap() {
		// 			IdentityOrRemoteIdentity::Identity(identity) => Either::Left(identity),
		// 			IdentityOrRemoteIdentity::RemoteIdentity(identity) => Either::Right(identity),
		// 		},
		// 	);

		// let mut libraries = manager
		// 	.libraries
		// 	.write()
		// 	.unwrap_or_else(PoisonError::into_inner);

		// // `self.owned_instances` exists so this call to `load_library` does override instances of other libraries.
		// if db_owned_instances.len() != 1 {
		// 	panic!(
		// 		"Library has '{}' owned instance! Something has gone very wrong!",
		// 		db_owned_instances.len()
		// 	);
		// }
		// owned_instances.insert(library.id, db_owned_instances[0].to_remote_identity());

		// TODO: Maintain old data.
		// let mut old_data = libraries.remove(&library.id);
		// libraries.insert(
		// 	library.id,
		// 	Service::new(),
		// 	LibraryData {
		// 		// We register all remote instances to track connection state(`IdentityOrRemoteIdentity::RemoteIdentity`'s only).
		// 		instances: db_instances
		// 			.into_iter()
		// 			.map(|identity| {
		// 				(
		// 					identity.clone(),
		// 					match old_data
		// 						.as_mut()
		// 						.and_then(|d| d.instances.remove(&identity))
		// 					{
		// 						Some(data) => data,
		// 						None => InstanceState::Unavailable,
		// 					},
		// 				)
		// 			})
		// 			.collect(),
		// 	},
		// );

		// self.p2p
		// 	.update_metadata(owned_instances.values().cloned().collect::<Vec<_>>())
		// 	.await;
	}

	pub(crate) async fn edit_library(&self, _library: &Library) {
		// TODO: Send changes to all connected nodes!

		// TODO: Update mdns
	}

	pub(crate) async fn delete_library(&self, library: &Library) {
		// // Lock them together to ensure changes to both become visible to readers at the same time
		// let mut libraries = self.libraries.write().await;
		// let mut owned_instances = self.owned_instances.write().await;

		// // TODO: Do proper library delete/unpair procedure.
		// libraries.remove(&library.id);
		// owned_instances.remove(&library.id);
		// self.p2p
		// 	.update_metadata(owned_instances.values().cloned().collect::<Vec<_>>())
		// 	.await;
	}
}
