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

/// Setup the lottery
const startingBlock = 800;
const dailyTotalBlocks = 14400;  // e.g., 1 day in blocks
const maximumDraws = 2;
const maximumBets = 1000;

await contract.tx
  .setup({ storageDepositLimit, gasLimit }, 
    startingBlock,
    dailyTotalBlocks,
    maximumDraws,
    maximumBets,
  )
  .signAndSend(alice, result => {
    if (result.status.isInBlock) {
      console.log('in a block');
    } else if (result.status.isFinalized) {
      console.log('finalized');
    }
  });

/// Get the lottery setup
const { result, output } = await contract.query.getLotterySetup(alice.address, { 
      gasLimit: gasLimit,
      storageDepositLimit: null,}
);
if (result.isOk) {
    console.log('Current value of "getLotterySetup":', output.toHuman());
} else {
    console.error('Error in getLotterySetup query:', result.asErr.toHuman());
}

process.exit(0);