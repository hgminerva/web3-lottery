/// Lottery Job
/// 1. Setup the lottery
/// 2. Setup the lottery draw
/// 3. Start the lottery
/// 4. Start all draws
/// 5. Process draw one by one 
/// 6. Close draw one by one
/// 7. If all draws are closed, Close Lottery
/// 8. Back to Start the lottery (No. 3)
///
///
/// Lottery settings
/// a. Lottery Starting Block (LSB)
/// b. Total blocks per cycle, e.g., 24 hr period
/// c. Next Lottery Starting Block (Computed: a+b)
///
/// Draw settings
/// e. Total blocks until opening from (a)
/// f. Total blocks until processing from (a)
/// g. Total blocks until override from (a)
/// h. Total blocks until closing from (a)


// Import the API
import 'dotenv/config';
import { ApiPromise, WsProvider } from "@polkadot/api";

import { getLottery } from "./get_lottery.js";
import { getDraws } from "./get_draws.js";

const WS_ENDPOINT = process.env.WS_ENDPOINT;

async function main () {
    console.log("Connecting to blockchain...");
    const wsProvider = new WsProvider(WS_ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider });
    console.log("Connected to:", (await api.rpc.system.chain()).toHuman());

    const unsubscribe = await api.rpc.chain.subscribeNewHeads((header) => {
        console.log(`Block: #${header.number}`);
        getLottery(api).then((lottery) => {
            if (lottery != null) {
                console.log(`Lottery (${lottery.Ok.isStarted}): [${lottery.Ok.startingBlock},${lottery.Ok.nextStartingBlock}]`);
                getDraws(api).then((draws) => {
                    console.log(
                        draws.Ok.map(d => `Draw: #${d.drawNumber} (${d.status},${d.isOpen}): ${d.winningNumber}`).join(", ")
                    );
                });
            }
        });
    });
}

main().catch(console.error);