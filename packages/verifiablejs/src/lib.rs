#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
extern crate core;

use alloc::vec::Vec;
use js_sys::{Boolean, JsString, Object, Uint8Array};
use parity_scale_codec::{Decode, Encode};
use verifiable::ring::ark_vrf::ring::SrsLookup;
use verifiable::{
	ring::{
		bandersnatch::{BandersnatchSha512Ell2, BandersnatchVrfVerifiable},
		ring_verifier_builder_params, RingDomainSize, RingSize, StaticChunk,
	},
	Alias, BatchProofItem, Entropy, GenerateVerifiable,
};
use wasm_bindgen::prelude::*;

type Bvv = BandersnatchVrfVerifiable;

fn parse_domain_size(domain_size: u32) -> Result<RingDomainSize, JsString> {
	match domain_size {
		11 => Ok(RingDomainSize::Domain11),
		12 => Ok(RingDomainSize::Domain12),
		16 => Ok(RingDomainSize::Domain16),
		_ => Err(JsString::from("Invalid domain_size. Use 11, 12, or 16.")),
	}
}

fn parse_capacity(domain_size: u32) -> Result<<Bvv as GenerateVerifiable>::Capacity, JsString> {
	let ds = parse_domain_size(domain_size)?;
	Ok(RingSize::from(ds))
}

fn decode_members(
	members: Uint8Array,
) -> Result<Vec<<Bvv as GenerateVerifiable>::Member>, JsString> {
	let raw = members.to_vec();
	Vec::<<Bvv as GenerateVerifiable>::Member>::decode(&mut &raw[..])
		.map_err(|_| JsString::from("Decoding Members failed"))
}

fn build_members_commitment(
	domain_size: u32,
	members: Vec<<Bvv as GenerateVerifiable>::Member>,
) -> Result<<Bvv as GenerateVerifiable>::Members, JsString> {
	let ds = parse_domain_size(domain_size)?;
	let capacity = RingSize::from(ds);

	let builder_params = ring_verifier_builder_params::<BandersnatchSha512Ell2>(ds);
	let get_many = |range| {
		(&builder_params)
			.lookup(range)
			.map(|v| v.into_iter().map(|i| StaticChunk(i)).collect::<Vec<_>>())
			.ok_or(())
	};

	let mut inter = Bvv::start_members(capacity);
	Bvv::push_members(&mut inter, members.into_iter(), get_many)
		.map_err(|_| JsString::from("push_members failed"))?;
	Ok(Bvv::finish_members(inter))
}

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
/**
 * Ring domain size. Determines the maximum ring capacity.
 * - 11: Domain11 (2^11, max ~255 members) - smallest, fastest
 * - 12: Domain12 (2^12, max ~767 members)
 * - 16: Domain16 (2^16, max ~16127 members) - largest
 */
export type RingDomainSize = 11 | 12 | 16;

export interface OneShotResult {
    proof: Uint8Array;
    alias: Uint8Array;
    member: Uint8Array;
    members: Uint8Array;
    context: Uint8Array;
    message: Uint8Array;
}

export interface MultiContextResult {
    proof: Uint8Array;
    aliases: Uint8Array;
    member: Uint8Array;
    members: Uint8Array;
    contexts: Uint8Array;
    message: Uint8Array;
}

export function one_shot(domain_size: RingDomainSize, entropy: Uint8Array, members: Uint8Array, context: Uint8Array, message: Uint8Array): OneShotResult;
export function create_multi_context(domain_size: RingDomainSize, entropy: Uint8Array, members: Uint8Array, contexts: Uint8Array, message: Uint8Array): MultiContextResult;
"#;

#[wasm_bindgen(skip_typescript)]
pub fn one_shot(
	domain_size: u32,
	entropy: Uint8Array,
	members: Uint8Array,
	context: Uint8Array,
	message: Uint8Array,
) -> Result<Object, JsString> {
	let capacity = parse_capacity(domain_size)?;

	let entropy_vec = entropy.to_vec();
	let entropy = Entropy::decode(&mut &entropy_vec[..])
		.map_err(|_| JsString::from("Entropy decoding failed"))?;

	// Secret
	let secret = Bvv::new_secret(entropy);

	// Member
	let member = Bvv::member_from_secret(&secret);
	let member_encoded = member.encode();

	// All Members
	let raw_members = members.to_vec();
	let members = Vec::<<Bvv as GenerateVerifiable>::Member>::decode(&mut &raw_members[..])
		.map_err(|_| JsString::from("Decoding Members failed"))?;

	let members_encoded = members.encode();

	// Open
	let commitment = Bvv::open(capacity, &member, members.into_iter())
		.map_err(|_| JsString::from("Verifiable::open failed"))?;

	// Create
	let context = &context.to_vec()[..];
	let message = &message.to_vec()[..];
	let (proof, alias) = Bvv::create(commitment, &secret, context, message)
		.map_err(|_| JsString::from("Verifiable::create failed"))?;

	// Return Results
	let obj = Object::new();
	js_sys::Reflect::set(
		&obj,
		&"member".into(),
		&Uint8Array::from(&member_encoded[..]),
	)
	.unwrap();
	js_sys::Reflect::set(
		&obj,
		&"members".into(),
		&Uint8Array::from(&members_encoded[..]),
	)
	.unwrap();
	js_sys::Reflect::set(
		&obj,
		&"proof".into(),
		&Uint8Array::from(&Encode::encode(&proof)[..]),
	)
	.unwrap();
	js_sys::Reflect::set(&obj, &"alias".into(), &Uint8Array::from(&alias[..])).unwrap();
	js_sys::Reflect::set(&obj, &"message".into(), &Uint8Array::from(&message[..])).unwrap();
	js_sys::Reflect::set(&obj, &"context".into(), &Uint8Array::from(&context[..])).unwrap();
	Ok(obj)
}

#[wasm_bindgen(skip_typescript)]
pub fn create_multi_context(
	domain_size: u32,
	entropy: Uint8Array,
	members: Uint8Array,
	contexts: Uint8Array,
	message: Uint8Array,
) -> Result<Object, JsString> {
	let capacity = parse_capacity(domain_size)?;

	let entropy_vec = entropy.to_vec();
	let entropy = Entropy::decode(&mut &entropy_vec[..])
		.map_err(|_| JsString::from("Entropy decoding failed"))?;

	// Secret
	let secret = Bvv::new_secret(entropy);

	// Member
	let member = Bvv::member_from_secret(&secret);
	let member_encoded = member.encode();

	// All Members
	let raw_members = members.to_vec();
	let decoded_members = Vec::<<Bvv as GenerateVerifiable>::Member>::decode(&mut &raw_members[..])
		.map_err(|_| JsString::from("Decoding Members failed"))?;
	let members_encoded = decoded_members.encode();

	// Decode contexts (SCALE-encoded Vec<Vec<u8>>)
	let raw_contexts = contexts.to_vec();
	let decoded_contexts = Vec::<Vec<u8>>::decode(&mut &raw_contexts[..])
		.map_err(|_| JsString::from("Decoding Contexts failed"))?;
	let contexts_encoded = decoded_contexts.encode();

	// Open
	let commitment = Bvv::open(capacity, &member, decoded_members.into_iter())
		.map_err(|_| JsString::from("Verifiable::open failed"))?;

	// Create multi-context proof
	let message_bytes = &message.to_vec()[..];
	let context_refs: Vec<&[u8]> = decoded_contexts.iter().map(|c| c.as_slice()).collect();
	let (proof, aliases) =
		Bvv::create_multi_context(commitment, &secret, &context_refs, message_bytes)
			.map_err(|_| JsString::from("Verifiable::create_multi_context failed"))?;

	// Return Results
	let obj = Object::new();
	js_sys::Reflect::set(
		&obj,
		&"member".into(),
		&Uint8Array::from(&member_encoded[..]),
	)
	.unwrap();
	js_sys::Reflect::set(
		&obj,
		&"members".into(),
		&Uint8Array::from(&members_encoded[..]),
	)
	.unwrap();
	js_sys::Reflect::set(
		&obj,
		&"proof".into(),
		&Uint8Array::from(&Encode::encode(&proof)[..]),
	)
	.unwrap();
	js_sys::Reflect::set(
		&obj,
		&"aliases".into(),
		&Uint8Array::from(&Encode::encode(&aliases)[..]),
	)
	.unwrap();
	js_sys::Reflect::set(
		&obj,
		&"message".into(),
		&Uint8Array::from(&message_bytes[..]),
	)
	.unwrap();
	js_sys::Reflect::set(
		&obj,
		&"contexts".into(),
		&Uint8Array::from(&contexts_encoded[..]),
	)
	.unwrap();
	Ok(obj)
}

#[wasm_bindgen]
pub fn validate(
	domain_size: u32,
	proof: Uint8Array,
	members: Uint8Array,
	context: Uint8Array,
	message: Uint8Array,
) -> Result<Uint8Array, JsString> {
	let capacity = parse_capacity(domain_size)?;

	let proof = proof.to_vec();
	let proof: <Bvv as GenerateVerifiable>::Proof =
		Decode::decode(&mut &proof[..]).map_err(|_| JsString::from("Decoding Proof failed"))?;

	let decoded_members = decode_members(members)?;
	let members_commitment = build_members_commitment(domain_size, decoded_members)?;

	let context = &context.to_vec()[..];
	let message = &message.to_vec()[..];
	let alias = Bvv::validate(capacity, &proof, &members_commitment, context, message)
		.map_err(|_| JsString::from("Proof not able to be validated"))?;

	Ok(Uint8Array::from(&Encode::encode(&alias)[..]))
}

#[wasm_bindgen]
pub fn validate_multi_context(
	domain_size: u32,
	proof: Uint8Array,
	members: Uint8Array,
	contexts: Uint8Array,
	message: Uint8Array,
) -> Result<Uint8Array, JsString> {
	let capacity = parse_capacity(domain_size)?;

	let proof = proof.to_vec();
	let proof: <Bvv as GenerateVerifiable>::Proof =
		Decode::decode(&mut &proof[..]).map_err(|_| JsString::from("Decoding Proof failed"))?;

	let decoded_members = decode_members(members)?;
	let members_commitment = build_members_commitment(domain_size, decoded_members)?;

	let raw_contexts = contexts.to_vec();
	let decoded_contexts = Vec::<Vec<u8>>::decode(&mut &raw_contexts[..])
		.map_err(|_| JsString::from("Decoding Contexts failed"))?;
	let context_refs: Vec<&[u8]> = decoded_contexts.iter().map(|c| c.as_slice()).collect();

	let message = &message.to_vec()[..];
	let aliases =
		Bvv::validate_multi_context(capacity, &proof, &members_commitment, &context_refs, message)
			.map_err(|_| JsString::from("Multi-context proof not able to be validated"))?;

	Ok(Uint8Array::from(&aliases.encode()[..]))
}

#[wasm_bindgen]
pub fn is_valid(
	domain_size: u32,
	proof: Uint8Array,
	members: Uint8Array,
	context: Uint8Array,
	alias: Uint8Array,
	message: Uint8Array,
) -> Result<Boolean, JsString> {
	let capacity = parse_capacity(domain_size)?;

	let proof = proof.to_vec();
	let proof: <Bvv as GenerateVerifiable>::Proof =
		Decode::decode(&mut &proof[..]).map_err(|_| JsString::from("Decoding Proof failed"))?;

	let decoded_members = decode_members(members)?;
	let members_commitment = build_members_commitment(domain_size, decoded_members)?;

	let alias_vec = alias.to_vec();
	let alias: Alias =
		Decode::decode(&mut &alias_vec[..]).map_err(|_| JsString::from("Decoding Alias failed"))?;

	let context = &context.to_vec()[..];
	let message = &message.to_vec()[..];
	let valid = Bvv::is_valid(capacity, &proof, &members_commitment, context, &alias, message);

	Ok(valid.into())
}

#[wasm_bindgen]
pub fn is_valid_multi_context(
	domain_size: u32,
	proof: Uint8Array,
	members: Uint8Array,
	contexts: Uint8Array,
	aliases: Uint8Array,
	message: Uint8Array,
) -> Result<Boolean, JsString> {
	let capacity = parse_capacity(domain_size)?;

	let proof = proof.to_vec();
	let proof: <Bvv as GenerateVerifiable>::Proof =
		Decode::decode(&mut &proof[..]).map_err(|_| JsString::from("Decoding Proof failed"))?;

	let decoded_members = decode_members(members)?;
	let members_commitment = build_members_commitment(domain_size, decoded_members)?;

	let raw_contexts = contexts.to_vec();
	let decoded_contexts = Vec::<Vec<u8>>::decode(&mut &raw_contexts[..])
		.map_err(|_| JsString::from("Decoding Contexts failed"))?;
	let context_refs: Vec<&[u8]> = decoded_contexts.iter().map(|c| c.as_slice()).collect();

	let raw_aliases = aliases.to_vec();
	let decoded_aliases = Vec::<Alias>::decode(&mut &raw_aliases[..])
		.map_err(|_| JsString::from("Decoding Aliases failed"))?;

	let message = &message.to_vec()[..];
	let valid = Bvv::is_valid_multi_context(
		capacity,
		&proof,
		&members_commitment,
		&context_refs,
		&decoded_aliases,
		message,
	);

	Ok(valid.into())
}

#[wasm_bindgen]
pub fn batch_validate(
	domain_size: u32,
	members: Uint8Array,
	proof_items: Uint8Array,
) -> Result<Uint8Array, JsString> {
	let capacity = parse_capacity(domain_size)?;

	let decoded_members = decode_members(members)?;
	let members_commitment = build_members_commitment(domain_size, decoded_members)?;

	// BatchProofItem doesn't implement Decode, so accept SCALE-encoded
	// Vec<(Proof, Vec<u8>, Vec<u8>)> tuples and construct items manually.
	let raw_items = proof_items.to_vec();
	let tuples = Vec::<(<Bvv as GenerateVerifiable>::Proof, Vec<u8>, Vec<u8>)>::decode(
		&mut &raw_items[..],
	)
	.map_err(|_| JsString::from("Decoding BatchProofItems failed"))?;

	let items: Vec<BatchProofItem<<Bvv as GenerateVerifiable>::Proof>> = tuples
		.into_iter()
		.map(|(proof, context, message)| BatchProofItem {
			proof,
			context,
			message,
		})
		.collect();

	let aliases = Bvv::batch_validate(capacity, &members_commitment, &items)
		.map_err(|_| JsString::from("Batch validation failed"))?;

	Ok(Uint8Array::from(&aliases.encode()[..]))
}

#[wasm_bindgen]
pub fn sign(entropy: Uint8Array, message: Uint8Array) -> Result<Uint8Array, JsString> {
	let entropy_vec = entropy.to_vec();
	let entropy = Entropy::decode(&mut &entropy_vec[..])
		.map_err(|_| JsString::from("Entropy decoding failed"))?;

	// Secret
	let secret = Bvv::new_secret(entropy);

	let message = &message.to_vec()[..];
	let signature =
		Bvv::sign(&secret, &message).map_err(|_| JsString::from("Verifiable::sign failed"))?;

	Ok(Uint8Array::from(&Encode::encode(&signature)[..]))
}

#[wasm_bindgen]
pub fn verify_signature(signature: Uint8Array, message: Uint8Array, member: Uint8Array) -> Boolean {
	let signature = signature.to_vec();
	let signature: <Bvv as GenerateVerifiable>::Signature =
		Decode::decode(&mut &signature[..]).unwrap();

	let member = member.to_vec();
	let member: <Bvv as GenerateVerifiable>::Member = Decode::decode(&mut &member[..]).unwrap();

	let message = &message.to_vec()[..];

	Bvv::verify_signature(&signature, &message, &member).into()
}

#[wasm_bindgen]
pub fn member_from_entropy(entropy: Uint8Array) -> Uint8Array {
	let entropy_vec = entropy.to_vec();
	let entropy = Entropy::decode(&mut &entropy_vec[..]).unwrap();

	// Secret
	let secret = Bvv::new_secret(entropy);

	// Member
	let member = Bvv::member_from_secret(&secret);
	let member_encoded = member.encode();

	Uint8Array::from(&member_encoded[..])
}

#[wasm_bindgen]
pub fn alias_in_context(entropy: Uint8Array, context: Uint8Array) -> Result<Uint8Array, JsString> {
	let entropy_vec = entropy.to_vec();
	let entropy = Entropy::decode(&mut &entropy_vec[..])
		.map_err(|_| JsString::from("Entropy decoding failed"))?;

	let secret = Bvv::new_secret(entropy);
	let context = &context.to_vec()[..];

	let alias = Bvv::alias_in_context(&secret, context)
		.map_err(|_| JsString::from("alias_in_context failed"))?;

	Ok(Uint8Array::from(&Encode::encode(&alias)[..]))
}

#[wasm_bindgen]
pub fn is_member_valid(member: Uint8Array) -> Boolean {
	let member_vec = member.to_vec();
	let member = <Bvv as GenerateVerifiable>::Member::decode(&mut &member_vec[..]);
	match member {
		Ok(m) => Bvv::is_member_valid(&m).into(),
		Err(_) => false.into(),
	}
}

/// Compute the ring root (MembersCommitment) from a SCALE-encoded Vec of members.
/// This returns the 768-byte commitment.
#[wasm_bindgen]
pub fn members_root(domain_size: u32, members: Uint8Array) -> Result<Uint8Array, JsString> {
	let decoded_members = decode_members(members)?;
	let members_commitment = build_members_commitment(domain_size, decoded_members)?;
	let commitment_encoded = members_commitment.encode();
	Ok(Uint8Array::from(&commitment_encoded[..]))
}

/// Compute the intermediate (MembersSet) from a SCALE-encoded Vec of members.
/// This returns the 848-byte intermediate needed for chain storage.
#[wasm_bindgen]
pub fn members_intermediate(domain_size: u32, members: Uint8Array) -> Result<Uint8Array, JsString> {
	let ds = parse_domain_size(domain_size)?;
	let capacity = RingSize::from(ds);

	let decoded_members = decode_members(members)?;

	let builder_params = ring_verifier_builder_params::<BandersnatchSha512Ell2>(ds);
	let get_many = |range| {
		(&builder_params)
			.lookup(range)
			.map(|v| v.into_iter().map(|i| StaticChunk(i)).collect::<Vec<_>>())
			.ok_or(())
	};

	let mut inter = Bvv::start_members(capacity);
	Bvv::push_members(&mut inter, decoded_members.into_iter(), get_many)
		.map_err(|_| JsString::from("push_members failed"))?;

	let intermediate_encoded = inter.encode();
	Ok(Uint8Array::from(&intermediate_encoded[..]))
}

#[cfg(test)]
mod tests {
	use super::*;
	use wasm_bindgen_test::*;

	const TEST_DOMAIN_SIZE: u32 = 11;

	fn get_secret_and_member(
		entropy: &[u8; 32],
	) -> (
		<Bvv as GenerateVerifiable>::Secret,
		<Bvv as GenerateVerifiable>::Member,
	) {
		let secret = Bvv::new_secret(entropy.clone());
		let member = Bvv::member_from_secret(&secret);
		(secret, member)
	}

	fn make_test_members(count: usize) -> Vec<<Bvv as GenerateVerifiable>::Member> {
		(0..count)
			.map(|i| get_secret_and_member(&[i as u8; 32]))
			.map(|(_, m)| m)
			.collect()
	}

	#[wasm_bindgen_test]
	fn create_proof_validate_proof() {
		let entropy = [5u8; 32];
		let js_member = member_from_entropy(Uint8Array::from(entropy.as_slice()));

		let members = make_test_members(10);

		assert_eq!(
			js_member.to_vec(),
			members.get(5).unwrap().encode().to_vec()
		);

		let context = b"Context";
		let message = b"FooBar";

		let result = one_shot(
			TEST_DOMAIN_SIZE,
			Uint8Array::from(entropy.as_slice()),
			Uint8Array::from(members.encode().to_vec().as_slice()),
			Uint8Array::from(context.as_slice()),
			Uint8Array::from(message.as_slice()),
		)
		.expect("creating one_shot proof should work");

		let alias =
			js_sys::Reflect::get(&result, &JsValue::from_str("alias")).expect("alias should exist");
		let alias = Uint8Array::new(&alias);

		let proof =
			js_sys::Reflect::get(&result, &JsValue::from_str("proof")).expect("proof should exist");
		let proof = Uint8Array::new(&proof);

		let validated_alias = validate(
			TEST_DOMAIN_SIZE,
			proof,
			Uint8Array::from(&members.encode().to_vec()[..]),
			Uint8Array::from(context.as_slice()),
			Uint8Array::from(message.as_slice()),
		)
		.expect("validate should succeed");

		assert_eq!(alias.to_vec(), validated_alias.to_vec());
	}

	#[wasm_bindgen_test]
	fn js_rust_equal_member() {
		let entropy = [0u8; 32];
		let alice_secret = Bvv::new_secret(entropy);
		let rust_member = Bvv::member_from_secret(&alice_secret);

		let js_member = member_from_entropy(Uint8Array::from(&entropy[..]));

		assert_eq!(rust_member.encode().len(), js_member.to_vec().len());
		assert_eq!(rust_member.encode().len(), 32);
		assert_eq!(js_member.to_vec().len(), 32);
		assert_eq!(rust_member.encode(), js_member.to_vec());
	}

	#[wasm_bindgen_test]
	fn js_rust_equal_members() {
		let rust_members = make_test_members(10);

		let js_members: Vec<Vec<u8>> = (0..10)
			.map(|i| member_from_entropy(Uint8Array::from([i as u8; 32].as_slice())))
			.map(|key| key.to_vec())
			.collect();

		assert_eq!(js_members.len(), rust_members.len());

		let rust_members_with_encoded_keys = rust_members
			.iter()
			.map(|key| key.encode())
			.collect::<Vec<Vec<u8>>>();

		let rust_members_with_encoded_keys = rust_members_with_encoded_keys.encode();
		let js_members = js_members.encode();

		assert_eq!(js_members, rust_members_with_encoded_keys);
	}

	#[wasm_bindgen_test]
	fn js_rust_equal_proofs() {
		let alice_entropy = [0u8; 32];

		let members = make_test_members(10);
		let alice_member = members.get(0).unwrap();

		// Create Rust Proof
		let context = b"Context";
		let message = b"FooBar";

		let capacity = RingSize::from(RingDomainSize::Domain11);
		let commitment =
			Bvv::open(capacity, &alice_member, members.clone().into_iter()).unwrap();
		let secret = Bvv::new_secret(alice_entropy);
		let (proof, alias) =
			Bvv::create(commitment, &secret, context, message).unwrap();

		// Create JS Proof
		let result = one_shot(
			TEST_DOMAIN_SIZE,
			Uint8Array::from(&alice_entropy[..]),
			Uint8Array::from(&members.encode().to_vec()[..]),
			Uint8Array::from(&context[..]),
			Uint8Array::from(&message[..]),
		)
		.expect("creating one_shot proof should work");

		// Compare js & rust values
		let get_u8a_value = |key: &str| {
			let value =
				js_sys::Reflect::get(&result, &JsValue::from_str(key)).expect("key should exist");
			let value = Uint8Array::new(&value);
			value
		};

		let js_alias = get_u8a_value("alias");
		assert_eq!(js_alias.to_vec(), alias.to_vec());

		let js_member = get_u8a_value("member");
		assert_eq!(js_member.to_vec(), alice_member.encode().to_vec());

		let js_members = get_u8a_value("members");
		assert_eq!(js_members.to_vec(), members.encode().to_vec());

		let js_context = get_u8a_value("context");
		assert_eq!(js_context.to_vec(), context.to_vec());

		let js_message = get_u8a_value("message");
		assert_eq!(js_message.to_vec(), message.to_vec());

		let js_proof = get_u8a_value("proof");
		assert_eq!(js_proof.to_vec().len(), proof.encode().len());

		let js_proof_alias = validate(
			TEST_DOMAIN_SIZE,
			js_proof,
			Uint8Array::from(&members.encode().to_vec()[..]),
			Uint8Array::from(context.as_slice()),
			Uint8Array::from(message.as_slice()),
		)
		.expect("validate should succeed");
		assert_eq!(js_proof_alias.to_vec(), alias.to_vec());

		let rs_proof_alias = validate(
			TEST_DOMAIN_SIZE,
			Uint8Array::from(&proof.encode().to_vec()[..]),
			Uint8Array::from(&members.encode().to_vec()[..]),
			Uint8Array::from(context.as_slice()),
			Uint8Array::from(message.as_slice()),
		)
		.expect("validate should succeed");
		assert_eq!(rs_proof_alias.to_vec(), alias.to_vec());
	}

	#[wasm_bindgen_test]
	fn js_produces_valid_signatures() {
		let entropy = [23u8; 32];
		let message = b"FooBar";
		let secret = Bvv::new_secret(entropy);

		let member = Bvv::member_from_secret(&secret);

		// Create Rust signature
		let rs_signature = Bvv::sign(&secret, message).unwrap();
		assert!(Bvv::verify_signature(&rs_signature, message, &member));

		// Create JS signature
		let js_signature = sign(
			Uint8Array::from(&entropy[..]),
			Uint8Array::from(&message[..]),
		)
		.expect("creating signature should work");

		let js_member = member_from_entropy(Uint8Array::from(&entropy[..]));

		assert!(verify_signature(
			js_signature.clone(),
			Uint8Array::from(&message[..]),
			js_member.clone()
		)
		.is_truthy());

		let other_message: &[u8; 6] = b"BarFoo";

		assert!(verify_signature(
			js_signature,
			Uint8Array::from(&other_message[..]),
			js_member
		)
		.is_falsy());
	}

	#[wasm_bindgen_test]
	fn test_alias_in_context() {
		let entropy = [5u8; 32];
		let context = b"Context";
		let message = b"FooBar";

		// Get alias via alias_in_context
		let direct_alias = alias_in_context(
			Uint8Array::from(&entropy[..]),
			Uint8Array::from(&context[..]),
		)
		.expect("alias_in_context should succeed");

		// Get alias via one_shot proof
		let members = make_test_members(10);
		let result = one_shot(
			TEST_DOMAIN_SIZE,
			Uint8Array::from(entropy.as_slice()),
			Uint8Array::from(members.encode().to_vec().as_slice()),
			Uint8Array::from(context.as_slice()),
			Uint8Array::from(message.as_slice()),
		)
		.expect("creating one_shot proof should work");

		let proof_alias =
			js_sys::Reflect::get(&result, &JsValue::from_str("alias")).expect("alias should exist");
		let proof_alias = Uint8Array::new(&proof_alias);

		// Aliases from both methods should match
		assert_eq!(direct_alias.to_vec(), proof_alias.to_vec());
	}

	#[wasm_bindgen_test]
	fn test_is_member_valid() {
		let entropy = [0u8; 32];
		let member = member_from_entropy(Uint8Array::from(&entropy[..]));

		// Valid member should return true
		assert!(is_member_valid(member).is_truthy());

		// Garbage bytes should return false
		let garbage = Uint8Array::from(&[0xffu8; 32][..]);
		assert!(is_member_valid(garbage).is_falsy());
	}

	#[wasm_bindgen_test]
	fn test_is_valid() {
		let entropy = [5u8; 32];
		let context = b"Context";
		let message = b"FooBar";

		let members = make_test_members(10);
		let result = one_shot(
			TEST_DOMAIN_SIZE,
			Uint8Array::from(entropy.as_slice()),
			Uint8Array::from(members.encode().to_vec().as_slice()),
			Uint8Array::from(context.as_slice()),
			Uint8Array::from(message.as_slice()),
		)
		.expect("creating one_shot proof should work");

		let alias =
			js_sys::Reflect::get(&result, &JsValue::from_str("alias")).expect("alias should exist");
		let alias = Uint8Array::new(&alias);

		let proof =
			js_sys::Reflect::get(&result, &JsValue::from_str("proof")).expect("proof should exist");
		let proof = Uint8Array::new(&proof);

		// Valid proof with correct alias should return true
		let valid = is_valid(
			TEST_DOMAIN_SIZE,
			proof.clone(),
			Uint8Array::from(&members.encode().to_vec()[..]),
			Uint8Array::from(context.as_slice()),
			alias,
			Uint8Array::from(message.as_slice()),
		)
		.expect("is_valid should not error");
		assert!(valid.is_truthy());

		// Wrong alias should return false
		let wrong_alias = Uint8Array::from(&[0u8; 32][..]);
		let invalid = is_valid(
			TEST_DOMAIN_SIZE,
			proof,
			Uint8Array::from(&members.encode().to_vec()[..]),
			Uint8Array::from(context.as_slice()),
			wrong_alias,
			Uint8Array::from(message.as_slice()),
		)
		.expect("is_valid should not error");
		assert!(invalid.is_falsy());
	}

	#[wasm_bindgen_test]
	fn test_multi_context_proof() {
		let entropy = [5u8; 32];
		let members = make_test_members(10);

		let contexts: Vec<Vec<u8>> = vec![b"Context1".to_vec(), b"Context2".to_vec()];
		let message = b"FooBar";

		let result = create_multi_context(
			TEST_DOMAIN_SIZE,
			Uint8Array::from(&entropy[..]),
			Uint8Array::from(&members.encode().to_vec()[..]),
			Uint8Array::from(&contexts.encode()[..]),
			Uint8Array::from(&message[..]),
		)
		.expect("create_multi_context should work");

		let proof =
			js_sys::Reflect::get(&result, &JsValue::from_str("proof")).expect("proof should exist");
		let proof = Uint8Array::new(&proof);

		let aliases = js_sys::Reflect::get(&result, &JsValue::from_str("aliases"))
			.expect("aliases should exist");
		let aliases = Uint8Array::new(&aliases);

		// Validate multi-context proof
		let validated_aliases = validate_multi_context(
			TEST_DOMAIN_SIZE,
			proof.clone(),
			Uint8Array::from(&members.encode().to_vec()[..]),
			Uint8Array::from(&contexts.encode()[..]),
			Uint8Array::from(&message[..]),
		)
		.expect("validate_multi_context should succeed");

		assert_eq!(aliases.to_vec(), validated_aliases.to_vec());

		// Decode and verify we got 2 aliases
		let decoded_aliases =
			Vec::<Alias>::decode(&mut &validated_aliases.to_vec()[..]).expect("should decode");
		assert_eq!(decoded_aliases.len(), 2);

		// is_valid_multi_context should confirm validity
		let valid = is_valid_multi_context(
			TEST_DOMAIN_SIZE,
			proof,
			Uint8Array::from(&members.encode().to_vec()[..]),
			Uint8Array::from(&contexts.encode()[..]),
			aliases,
			Uint8Array::from(&message[..]),
		)
		.expect("is_valid_multi_context should not error");
		assert!(valid.is_truthy());
	}

	#[wasm_bindgen_test]
	fn test_multi_context_aliases_are_unlinkable() {
		// Same member in different contexts should get different aliases
		let entropy = [5u8; 32];

		let alias_a = alias_in_context(
			Uint8Array::from(&entropy[..]),
			Uint8Array::from(&b"ContextA"[..]),
		)
		.expect("alias_in_context should succeed");

		let alias_b = alias_in_context(
			Uint8Array::from(&entropy[..]),
			Uint8Array::from(&b"ContextB"[..]),
		)
		.expect("alias_in_context should succeed");

		// Aliases in different contexts must be different (unlinkable)
		assert_ne!(alias_a.to_vec(), alias_b.to_vec());

		// Same context must produce the same alias (deterministic)
		let alias_a2 = alias_in_context(
			Uint8Array::from(&entropy[..]),
			Uint8Array::from(&b"ContextA"[..]),
		)
		.expect("alias_in_context should succeed");
		assert_eq!(alias_a.to_vec(), alias_a2.to_vec());
	}

	#[wasm_bindgen_test]
	fn test_different_members_different_aliases() {
		let context = b"SameContext";

		let alias_0 = alias_in_context(
			Uint8Array::from(&[0u8; 32][..]),
			Uint8Array::from(&context[..]),
		)
		.expect("should succeed");

		let alias_1 = alias_in_context(
			Uint8Array::from(&[1u8; 32][..]),
			Uint8Array::from(&context[..]),
		)
		.expect("should succeed");

		// Different members get different aliases in the same context
		assert_ne!(alias_0.to_vec(), alias_1.to_vec());
	}

	#[wasm_bindgen_test]
	fn test_validate_wrong_context_fails() {
		let entropy = [5u8; 32];
		let members = make_test_members(10);
		let context = b"CorrectContext";
		let message = b"FooBar";

		let result = one_shot(
			TEST_DOMAIN_SIZE,
			Uint8Array::from(entropy.as_slice()),
			Uint8Array::from(members.encode().to_vec().as_slice()),
			Uint8Array::from(context.as_slice()),
			Uint8Array::from(message.as_slice()),
		)
		.expect("one_shot should work");

		let proof =
			js_sys::Reflect::get(&result, &JsValue::from_str("proof")).expect("proof should exist");
		let proof = Uint8Array::new(&proof);

		// Validating with a different context should fail
		let wrong_context = b"WrongContext";
		let result = validate(
			TEST_DOMAIN_SIZE,
			proof,
			Uint8Array::from(&members.encode().to_vec()[..]),
			Uint8Array::from(wrong_context.as_slice()),
			Uint8Array::from(message.as_slice()),
		);
		assert!(result.is_err());
	}

	#[wasm_bindgen_test]
	fn test_validate_wrong_message_fails() {
		let entropy = [5u8; 32];
		let members = make_test_members(10);
		let context = b"Context";
		let message = b"CorrectMessage";

		let result = one_shot(
			TEST_DOMAIN_SIZE,
			Uint8Array::from(entropy.as_slice()),
			Uint8Array::from(members.encode().to_vec().as_slice()),
			Uint8Array::from(context.as_slice()),
			Uint8Array::from(message.as_slice()),
		)
		.expect("one_shot should work");

		let proof =
			js_sys::Reflect::get(&result, &JsValue::from_str("proof")).expect("proof should exist");
		let proof = Uint8Array::new(&proof);

		// Validating with a different message should fail
		let wrong_message = b"WrongMessage";
		let result = validate(
			TEST_DOMAIN_SIZE,
			proof,
			Uint8Array::from(&members.encode().to_vec()[..]),
			Uint8Array::from(context.as_slice()),
			Uint8Array::from(wrong_message.as_slice()),
		);
		assert!(result.is_err());
	}

	#[wasm_bindgen_test]
	fn test_validate_wrong_members_fails() {
		let entropy = [5u8; 32];
		let members = make_test_members(10);
		let context = b"Context";
		let message = b"FooBar";

		let result = one_shot(
			TEST_DOMAIN_SIZE,
			Uint8Array::from(entropy.as_slice()),
			Uint8Array::from(members.encode().to_vec().as_slice()),
			Uint8Array::from(context.as_slice()),
			Uint8Array::from(message.as_slice()),
		)
		.expect("one_shot should work");

		let proof =
			js_sys::Reflect::get(&result, &JsValue::from_str("proof")).expect("proof should exist");
		let proof = Uint8Array::new(&proof);

		// Validating with a different members set should fail
		let different_members = make_test_members(8);
		let result = validate(
			TEST_DOMAIN_SIZE,
			proof,
			Uint8Array::from(&different_members.encode().to_vec()[..]),
			Uint8Array::from(context.as_slice()),
			Uint8Array::from(message.as_slice()),
		);
		assert!(result.is_err());
	}

	#[wasm_bindgen_test]
	fn test_invalid_domain_size_rejected() {
		let entropy = [5u8; 32];
		let members = make_test_members(10);
		let context = b"Context";
		let message = b"FooBar";

		// Domain size 13 is not valid
		let result = one_shot(
			13,
			Uint8Array::from(entropy.as_slice()),
			Uint8Array::from(members.encode().to_vec().as_slice()),
			Uint8Array::from(context.as_slice()),
			Uint8Array::from(message.as_slice()),
		);
		assert!(result.is_err());
	}

	#[wasm_bindgen_test]
	fn test_non_member_cannot_prove() {
		// Entropy [99; 32] is NOT in the 10-member ring (which uses entropies [0..10])
		let non_member_entropy = [99u8; 32];
		let members = make_test_members(10);
		let context = b"Context";
		let message = b"FooBar";

		let result = one_shot(
			TEST_DOMAIN_SIZE,
			Uint8Array::from(non_member_entropy.as_slice()),
			Uint8Array::from(members.encode().to_vec().as_slice()),
			Uint8Array::from(context.as_slice()),
			Uint8Array::from(message.as_slice()),
		);
		assert!(result.is_err());
	}

	#[wasm_bindgen_test]
	fn test_empty_context_and_message() {
		let entropy = [5u8; 32];
		let members = make_test_members(10);
		let empty_context = b"";
		let empty_message = b"";

		let result = one_shot(
			TEST_DOMAIN_SIZE,
			Uint8Array::from(entropy.as_slice()),
			Uint8Array::from(members.encode().to_vec().as_slice()),
			Uint8Array::from(empty_context.as_slice()),
			Uint8Array::from(empty_message.as_slice()),
		)
		.expect("empty context and message should work");

		let proof =
			js_sys::Reflect::get(&result, &JsValue::from_str("proof")).expect("proof should exist");
		let proof = Uint8Array::new(&proof);

		let alias = validate(
			TEST_DOMAIN_SIZE,
			proof,
			Uint8Array::from(&members.encode().to_vec()[..]),
			Uint8Array::from(empty_context.as_slice()),
			Uint8Array::from(empty_message.as_slice()),
		)
		.expect("validate should succeed with empty context/message");

		assert_eq!(alias.to_vec().len(), 32);
	}

	#[wasm_bindgen_test]
	fn test_members_root_and_intermediate() {
		let members = make_test_members(10);
		let encoded = Uint8Array::from(&members.encode().to_vec()[..]);

		let root =
			members_root(TEST_DOMAIN_SIZE, encoded.clone()).expect("members_root should succeed");
		assert_eq!(root.to_vec().len(), 768);

		let inter = members_intermediate(TEST_DOMAIN_SIZE, encoded)
			.expect("members_intermediate should succeed");
		assert_eq!(inter.to_vec().len(), 848);
	}

	#[wasm_bindgen_test]
	fn test_members_root_deterministic() {
		let members = make_test_members(10);
		let encoded = Uint8Array::from(&members.encode().to_vec()[..]);

		let root1 =
			members_root(TEST_DOMAIN_SIZE, encoded.clone()).expect("should succeed");
		let root2 =
			members_root(TEST_DOMAIN_SIZE, encoded).expect("should succeed");

		// Same members, same domain -> same commitment
		assert_eq!(root1.to_vec(), root2.to_vec());
	}

	#[wasm_bindgen_test]
	fn test_batch_validate() {
		let members = make_test_members(10);
		let encoded_members = members.encode().to_vec();

		// Create two proofs from different members with different contexts
		let entropy_a = [3u8; 32];
		let context_a = b"ContextA";
		let message_a = b"MessageA";

		let result_a = one_shot(
			TEST_DOMAIN_SIZE,
			Uint8Array::from(entropy_a.as_slice()),
			Uint8Array::from(encoded_members.as_slice()),
			Uint8Array::from(context_a.as_slice()),
			Uint8Array::from(message_a.as_slice()),
		)
		.expect("first one_shot should work");

		let entropy_b = [7u8; 32];
		let context_b = b"ContextB";
		let message_b = b"MessageB";

		let result_b = one_shot(
			TEST_DOMAIN_SIZE,
			Uint8Array::from(entropy_b.as_slice()),
			Uint8Array::from(encoded_members.as_slice()),
			Uint8Array::from(context_b.as_slice()),
			Uint8Array::from(message_b.as_slice()),
		)
		.expect("second one_shot should work");

		// Extract proofs
		let proof_a = js_sys::Reflect::get(&result_a, &JsValue::from_str("proof")).unwrap();
		let proof_a = Uint8Array::new(&proof_a);
		let proof_a_decoded: <Bvv as GenerateVerifiable>::Proof =
			Decode::decode(&mut &proof_a.to_vec()[..]).unwrap();

		let proof_b = js_sys::Reflect::get(&result_b, &JsValue::from_str("proof")).unwrap();
		let proof_b = Uint8Array::new(&proof_b);
		let proof_b_decoded: <Bvv as GenerateVerifiable>::Proof =
			Decode::decode(&mut &proof_b.to_vec()[..]).unwrap();

		// Encode as Vec<(Proof, Vec<u8>, Vec<u8>)> for batch_validate
		let tuples: Vec<(<Bvv as GenerateVerifiable>::Proof, Vec<u8>, Vec<u8>)> = vec![
			(proof_a_decoded, context_a.to_vec(), message_a.to_vec()),
			(proof_b_decoded, context_b.to_vec(), message_b.to_vec()),
		];
		let encoded_tuples = tuples.encode();

		let aliases = batch_validate(
			TEST_DOMAIN_SIZE,
			Uint8Array::from(encoded_members.as_slice()),
			Uint8Array::from(encoded_tuples.as_slice()),
		)
		.expect("batch_validate should succeed");

		// Should get 2 aliases back
		let decoded_aliases =
			Vec::<Alias>::decode(&mut &aliases.to_vec()[..]).expect("should decode aliases");
		assert_eq!(decoded_aliases.len(), 2);

		// The aliases should match the individual proof aliases
		let alias_a = js_sys::Reflect::get(&result_a, &JsValue::from_str("alias")).unwrap();
		let alias_a = Uint8Array::new(&alias_a);
		assert_eq!(decoded_aliases[0].to_vec(), alias_a.to_vec());

		let alias_b = js_sys::Reflect::get(&result_b, &JsValue::from_str("alias")).unwrap();
		let alias_b = Uint8Array::new(&alias_b);
		assert_eq!(decoded_aliases[1].to_vec(), alias_b.to_vec());
	}

	#[wasm_bindgen_test]
	fn test_signature_wrong_member_fails() {
		let entropy = [23u8; 32];
		let message = b"FooBar";

		let signature = sign(
			Uint8Array::from(&entropy[..]),
			Uint8Array::from(&message[..]),
		)
		.expect("sign should work");

		// Verify with a different member's key
		let other_member = member_from_entropy(Uint8Array::from(&[99u8; 32][..]));
		assert!(verify_signature(
			signature,
			Uint8Array::from(&message[..]),
			other_member,
		)
		.is_falsy());
	}

	#[wasm_bindgen_test]
	fn test_domain12_proof() {
		let entropy = [3u8; 32];
		let members = make_test_members(10);
		let context = b"Context";
		let message = b"FooBar";

		// Create and validate with Domain12
		let result = one_shot(
			12,
			Uint8Array::from(entropy.as_slice()),
			Uint8Array::from(members.encode().to_vec().as_slice()),
			Uint8Array::from(context.as_slice()),
			Uint8Array::from(message.as_slice()),
		)
		.expect("one_shot with domain 12 should work");

		let proof =
			js_sys::Reflect::get(&result, &JsValue::from_str("proof")).expect("proof should exist");
		let proof = Uint8Array::new(&proof);
		let alias =
			js_sys::Reflect::get(&result, &JsValue::from_str("alias")).expect("alias should exist");
		let alias = Uint8Array::new(&alias);

		let validated_alias = validate(
			12,
			proof,
			Uint8Array::from(&members.encode().to_vec()[..]),
			Uint8Array::from(context.as_slice()),
			Uint8Array::from(message.as_slice()),
		)
		.expect("validate with domain 12 should succeed");

		assert_eq!(alias.to_vec(), validated_alias.to_vec());
	}

	#[wasm_bindgen_test]
	fn test_single_member_ring() {
		let entropy = [0u8; 32];
		let members = make_test_members(1);
		let context = b"Context";
		let message = b"FooBar";

		let result = one_shot(
			TEST_DOMAIN_SIZE,
			Uint8Array::from(entropy.as_slice()),
			Uint8Array::from(members.encode().to_vec().as_slice()),
			Uint8Array::from(context.as_slice()),
			Uint8Array::from(message.as_slice()),
		)
		.expect("one_shot with single member should work");

		let proof =
			js_sys::Reflect::get(&result, &JsValue::from_str("proof")).expect("proof should exist");
		let proof = Uint8Array::new(&proof);

		let validated = validate(
			TEST_DOMAIN_SIZE,
			proof,
			Uint8Array::from(&members.encode().to_vec()[..]),
			Uint8Array::from(context.as_slice()),
			Uint8Array::from(message.as_slice()),
		)
		.expect("validate single member ring should succeed");

		assert_eq!(validated.to_vec().len(), 32);
	}

	#[wasm_bindgen_test]
	fn test_cross_domain_proof_fails() {
		let entropy = [5u8; 32];
		let members = make_test_members(10);
		let context = b"Context";
		let message = b"FooBar";

		// Create proof with domain 11
		let result = one_shot(
			11,
			Uint8Array::from(entropy.as_slice()),
			Uint8Array::from(members.encode().to_vec().as_slice()),
			Uint8Array::from(context.as_slice()),
			Uint8Array::from(message.as_slice()),
		)
		.expect("one_shot should work");

		let proof =
			js_sys::Reflect::get(&result, &JsValue::from_str("proof")).expect("proof should exist");
		let proof = Uint8Array::new(&proof);

		// Validate with domain 12 should fail
		let result = validate(
			12,
			proof,
			Uint8Array::from(&members.encode().to_vec()[..]),
			Uint8Array::from(context.as_slice()),
			Uint8Array::from(message.as_slice()),
		);
		assert!(result.is_err());
	}
}
