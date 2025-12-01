import { sign, verify_signature, member_from_entropy, one_shot, validate } from 'verifiablejs/nodejs';
import { compactAddLength, u8aConcat } from '@polkadot/util';

// Type for one_shot result
interface OneShotResult {
  proof: Uint8Array;
  alias: Uint8Array;
  member: Uint8Array;
  members: Uint8Array;
  context: Uint8Array;
  message: Uint8Array;
}

// Helper function to encode members list using SCALE codec
function encodeMembers(members: Uint8Array[]): Uint8Array {
  // SCALE encode Vec<Member> where each Member is 32 bytes
  // Compact encode the length (number of items, not bytes)
  const length = members.length;
  let compactLength: Uint8Array;

  if (length < 64) {
    // Single byte mode: length << 2
    compactLength = new Uint8Array([length << 2]);
  } else if (length < 16384) {
    // Two byte mode: (length << 2) | 0b01
    compactLength = new Uint8Array([
      ((length & 0x3f) << 2) | 0b01,
      (length >> 6) & 0xff
    ]);
  } else {
    throw new Error('Too many members');
  }

  return u8aConcat(compactLength, ...members);
}

async function runExample() {
  try {
    console.log('Running verifiablejs examples...\n');

    // === Example 1: Signature Creation and Verification ===
    console.log('=== Signature Example ===');

    const entropy = new Uint8Array(32);
    crypto.getRandomValues(entropy);

    console.log('Generating member from entropy...');
    const member = member_from_entropy(entropy);
    console.log(`Member generated! Length: ${member.length} bytes\n`);

    const message = new TextEncoder().encode("Hello from verifiablejs!");
    console.log('Signing message...');
    const signature = sign(entropy, message);
    console.log(`Signature created! Length: ${signature.length} bytes\n`);

    console.log('Verifying signature...');
    const isValid = verify_signature(signature, message, member);
    console.log(`Signature valid: ${isValid}\n`);

    const wrongMessage = new TextEncoder().encode("Wrong message");
    console.log('Verifying with wrong message...');
    const isInvalid = verify_signature(signature, wrongMessage, member);
    console.log(`Signature valid for wrong message: ${isInvalid}\n`);

    // === Example 2: Ring Proof Creation and Validation ===
    console.log('=== Ring Proof Example ===');

    // Create a ring of 10 members
    console.log('Creating ring of 10 members...');
    const membersList: Uint8Array[] = [];
    for (let i = 0; i < 10; i++) {
      const entropy = new Uint8Array(32).fill(i);
      membersList.push(member_from_entropy(entropy));
    }
    console.log(`Created ${membersList.length} members\n`);

    // Use member at index 5 to create proof
    const proverEntropy = new Uint8Array(32).fill(5);
    const encodedMembers = encodeMembers(membersList);

    const context = new TextEncoder().encode("test-context");
    const proofMessage = new TextEncoder().encode("test-message");

    console.log('Creating ring proof...');
    const result = one_shot(proverEntropy, encodedMembers, context, proofMessage) as OneShotResult;
    console.log('Proof created!');
    console.log(`Proof length: ${result.proof.length} bytes`);
    console.log(`Alias length: ${result.alias.length} bytes\n`);

    console.log('Validating ring proof...');
    const validatedAlias = validate(result.proof, encodedMembers, context, proofMessage);
    console.log(`Proof validated! Alias length: ${validatedAlias.length} bytes`);

    // Check if aliases match
    const aliasesMatch = result.alias.length === validatedAlias.length &&
      result.alias.every((byte: number, i: number) => byte === validatedAlias[i]);
    console.log(`Aliases match: ${aliasesMatch}\n`);

    console.log('All operations completed successfully!');
  } catch (error) {
    console.error('Error:', error);
    if (error instanceof Error) {
      console.error('Message:', error.message);
      console.error('Stack:', error.stack);
    }
  }
}

runExample();
