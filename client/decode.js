export function decode(data) {
    if (!data || data.length < 2) {
        throw new Error("Invalid event data format");
    }

    const [address, raw] = data;

    const bytes = raw.toU8a();
    const operatorHash = bytes.slice(0, 32);    // topic in payload
    const payload = bytes.slice(32);            // event payload

    const errorMap = [
        "AlreadyStarted",
        "StartingBlockPassed",
        "NoRecords",
        "BadOrigin",
        "TooManyDraws",
        "DrawNotFound",
        "DrawClosed",
        "DrawOpen",
        "DrawProcessed"
    ]; 

    const successMap = [
        "LotterySetup",
        "LotteryStarted",
        "BetAdded",
    ];     

    if (payload[1] === 0) {
        return successMap[payload[2]];
    } else if (payload[1] === 1) {
        return errorMap[payload[2]];
    } else {
        throw new Error("Invalid event payload");
    }    
}