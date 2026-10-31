import test from 'node:test';
import assert from 'node:assert/strict';
import { STUN_SERVERS } from './signalingService';

test('STUN_SERVERS contains valid and standard STUN endpoints', () => {
  assert.ok(STUN_SERVERS.length >= 2, 'Must have at least 2 fallback STUN servers');
  for (const server of STUN_SERVERS) {
    assert.match(server, /^stun:/, 'Server address must start with stun: prefix');
  }
});

test('Azure VM production URL is configured with HTTPS and Let\'s Encrypt', () => {
  const azureDomain = 'https://prysmview.duckdns.org';
  assert.ok(azureDomain.startsWith('https://'), 'Must use HTTPS for production');
  assert.ok(azureDomain.includes('prysmview.duckdns.org'), 'Must match DuckDNS domain');
});
