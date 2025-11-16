import { ApiPromise, WsProvider } from "@polkadot/api";
import { ContractPromise } from "@polkadot/api-contract";
import { Keyring } from "@polkadot/keyring";
import fs from "fs";
import 'dotenv/config';

const WS_ENDPOINT = process.env.WS_ENDPOINT;
const CONTRACT_ADDRESS = process.env.CONTRACT_ADDRESS;
const CONTRACT_ABI_PATH = process.env.CONTRACT_ABI_PATH;
const ACCOUNT = process.env.ACCOUNT;

/// Test the blockchain connection
console.log("Connecting to blockchain...");
const wsProvider = new WsProvider(WS_ENDPOINT);
const api = await ApiPromise.create({ provider: wsProvider });
console.log("Connected to:", (await api.rpc.system.chain()).toHuman());

const abiJSON = JSON.parse(fs.readFileSync(CONTRACT_ABI_PATH, "utf8"));
const contract = new ContractPromise(api, abiJSON, CONTRACT_ADDRESS);

const keyring = new Keyring({ type: "sr25519" });
const alice = keyring.addFromUri(ACCOUNT);

const gasLimit = api.registry.createType('WeightV2', {
          refTime: 300000000000,
          proofSize: 500000,
});
const storageDepositLimit = null;

const recipient = CONTRACT_ADDRESS; 
const amount = 500000; // 0.5 (1_000_000)
const asset_id = 1984

const formattedAmount = api.createType(
      "Compact<u128>",
      amount
);

await new Promise(async (resolve, reject) => {
  const unsub = await api.tx.assets.transferKeepAlive(
    asset_id,
    recipient,
    formattedAmount
  ).signAndSend(alice, ({ status, dispatchError, events, txHash }) => {
    console.log("Status:", status.type);
    //console.log("Event:", events);
    //console.log("Error:", dispatchError);
    if (status.isInBlock) {
      const isSuccess = events.some(({ event }) =>
        event.section === "system" && event.method === "ExtrinsicSuccess"
      );

      if (isSuccess) {
        console.log(txHash.toHex());
      } else {
        const failure = events.find(({ event }) =>
          event.section === "system" && event.method === "ExtrinsicFailed"
        );
        const [dispatchError] = failure.event.data;
        if (dispatchError.isModule) {
          const decoded = api.registry.findMetaError(dispatchError.asModule);
          console.log(decoded.section);
          console.log(decoded.name);
          console.log(decoded.docs.join(" "));
        }
      }

      unsub();
      resolve();
    }
  });
});

process.exit(0);