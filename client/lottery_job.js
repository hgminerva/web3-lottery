/// Lottery Job
/// 1. Setup the lottery
/// 2. Setup the lottery draw
/// 3. Start the lottery (this will immediately starts all the draws)
/// 4. Start all draws
/// 5. Processed draws one by one 
/// 6. Close draws one by one
/// 7. If all draws are closed, Close Lottery
/// 8. Back to start the lottery (No. 3)
///
/// Lottery settings
/// a. Lottery Starting Block (LSB)
/// b. Total blocks per cycle, e.g., 24 hr period
/// c. Next Lottery Starting Block (Computed: a+b)
///
/// Draw settings
/// e. Total blocks until opening from LSB(a)
/// f. Total blocks until processing from (a)
/// g. Total blocks until closing from (a)


// Import the API
import 'dotenv/config';
import { ApiPromise, WsProvider } from "@polkadot/api";

import { getLottery } from "./get_lottery.js";
import { getDraws } from "./get_draws.js";

import { startLottery } from "./start_lottery.js";

const WS_ENDPOINT = process.env.WS_ENDPOINT;

async function main () {
    console.log("Connecting to blockchain...");
    const wsProvider = new WsProvider(WS_ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider });
    console.log("Connected to:", (await api.rpc.system.chain()).toHuman());

    const unsubscribe = await api.rpc.chain.subscribeNewHeads((header) => {
        console.log(`Block: #${header.number}`);

        let lottery_started = false;

        // Get lottery and draw information
        getLottery(api).then((lottery) => {
            if (lottery != null) {
                console.log(`Lottery (${lottery.Ok.isStarted}): [${lottery.Ok.startingBlock},${lottery.Ok.nextStartingBlock}]`);

                // Starting and stopping lottery
                if (lottery.Ok.isStarted) {
                    lottery_started = true;
                } else {
                    lottery_started = false
                }
                
                getDraws(api).then((draws) => {
                    console.log(
                        draws.Ok.map(d => `Draw: #${d.drawNumber} (${d.status},${d.isOpen}): ${d.winningNumber}`).join(", ")
                    );
                });
            }
        });

        // Start or stop lottery information
        if (!lottery_started) {
            startLottery(api).then((event) => {
                console.log(event);
            });
        } else {
            stopLottery(api).then((event) => {
                console.log(event);
            });
        }

    });
}


main().catch(console.error);

/// Test scenario
/// 1. Lottery start/stop
///     1.1. No draws
///     1.2. Start block: 100
///     1.3. Total daily blocks: 20
/// 2. Test if all the controls are working
/// 3. Test of the next_starting_block is carried over

