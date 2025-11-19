import { ApiPromise, WsProvider } from "@polkadot/api";
import { ContractPromise } from "@polkadot/api-contract";
import { Keyring } from "@polkadot/keyring";
import fs from "fs";
import 'dotenv/config';

export async function getLottery(api) {
    const CONTRACT_ADDRESS = process.env.CONTRACT_ADDRESS;
    const CONTRACT_ABI_PATH = process.env.CONTRACT_ABI_PATH;
    const ALICE = process.env.ALICE;

    const abiJSON = JSON.parse(fs.readFileSync(CONTRACT_ABI_PATH, "utf8"));
    const contract = new ContractPromise(api, abiJSON, CONTRACT_ADDRESS);

    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri(ALICE);

    const gasLimit = api.registry.createType('WeightV2', {
            refTime: 300000000000,
            proofSize: 500000,
    });
    const storageDepositLimit = null;

    /// Get the lottery setup
    const { result, output } = await contract.query.getLotterySetup(alice.address, { 
        gasLimit: gasLimit,
        storageDepositLimit: storageDepositLimit,}
    );
    if (result.isOk) {
        return output.toHuman();
    } else {
        console.error(result.asErr.toHuman());
        return null;
    }
}