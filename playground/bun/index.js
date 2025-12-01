import { sign, verify_signature, member_from_entropy } from 'verifiablejs/nodejs';

async function runExample() {
  try {
    console.log('Running verifiablejs example...\n');

    // Generate a member from entropy
    const entropy = new Uint8Array(32);
    crypto.getRandomValues(entropy);

    console.log('Generating member from entropy...');
    const member = member_from_entropy(entropy);
    console.log(`Member generated! Length: ${member.length} bytes\n`);

    // Sign a message
    const message = new TextEncoder().encode("Hello from verifiablejs!");
    console.log('Signing message...');
    const signature = sign(entropy, message);
    console.log(`Signature created! Length: ${signature.length} bytes\n`);

    // Verify the signature
    console.log('Verifying signature...');
    const isValid = verify_signature(signature, message, member);
    console.log(`Signature valid: ${isValid}\n`);

    // Try with wrong message (should fail)
    const wrongMessage = new TextEncoder().encode("Wrong message");
    console.log('Verifying with wrong message...');
    const isInvalid = verify_signature(signature, wrongMessage, member);
    console.log(`Signature valid for wrong message: ${isInvalid}\n`);

    console.log('All operations completed successfully!');
  } catch (error) {
    console.error('Error:', error);
    if (error && error.message) {
      console.error('Message:', error.message);
    }
    if (error && error.stack) {
      console.error('Stack:', error.stack);
    }
  }
}

runExample();
