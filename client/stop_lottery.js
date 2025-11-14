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

await new Promise(async (resolve, reject) => {
  const unsub = await contract.tx
    .stop({ storageDepositLimit, gasLimit })
    .signAndSend(alice, ({ status, events, dispatchError }) => {    
      console.log("Status:", status?.type);
      if(events?.length > 0) {
        events.forEach(({ event }) => {
           const { section, method, data } = event;
           if (section === "system") {
            console.log(method);
            unsub(); 
            resolve();          
           }
        });
      }
    });
});

process.exit(0);
