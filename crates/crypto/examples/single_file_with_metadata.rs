#![cfg(feature = "headers")]

use sd_crypto::{
	crypto::Encryptor,
	encoding,
	header::{FileHeader, HeaderObjectName},
	keys::Hasher,
	primitives::{CRYPTO_TEST_CONTEXT, CRYPTO_TEST_MAGIC_BYTES, LATEST_FILE_HEADER},
	types::{Algorithm, HashingAlgorithm, Key, Params, Salt},
	Protected,
};
use tokio::fs::File;

const ALGORITHM: Algorithm = Algorithm::XChaCha20Poly1305;
const HASHING_ALGORITHM: HashingAlgorithm = HashingAlgorithm::Argon2id(Params::Standard);

#[derive(bincode::Encode, bincode::Decode)]
pub struct FileInformation {
	pub file_name: String,
}

async fn encrypt() {
	let password = Protected::new(b"password".to_vec());

	let embedded_metadata = FileInformation {
		file_name: "filename.txt".to_string(),
	};

	// Open both the source and the output file
	let mut reader = File::open("test").await.unwrap();
	let mut writer = File::create("test.encrypted").await.unwrap();

	// This needs to be generated here, otherwise we won't have access to it for encryption
	let master_key = Key::generate();

	// These should ideally be done by a key management system
	let content_salt = Salt::generate();
	let hashed_password = Hasher::hash(HASHING_ALGORITHM, password, content_salt, None).unwrap();

	// Create the header for the encrypted file
	let mut header = FileHeader::new(LATEST_FILE_HEADER, ALGORITHM);

	// Create a keyslot to be added to the header
	header
		.add_keyslot(
			HASHING_ALGORITHM,
			content_salt,
			hashed_password,
			master_key.clone(),
			CRYPTO_TEST_CONTEXT,
		)
		.unwrap();

	header
		.add_object(
			HeaderObjectName::new("Metadata"),
			CRYPTO_TEST_CONTEXT,
			master_key.clone(),
			&encoding::encode(&embedded_metadata).unwrap(),
		)
		.unwrap();

	// Write the header to the file
	header
		.write_async(&mut writer, CRYPTO_TEST_MAGIC_BYTES)
		.await
		.unwrap();

	// Use the nonce created by the header to initialise a stream encryption object
	let encryptor = Encryptor::new(master_key, header.get_nonce(), header.get_algorithm()).unwrap();

	// Encrypt the data from the reader, and write it to the writer
	// Use AAD so the header can be authenticated against every block of data
	encryptor
		.encrypt_streams_async(&mut reader, &mut writer, header.get_aad().inner())
		.await
		.unwrap();
}

async fn decrypt_metadata() {
	let password = Protected::new(b"password".to_vec());

	// Open the encrypted file
	let mut reader = File::open("test.encrypted").await.unwrap();

	// Deserialize the header, keyslots, etc from the encrypted file
	let header = FileHeader::from_reader_async(&mut reader, CRYPTO_TEST_MAGIC_BYTES)
		.await
		.unwrap();

	let master_key = header
		.decrypt_master_key_with_password(password, CRYPTO_TEST_CONTEXT)
		.unwrap();

	// Decrypt the metadata
	let file_info: FileInformation = encoding::decode(
		header
			.decrypt_object(
				HeaderObjectName::new("Metadata"),
				CRYPTO_TEST_CONTEXT,
				master_key,
			)
			.unwrap()
			.expose(),
	)
	.unwrap();

	println!("file name: {}", file_info.file_name);
}

#[tokio::main]
async fn main() {
	encrypt().await;

	decrypt_metadata().await;
}
