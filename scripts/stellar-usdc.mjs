import { Asset, Networks } from '@stellar/stellar-sdk';

// Official Circle asset identities, not assets trusted by ticker alone.
// Mainnet metadata is informational; the smoke runner supports testnet only.
export const CIRCLE_SOURCE =
  'https://developers.circle.com/stablecoins/usdc-contract-addresses';
export const STELLAR_USDC = Object.freeze({
  testnet: Object.freeze({
    code: 'USDC', decimals: 7,
    issuer: 'GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5',
    sac: 'CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA',
  }),
  mainnet: Object.freeze({
    code: 'USDC', decimals: 7,
    issuer: 'GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN',
    sac: 'CCW67TSZV3SSS2HXMBQ5JFGCKJNXKZM7UQUWUZPUTHXSTZLEO7SJMI75',
  }),
});

export function testnetUsdcAsset(networkPassphrase) {
  if (networkPassphrase !== Networks.TESTNET) {
    throw new Error('USDC smoke runner permits Stellar Testnet only');
  }
  const config = STELLAR_USDC.testnet;
  const asset = new Asset(config.code, config.issuer);
  if (asset.contractId(networkPassphrase) !== config.sac) {
    throw new Error('Circle USDC issuer/SAC binding mismatch');
  }
  return asset;
}

export function assertUsdcTransferDeltas(before, after, amount) {
  if (amount <= 0n || before.sender - after.sender !== amount ||
      after.recipient - before.recipient !== amount) {
    throw new Error('USDC balance deltas do not reconcile');
  }
}
