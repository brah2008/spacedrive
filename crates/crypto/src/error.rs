//! This module contains all possible errors that this crate can return.

use std::string::FromUtf8Error;

use thiserror::Error;

#[cfg(feature = "rspc")]
impl From<Error> for rspc::Error {
	fn from(err: Error) -> Self {
		Self::new(rspc::ErrorCode::InternalServerError, err.to_string())
	}
}

#[cfg(feature = "encoding")]
impl From<Error> for bincode::error::EncodeError {
	fn from(value: Error) -> Self {
		Self::OtherString(value.to_string())
	}
}

#[cfg(feature = "encoding")]
impl From<Error> for bincode::error::DecodeError {
	fn from(value: Error) -> Self {
		Self::OtherString(value.to_string())
	}
}

pub type Result<T> = std::result::Result<T, Error>;

/// This enum defines all possible errors that this crate can give
#[derive(Error, Debug)]
pub enum Error {
	// crypto primitive errors (STREAM, hashing)
	#[error("there was an error while password hashing")]
	PasswordHash,
	#[error("error while encrypting")]
	Encrypt,
	#[error("error while decrypting")]
	Decrypt,
	#[error("error initialising stream encryption/decryption")]
	StreamModeInit,
	#[error("a provided type is completely null")]
	NullType,

	// header errors
	#[cfg(feature = "encoding")]
	#[error("no keyslots available")]
	NoKeyslots,
	#[cfg(feature = "encoding")]
	#[error("tried adding too many keyslots to a header")]
	TooManyKeyslots,
	#[cfg(feature = "encoding")]
	#[error("no header objects available (or none that match)")]
	NoObjects,
	#[cfg(feature = "encoding")]
	#[error("tried to run an object operation which resulted in duplicates")]
	DuplicateObjects,
	#[cfg(feature = "encoding")]
	#[error("tried adding too many objects to a header")]
	TooManyObjects,
	#[cfg(feature = "encoding")]
	#[error("error while encoding with bincode: {0}")]
	BincodeEncode(#[from] bincode::error::EncodeError),
	#[cfg(feature = "encoding")]
	#[error("error while decoding with bincode: {0}")]
	BincodeDecode(#[from] bincode::error::DecodeError),

	// key manager
	#[cfg(feature = "keymanager")]
	#[error("requested key wasn't found in the key manager")]
	KeyNotFound,
	#[cfg(feature = "keymanager")]
	#[error("key is already mounted")]
	KeyAlreadyMounted,
	#[cfg(feature = "keymanager")]
	#[error("key not mounted")]
	KeyNotMounted,
	#[cfg(feature = "keymanager")]
	#[error("key isn't in the queue")]
	KeyNotQueued,
	#[cfg(feature = "keymanager")]
	#[error("key is already in the queue")]
	KeyAlreadyQueued,
	#[cfg(feature = "keymanager")]
	#[error("no default key has been set")]
	NoDefaultKeySet,
	#[cfg(feature = "keymanager")]
	#[error("keymanager is not unlocked")]
	NotUnlocked,
	#[cfg(feature = "keymanager")]
	#[error("no verification key")]
	NoVerificationKey,
	#[cfg(feature = "keymanager")]
	#[error("key isn't flagged as memory only")]
	KeyNotMemoryOnly,

	// general errors
	#[error("expected length differs from actual length")]
	LengthMismatch,
	#[error("I/O error: {0}")]
	Io(#[from] std::io::Error),
	#[error("incorrect password/details were provided")]
	IncorrectPassword,
	#[error("error while serializing/deserializing an item")]
	Serialization,
	#[error("string parse error")]
	StringParse(#[from] FromUtf8Error),

	// keyring
	#[cfg(all(target_os = "linux", feature = "os-keyrings"))]
	#[error("error with the linux keyring: {0}")]
	LinuxKeyringError(#[from] secret_service::Error),
	#[cfg(all(any(target_os = "macos", target_os = "ios"), feature = "os-keyrings"))]
	#[error("error with the apple keyring: {0}")]
	AppleKeyringError(#[from] security_framework::base::Error),
	#[cfg(feature = "os-keyrings")]
	#[error("generic keyring error")]
	KeyringError,
	#[cfg(feature = "os-keyrings")]
	#[error("keyring not available on this platform")]
	KeyringNotSupported,
}
