import Foundation
import ErgoLibC

class BatchMerkleProof {
    internal var pointer: BatchMerkleProofPtr
    internal init(withRawPointer ptr: BatchMerkleProofPtr) {
        self.pointer = ptr
    }
    init(withJson json: String) throws {
        var ptr: BatchMerkleProofPtr?
        let error = json.withCString { cs in
            ergo_lib_batch_merkle_proof_from_json(cs, &ptr)
        }
        try checkError(error)
        self.pointer = ptr!
    }
    func valid(expected_root: [UInt8]) -> Bool {
        return ergo_lib_batch_merkle_proof_valid(self.pointer, expected_root, UInt(expected_root.count))
    }

    deinit {
        ergo_lib_batch_merkle_proof_delete(self.pointer)
    }

}
