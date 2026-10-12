const fs = require('node:fs');
const crypto = require('node:crypto');
const { pipeline } = require('node:stream/promises');

async function main() {
  const [mode, input, output, keyFile] = process.argv.slice(2);
  if (!['encrypt', 'decrypt'].includes(mode) || !input || !output) throw new Error('Usage: ipa-artifact.cjs encrypt|decrypt input output [key-file]');
  const keyHex = (keyFile ? fs.readFileSync(keyFile, 'utf8') : process.env.MUGUET_ARTIFACT_KEY || '').trim();
  if (!/^[0-9a-f]{64}$/i.test(keyHex)) throw new Error('A 32-byte artifact key is required');
  if (fs.existsSync(output)) throw new Error('Output already exists; preserving it');
  const key = Buffer.from(keyHex, 'hex');
  const signature = Buffer.from('MUGUET1\n');
  const temporary = output + '.part';
  if (mode === 'encrypt') {
    const nonce = crypto.randomBytes(12);
    fs.writeFileSync(temporary, Buffer.concat([signature, nonce]));
    const cipher = crypto.createCipheriv('aes-256-gcm', key, nonce);
    await pipeline(fs.createReadStream(input), cipher, fs.createWriteStream(temporary, { flags: 'a' }));
    fs.appendFileSync(temporary, cipher.getAuthTag());
  } else {
    const size = fs.statSync(input).size;
    if (size < 36) throw new Error('Invalid encrypted artifact');
    const fd = fs.openSync(input, 'r');
    const header = Buffer.alloc(20);
    const tag = Buffer.alloc(16);
    fs.readSync(fd, header, 0, 20, 0);
    fs.readSync(fd, tag, 0, 16, size - 16);
    fs.closeSync(fd);
    if (!header.subarray(0, 8).equals(signature)) throw new Error('Invalid artifact signature');
    const cipher = crypto.createDecipheriv('aes-256-gcm', key, header.subarray(8));
    cipher.setAuthTag(tag);
    await pipeline(fs.createReadStream(input, { start: 20, end: size - 17 }), cipher, fs.createWriteStream(temporary));
  }
  fs.renameSync(temporary, output);
  console.log(JSON.stringify({ mode, output, bytes: fs.statSync(output).size, authentication: 'AES-256-GCM' }));
}
main().catch(error => { console.error(error.message); process.exitCode = 1; });
