import { sign, verify_signature, member_from_entropy } from 'verifiablejs/bundler';

const button = document.getElementById('runExample')!;
const output = document.getElementById('output')!;

button.addEventListener('click', async () => {
  try {
    output.textContent = 'Running verifiablejs example...\n\n';

    // Generate a member from entropy
    const entropy = new Uint8Array(32);
    crypto.getRandomValues(entropy);

    output.textContent += 'Generating member from entropy...\n';
    const member = member_from_entropy(entropy);
    output.textContent += `Member generated! Length: ${member.length} bytes\n\n`;

    // Sign a message
    const message = new TextEncoder().encode("Hello from verifiablejs!");
    output.textContent += 'Signing message...\n';
    const signature = sign(entropy, message);
    output.textContent += `Signature created! Length: ${signature.length} bytes\n\n`;

    // Verify the signature
    output.textContent += 'Verifying signature...\n';
    const isValid = verify_signature(signature, message, member);
    output.textContent += `Signature valid: ${isValid}\n\n`;

    // Try with wrong message (should fail)
    const wrongMessage = new TextEncoder().encode("Wrong message");
    output.textContent += 'Verifying with wrong message...\n';
    const isInvalid = verify_signature(signature, wrongMessage, member);
    output.textContent += `Signature valid for wrong message: ${isInvalid}\n\n`;

    output.textContent += 'All operations completed successfully!';
  } catch (error) {
    output.textContent += `\nError: ${error instanceof Error ? error.message : String(error)}\n${error instanceof Error ? error.stack : ''}`;
  }
});
