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
const CHARLIE = process.env.CHARLIE;

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
const charlie = keyring.addFromUri(CHARLIE);

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

let success = false;

/// Send the token to the lottery contract
const txHash = await new Promise(async (resolve, reject) => {

  const tx = api.tx.assets.transferKeepAlive(
    asset_id,
    recipient,
    formattedAmount
  );

  const hash = tx.hash.toHex();

  const unsub = await tx.signAndSend(charlie, ({ status, events, data  }) => {
    console.log("Status:", status?.type);
    if(events?.length > 0) {
      events.forEach(({ event }) => {
        if (event.section === "assets" && event.method === "Transferred") {
          success = true;
          console.log("Transfer successful.");
          unsub();
          resolve(hash);
        }
      });
    }
  });
});

/// If successful, record the bet
const draw_number = 1;
const bet_number = 555;
const bettor = charlie.address; 
const upline = bob.address; 

if (success) {
  await new Promise(async (resolve, reject) => {
    const unsub = await contract.tx
      .addBet({ storageDepositLimit, gasLimit }, 
        draw_number,
        bet_number,
        bettor,
        upline,
        txHash,
      ).signAndSend(bob, ({ status, events }) => {    
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
}



process.exit(0);