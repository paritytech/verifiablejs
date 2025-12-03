import { sign, verify_signature, member_from_entropy, one_shot, validate } from 'verifiablejs/bundler';
import { u8aConcat } from '@polkadot/util';

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

const button = document.getElementById('runExample')!;
const output = document.getElementById('output')!;

button.addEventListener('click', async () => {
  try {
    output.textContent = 'Running verifiablejs examples...\n\n';

    // === Example 1: Signature Creation and Verification ===
    output.textContent += '=== Signature Example ===\n';

    const entropy = new Uint8Array(32);
    crypto.getRandomValues(entropy);

    output.textContent += 'Generating member from entropy...\n';
    const member = member_from_entropy(entropy);
    output.textContent += `Member generated! Length: ${member.length} bytes\n\n`;

    const message = new TextEncoder().encode("Hello from verifiablejs!");
    output.textContent += 'Signing message...\n';
    const signature = sign(entropy, message);
    output.textContent += `Signature created! Length: ${signature.length} bytes\n\n`;

    output.textContent += 'Verifying signature...\n';
    const isValid = verify_signature(signature, message, member);
    output.textContent += `Signature valid: ${isValid}\n\n`;

    const wrongMessage = new TextEncoder().encode("Wrong message");
    output.textContent += 'Verifying with wrong message...\n';
    const isInvalid = verify_signature(signature, wrongMessage, member);
    output.textContent += `Signature valid for wrong message: ${isInvalid}\n\n`;

    // === Example 2: Ring Proof Creation and Validation ===
    output.textContent += '=== Ring Proof Example ===\n';

    // Create a ring of 10 members
    output.textContent += 'Creating ring of 10 members...\n';
    const membersList: Uint8Array[] = [];
    for (let i = 0; i < 10; i++) {
      const entropy = new Uint8Array(32).fill(i);
      membersList.push(member_from_entropy(entropy));
    }
    output.textContent += `Created ${membersList.length} members\n\n`;

    // Use member at index 5 to create proof
    const proverEntropy = new Uint8Array(32).fill(5);
    const encodedMembers = encodeMembers(membersList);

    const context = new TextEncoder().encode("test-context");
    const proofMessage = new TextEncoder().encode("test-message");

    output.textContent += 'Creating ring proof...\n';
    const result = one_shot(proverEntropy, encodedMembers, context, proofMessage);
    output.textContent += `Proof created!\n`;
    output.textContent += `Proof length: ${result.proof.length} bytes\n`;
    output.textContent += `Alias length: ${result.alias.length} bytes\n\n`;

    output.textContent += 'Validating ring proof...\n';
    const validatedAlias = validate(result.proof, encodedMembers, context, proofMessage);
    output.textContent += `Proof validated! Alias length: ${validatedAlias.length} bytes\n`;

    // Check if aliases match
    const aliasesMatch = result.alias.length === validatedAlias.length &&
      result.alias.every((byte: number, i: number) => byte === validatedAlias[i]);
    output.textContent += `Aliases match: ${aliasesMatch}\n\n`;

    output.textContent += 'All operations completed successfully!';
  } catch (error) {
    output.textContent += `\nError: ${error instanceof Error ? error.message : String(error)}\n${error instanceof Error ? error.stack : ''}`;
  }
});
