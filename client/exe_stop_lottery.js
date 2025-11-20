import 'dotenv/config';
import { ApiPromise, WsProvider } from "@polkadot/api";

import { stopLottery } from "./stop_lottery.js";

const WS_ENDPOINT = process.env.WS_ENDPOINT;

async function main () {
    console.log("Connecting to blockchain...");
    const wsProvider = new WsProvider(WS_ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider });
    console.log("Connected to:", (await api.rpc.system.chain()).toHuman());

    stopLottery(api).then((event) => {
        console.log(event);
    });

}

main().catch(console.error);