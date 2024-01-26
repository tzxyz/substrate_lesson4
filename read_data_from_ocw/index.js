import { ApiPromise } from "@polkadot/api";
import { u8aToString } from "@polkadot/util";

async function main() {
  const api = await ApiPromise.create();
  const blockNumber = "1";
  const value = await api.rpc.offchain.localStorageGet(
    "PERSISTENT",
    blockNumber
  );
  const hexValue = value.toHex();
  const u8aValue = new Uint8Array(
    (hexValue.match(/.{1,2}/g) || []).map((byte) => parseInt(byte, 16))
  );
  const stringValue = u8aToString(u8aValue);
  console.log("value in offchain storage: ", stringValue);
}

main().catch((error) => {
  console.error(error);
  process.exit(-1);
});