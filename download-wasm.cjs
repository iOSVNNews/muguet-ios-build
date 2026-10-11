const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const crypto = require('node:crypto');

const url = 'https://hml.sinimediata.com.br/b/8b0b5899ed/game.wasm';
const total = 63201802;
const chunkSize = 65536;
const expectedHash = '11ca8d2c04c5e843d18ff4aea4899d72c86973c6b031df334e67c446b2ae83e0';
const destination = process.env.MUGUET_WASM_DEST || path.join(os.homedir(), 'Downloads', 'GTA-V-game.wasm');
const partial = destination + '.part';
const checkpoint = partial + '.json';

async function main() {
  if (fs.existsSync(destination)) {
    const hash = crypto.createHash('sha256').update(fs.readFileSync(destination)).digest('hex');
    if (hash !== expectedHash) throw new Error('Existing destination has a different checksum; preserving it.');
    console.log(JSON.stringify({ destination, status: 'already validated', sha256: hash }));
    return;
  }
  let finished = new Set();
  if (fs.existsSync(partial) && fs.existsSync(checkpoint)) {
    const saved = JSON.parse(fs.readFileSync(checkpoint, 'utf8'));
    if (saved.url === url && saved.total === total && saved.chunkSize === chunkSize) finished = new Set(saved.finished);
  }
  const handle = await fs.promises.open(partial, fs.existsSync(partial) ? 'r+' : 'w+');
  await handle.truncate(total);
  const count = Math.ceil(total / chunkSize);
  const pending = Array.from({ length: count }, (_, i) => i).filter(i => !finished.has(i));
  let next = 0;
  const save = () => fs.writeFileSync(checkpoint, JSON.stringify({ url, total, chunkSize, finished: [...finished] }));
  async function worker() {
    while (next < pending.length) {
      const index = pending[next++];
      const start = index * chunkSize;
      const end = Math.min(total - 1, start + chunkSize - 1);
      let successful = false;
      for (let attempt = 1; attempt <= 3; attempt++) {
        try {
          const response = await fetch(url, {
            headers: { Range: `bytes=${start}-${end}` },
            signal: AbortSignal.timeout(20000),
          });
          if (response.status !== 206 || response.headers.get('content-range') !== `bytes ${start}-${end}/${total}`) {
            await response.body?.cancel();
            throw new Error(`Invalid range response (${response.status})`);
          }
          const bytes = Buffer.from(await response.arrayBuffer());
          if (bytes.length !== end - start + 1) throw new Error('Incomplete chunk');
          const written = await handle.write(bytes, 0, bytes.length, start);
          if (written.bytesWritten !== bytes.length) throw new Error('Incomplete local write');
          finished.add(index);
          if (finished.size % 64 === 0 || finished.size === count) {
            save();
            console.log(`Downloaded ${finished.size}/${count} chunks (${Math.round(finished.size * 100 / count)}%)`);
          }
          successful = true;
          break;
        } catch (error) {
          if (attempt === 3) throw new Error(`Chunk ${index}: ${error.message}`);
          await new Promise(resolve => setTimeout(resolve, 500 * attempt));
        }
      }
      if (!successful) throw new Error(`Chunk ${index} failed`);
    }
  }
  console.log(`Downloading ${pending.length} remaining chunks in four streams`);
  const results = await Promise.allSettled(Array.from({ length: 4 }, worker));
  save();
  await handle.close();
  const failure = results.find(result => result.status === 'rejected');
  if (failure) throw failure.reason;
  const payload = fs.readFileSync(partial);
  const hash = crypto.createHash('sha256').update(payload).digest('hex');
  if (payload.length !== total || payload.subarray(0, 8).toString('hex') !== '0061736d01000000' || hash !== expectedHash) {
    throw new Error(`Complete file validation failed: SHA-256 ${hash}`);
  }
  fs.renameSync(partial, destination);
  fs.unlinkSync(checkpoint);
  console.log(JSON.stringify({ destination, size: payload.length, sha256: hash, validation: 'pass' }));
}
main().catch(error => { console.error(error.message); process.exitCode = 1; });
