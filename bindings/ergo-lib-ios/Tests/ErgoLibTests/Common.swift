// The following two functions copied from: https://stackoverflow.com/a/49890939

func toPairsOfChars(pairs: [String], string: String) -> [String] {
    if string.count == 0 {
        return pairs
    }
    var pairsMod = pairs
    pairsMod.append(String(string.prefix(2)))
    return toPairsOfChars(pairs: pairsMod, string: String(string.dropFirst(2)))
}

func base16StringToBytes(_ string: String) -> [UInt8]? {
    // omit error checking: remove '0x', make sure even, valid chars
    let pairs = toPairsOfChars(pairs: [], string: string)
    return pairs.map { UInt8($0, radix: 16)! }
}
