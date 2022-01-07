import Foundation
import ErgoLibC

/// Represents which side the node is on in the merkle tree
enum NodeSide: UInt8 {
    case Left = 0
    case Right = 1
}
class MerkleProof {
    internal var pointer: MerkleProofPtr

    init(withJson json: String) throws {
        var ptr: MerkleProofPtr?
        let error = json.withCString { cs in
            ergo_merkle_proof_from_json(cs, &ptr)
        }
        try checkError(error)
        self.pointer = ptr!
    }
    init(leafData: [UInt8]) throws {
        var ptr: MerkleProofPtr?
        let error = ergo_merkle_proof_new(leafData, UInt(leafData.count), &ptr)
        try checkError(error)
        self.pointer = ptr!
    }

    /// Adds a new node and it's hash to the MerkleProof. Hash must be 32 bytes in size
    func addNode(hash: [UInt8], side: NodeSide) throws {
        let error = ergo_merkle_proof_add_node(self.pointer, hash, UInt(hash.count), side.rawValue)
        try checkError(error)
    }

    /// Validates the MerkleProof against the provided root hash
    func valid(expected_root: [UInt8]) -> Bool {
        return ergo_merkle_proof_valid(self.pointer, expected_root, UInt(expected_root.count))
    }
    /// Validates the MerkleProof against the provided base16 root hash
    func valid(expected_root: String) throws -> Bool {
        var result = false
        let error = expected_root.withCString { cs in
            ergo_merkle_proof_valid_base16(self.pointer, expected_root, &result)
        }
        try checkError(error)
        return result
    }
    deinit {
        ergo_merkle_proof_delete(self.pointer)
    }
}
