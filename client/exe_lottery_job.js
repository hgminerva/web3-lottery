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
import { stopLottery } from "./stop_lottery.js";

const WS_ENDPOINT = process.env.WS_ENDPOINT;

async function main () {
    console.log("Connecting to blockchain...");
    const wsProvider = new WsProvider(WS_ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider });
    console.log("Connected to:", (await api.rpc.system.chain()).toHuman());

    let current_block = 0;
    let starting_block = 0;
    let next_starting_block = 0;    

    const colors = {
        red: (t) => `\x1b[31m${t}\x1b[0m`,
        green: (t) => `\x1b[32m${t}\x1b[0m`,
        yellow: (t) => `\x1b[33m${t}\x1b[0m`
    };    

    const unsubscribe = await api.rpc.chain.subscribeNewHeads((header) => {
        console.log(colors.yellow(`Block: #${header.number}`));

        let lottery_started = false;

        current_block = header.number;

        // Get lottery and draw information
        getLottery(api).then((lottery) => {
            if (lottery != null) {
                console.log(`Lottery (${lottery.Ok.isStarted}): [${lottery.Ok.startingBlock}, ${lottery.Ok.nextStartingBlock}]`);

                starting_block = lottery.Ok.startingBlock.replace(/,/g, '');
                next_starting_block = lottery.Ok.nextStartingBlock.replace(/,/g, '');

                // Starting and stopping lottery
                if (lottery.Ok.isStarted) {
                    lottery_started = true;
                } else {
                    lottery_started = false
                }
                
                // Start when not yet started and current block is greater or equal to the starting block
                if (!lottery_started && current_block >= starting_block) {
                    startLottery(api).then((event) => {
                        console.log(colors.green(`Start: ${event}`));
                    });
                }  
                
                // Stop if all draws are close already.  If no draws, we will use the next starting block
                if (lottery_started && current_block >= next_starting_block) {
                    stopLottery(api).then((event) => {
                        console.log(colors.red(`Stop: ${event}`));
                    });
                }

                // getDraws(api).then((draws) => {
                //     console.log(
                //         draws.Ok.map(d => `Draw: #${d.drawNumber} (${d.status},${d.isOpen}): ${d.winningNumber}`).join(", ")
                //     );
                // });
            }
        });



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

