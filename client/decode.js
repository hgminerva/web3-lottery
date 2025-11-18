export function decode(data) {
    if (!data || data.length < 2) {
        throw new Error("Invalid event data format");
    }

    const [address, raw] = data;

    const bytes = raw.toU8a();
    const operatorHash = bytes.slice(0, 32);    // topic in payload
    const payload = bytes.slice(32);            // event payload

    const errorMap = [
        "Error::AlreadyStarted",
        "Error::StartingBlockPassed",
        "Error::NoRecords",
        "Error::BadOrigin",
        "Error::TooManyDraws",
        "Error::DrawNotFound",
        "Error::DrawClosed",
        "Error::DrawOpen",
        "Error::DrawProcessed"
    ]; 

    const successMap = [
        "Success::LotterySetup",
        "Success::LotteryStarted",
        "Success::LotteryStopped",
        "Success::DrawAdded",
        "Success::BetAdded",
    ];     

    if (payload[1] === 0) {
        return successMap[payload[2]];
    } else if (payload[1] === 1) {
        return errorMap[payload[2]];
    } else {
        throw new Error("Invalid event payload");
    }    
}