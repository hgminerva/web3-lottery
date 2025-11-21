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

import { openDraw } from "./open_draw.js";
import { processDraw } from "./process_draw.js";
import { closeDraw } from "./close_draw.js";

import { colors } from "./colors.js";

const WS_ENDPOINT = process.env.WS_ENDPOINT;
const CONTRACT_ADDRESS = process.env.CONTRACT_ADDRESS;

function truncateMiddle(str, head = 5, tail = 5) {
  if (typeof str !== 'string') return String(str ?? '');
  if (str.length <= head + tail + 3) return str; 
  return `${str.slice(0, head)}...${str.slice(-tail)}`;
}

async function main () {
    console.log("Connecting to blockchain...");
    const wsProvider = new WsProvider(WS_ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider });
    console.log("Connected to:", (await api.rpc.system.chain()).toHuman());

    let current_block = 0;
    let starting_block = 0;
    let next_starting_block = 0;    
    
    const unsubscribe = await api.rpc.chain.subscribeNewHeads((header) => {
        console.log(colors.yellow(`Block: #${header.number}`));

        current_block = header.number;

        // Get lottery and draw information
        getLottery(api).then((lottery) => {
            if (lottery != null) {
                console.log(`Lottery ${truncateMiddle(CONTRACT_ADDRESS)} (${lottery.Ok.isStarted}): [${lottery.Ok.startingBlock}, ${lottery.Ok.nextStartingBlock}] O:${truncateMiddle(lottery.Ok.operator)}, D:${truncateMiddle(lottery.Ok.dev)}`);

                starting_block = Number(lottery.Ok.startingBlock.replace(/,/g, ''));
                next_starting_block = Number(lottery.Ok.nextStartingBlock.replace(/,/g, ''));

                getDraws(api).then((draws) => {
                    console.log(colors.purple(
                        draws.Ok.map(d => {
                            const opening_blocks = Number(d.openingBlocks.replace(/,/g, '')) + starting_block;
                            const processing_blocks = Number(d.processingBlocks.replace(/,/g, '')) + starting_block;
                            const closing_blocks = Number(d.closingBlocks.replace(/,/g, '')) + starting_block;

                            // Open the draw
                            if (!d.isOpen && d.status == "Close" && current_block >= opening_blocks) {
                                // Prevention from re-opening of draw in the cycle.
                                // Draws can only open once per cycle.
                                if(current_block < closing_blocks) {
                                    openDraw(api, d.drawNumber).then((event) => {
                                        console.log(colors.green(`Start: ${event}`));
                                    });
                                }
                            }

                            // Process the draw
                            if (d.isOpen && d.status == "Open" && current_block >= processing_blocks) {
                                processDraw(api, d.drawNumber).then((event) => {
                                    console.log(colors.green(`Start: ${event}`));
                                });
                            }    

                            // Close the draw
                            if (!d.isOpen && d.status == "Processing" && current_block >= closing_blocks) {
                                closeDraw(api, d.drawNumber).then((event) => {
                                    console.log(colors.green(`Start: ${event}`));
                                });
                            }

                            let jackpot = Number(d.jackpot.replace(/,/g, '')) / 1000000; // Token decimal
                            
                            return `[Draw: #${d.drawNumber} (${d.status}, ${d.isOpen}, O:${opening_blocks}, P:${processing_blocks}, C:${closing_blocks}): ` +
                                `Pot:${jackpot}USDT Bets:${d.bets.length} Win#:${d.winningNumber} Winners:${d.winners.length}]`;
                        }).join(", ")
                    ));
                }); 
                
                // Start when not yet started and current block is greater or equal to the starting block
                if (!lottery.Ok.isStarted && current_block >= starting_block) {
                    startLottery(api).then((event) => {
                        console.log(colors.green(`Start: ${event}`));
                    });
                }  
                
                // Stop if all draws are close already.  If no draws, we will use the next starting block
                if (lottery.Ok.isStarted && current_block >= next_starting_block) {
                    stopLottery(api).then((event) => {
                        console.log(colors.red(`Stop: ${event}`));
                    });
                }

            }
        });

    });
}

main().catch(console.error);


