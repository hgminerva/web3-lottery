import { ContractPromise } from "@polkadot/api-contract";
import { Keyring } from "@polkadot/keyring";
import fs from "fs";
import 'dotenv/config';

import { decode } from "./decode.js";
import { colors } from "./colors.js";

export async function closeDraw(api, draw_number) {

  const CONTRACT_ADDRESS = process.env.CONTRACT_ADDRESS;
  const CONTRACT_ABI_PATH = process.env.CONTRACT_ABI_PATH;
  const ALICE = process.env.ALICE;
  const BOB = process.env.BOB;

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

  let event_message = "";

  await new Promise(async (resolve, reject) => {
    const unsub = await contract.tx
      .closeDraw({storageDepositLimit, gasLimit}, draw_number)
      .signAndSend(bob, ({ status, events, dispatchError }) => {
        console.log(colors.darkGray(`Closing Draw Status: ${status?.type}`)); 
        if(events?.length > 0) {
          events.forEach(({ event }) => {
            if (event.section === "contracts" && event.method === "ContractEmitted") {
              event_message = decode(event.data);

              unsub();
              resolve();
            }
          });
        }
      });
  });

  return event_message;
}
