import { ApiPromise, WsProvider } from "@polkadot/api";
import { ContractPromise } from "@polkadot/api-contract";
import { Keyring } from "@polkadot/keyring";
import fs from "fs";
import 'dotenv/config';

import { decode } from "./decode.js";

const WS_ENDPOINT = process.env.WS_ENDPOINT;
const CONTRACT_ADDRESS = process.env.CONTRACT_ADDRESS;
const CONTRACT_ABI_PATH = process.env.CONTRACT_ABI_PATH;
const ALICE = process.env.ALICE;
const BOB = process.env.BOB;

/// Test the blockchain connection
console.log("Connecting to blockchain...");
const wsProvider = new WsProvider(WS_ENDPOINT);
const api = await ApiPromise.create({ provider: wsProvider });
console.log("Connected to:", (await api.rpc.system.chain()).toHuman());

const abiJSON = JSON.parse(fs.readFileSync(CONTRACT_ABI_PATH, "utf8"));
const contract = new ContractPromise(api, abiJSON, CONTRACT_ADDRESS);

const keyring = new Keyring({ type: "sr25519" });
const alice = keyring.addFromUri(ALICE);
const bob = keyring.addFromUri(BOB);

const gasLimit = api.registry.createType('WeightV2', {
          refTime: 300000000000,
          proofSize: 500000,
});
const storageDepositLimit = null;

const draw_number = 1;
const winning_number = 555;

await new Promise(async (resolve, reject) => {

  const unsub = await contract.tx
    .overrideDraw({ storageDepositLimit, gasLimit }, 
      draw_number,
      winning_number,
    ).signAndSend(bob, ({ status, events, dispatchError }) => {    
      console.log("Status:", status?.type);
      if(events?.length > 0) {
        events.forEach(({ event }) => {
          if (event.section === "contracts" && event.method === "ContractEmitted") {
            console.log(decode(event.data));
            unsub();
            resolve();
          }
        });
      }
  });

});

process.exit(0);