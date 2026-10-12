const assert = require('node:assert/strict');
const dgram = require('node:dgram');
const {spawn} = require('node:child_process');
const {setTimeout: sleep} = require('node:timers/promises');

async function main() {
  const binary = process.argv[2];
  assert(binary, 'Provide the compiled muguet-server path');
  const probe = dgram.createSocket('udp4');
  await new Promise(resolve => probe.bind(0, '127.0.0.1', resolve));
  const port = probe.address().port;
  await new Promise(resolve => probe.close(resolve));
  const server = spawn(binary, [String(port), '2'], {stdio: ['ignore', 'pipe', 'pipe']});
  let output = '';
  server.stdout.on('data', data => { output += data; });
  const clients = [];
  const send = (client, kind, payload = Buffer.alloc(0)) => new Promise((resolve, reject) => {
    client.socket.send(Buffer.concat([Buffer.from([0x47, kind]), payload]), port, '127.0.0.1', error => error ? reject(error) : resolve());
  });
  async function client() {
    const socket = dgram.createSocket('udp4');
    const queue = [];
    socket.on('message', message => queue.push(message));
    await new Promise(resolve => socket.bind(0, '127.0.0.1', resolve));
    const result = {socket, queue};
    clients.push(result);
    return result;
  }
  async function packet(client, kind) {
    const deadline = Date.now() + 3000;
    while (Date.now() < deadline) {
      const index = client.queue.findIndex(message => message[0] === 0x47 && message[1] === kind);
      if (index >= 0) return client.queue.splice(index, 1)[0].subarray(2);
      await sleep(10);
    }
    throw new Error(`No server response for packet ${kind}: ${output}`);
  }
  try {
    const deadline = Date.now() + 3000;
    while (!output.includes('listening on UDP') && Date.now() < deadline) await sleep(10);
    assert(output.includes('listening on UDP'), 'Server did not start');
    const [a, b, c] = await Promise.all([client(), client(), client()]);
    await send(a, 1, Buffer.from('Alice'));
    const aid = (await packet(a, 2))[0];
    await send(b, 1, Buffer.from('Bob'));
    const bid = (await packet(b, 2))[0];
    assert.notEqual(aid, bid);
    assert.equal((await packet(a, 4)).subarray(1).toString(), 'Bob');
    const state = Buffer.from([10, 20, 30]);
    await send(a, 3, state);
    assert.deepEqual(await packet(b, 3), Buffer.concat([Buffer.from([aid]), state]));
    const shot = Buffer.from([4, 5, 6]);
    await send(b, 9, shot);
    assert.deepEqual(await packet(a, 9), Buffer.concat([Buffer.from([bid]), shot]));
    await send(c, 1, Buffer.from('Carol'));
    assert.equal((await packet(c, 7)).length, 0);
    await send(a, 6);
    assert.equal((await packet(b, 5))[0], aid);
    await send(c, 1, Buffer.from('Carol'));
    assert((await packet(c, 2))[0] > 0);
    console.log('UDP integration: two players join, names/state/shots relay, room capacity and leave/rejoin PASS');
  } finally {
    clients.forEach(client => client.socket.close());
    server.kill();
  }
}
main().catch(error => { console.error(error); process.exitCode = 1; });
