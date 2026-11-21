use ergo_chain_types::{ExtensionCandidate, Header};
use ergo_nipopow::{NipopowAlgos, PoPowHeader};
use serde_json::Value as J;
use sigma_ser::ScorexSerializable;

use crate::hex_to_bytes;

fn bytes_to_hex(b: &[u8]) -> String {
    b.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn block_id_hex(id: &ergo_chain_types::BlockId) -> String {
    bytes_to_hex(id.0.as_ref())
}

fn parse_chain(chain_json: &[J]) -> Vec<PoPowHeader> {
    let headers: Vec<Header> = chain_json
        .iter()
        .map(|h| {
            let hex = h["headerHex"].as_str().expect("missing headerHex");
            let bytes = hex_to_bytes(hex).expect("bad headerHex");
            Header::scorex_parse_bytes(&bytes).expect("header parse failed")
        })
        .collect();

    let mut popow_headers = Vec::with_capacity(headers.len());

    let genesis_interlinks = vec![headers[0].id];
    let genesis_ext_fields = NipopowAlgos::pack_interlinks(genesis_interlinks.clone());
    let genesis_ext = ExtensionCandidate::new(genesis_ext_fields).expect("genesis ext");
    let genesis_proof =
        NipopowAlgos::proof_for_interlink_vector(&genesis_ext).expect("genesis interlink proof");
    popow_headers.push(PoPowHeader {
        header: headers[0].clone(),
        interlinks: genesis_interlinks,
        interlinks_proof: genesis_proof,
    });

    for i in 1..headers.len() {
        let prev = &popow_headers[i - 1];
        let interlinks =
            NipopowAlgos::update_interlinks(prev.header.clone(), prev.interlinks.clone())
                .expect("update_interlinks failed");
        let ext_fields = NipopowAlgos::pack_interlinks(interlinks.clone());
        let ext = ExtensionCandidate::new(ext_fields).expect("ext");
        let proof = NipopowAlgos::proof_for_interlink_vector(&ext).expect("interlink proof");
        popow_headers.push(PoPowHeader {
            header: headers[i].clone(),
            interlinks,
            interlinks_proof: proof,
        });
    }

    popow_headers
}

pub fn run_interlinks(chain_json: &[J]) -> J {
    let popow_headers = parse_chain(chain_json);
    let interlinks: Vec<J> = popow_headers
        .iter()
        .map(|ph| {
            let ids: Vec<J> = ph
                .interlinks
                .iter()
                .map(|id| J::String(block_id_hex(id)))
                .collect();
            J::Array(ids)
        })
        .collect();
    serde_json::json!({ "interlinks": interlinks, "error": J::Null })
}

pub fn run_prove(chain_json: &[J], m: u32, k: u32, header_id: Option<&str>) -> J {
    let popow_headers = parse_chain(chain_json);

    let chain = if let Some(hid) = header_id {
        let hid_lower = hid.to_lowercase();
        let idx = popow_headers
            .iter()
            .position(|ph| block_id_hex(&ph.header.id) == hid_lower)
            .expect("headerId not found in chain");
        popow_headers[..idx + k as usize + 1].to_vec()
    } else {
        popow_headers
    };

    let algos = NipopowAlgos::default();
    let proof = algos.prove(&chain, k, m).expect("prove failed");
    // `proofHex` is ergo's `NipopowProofSerializer.toBytes`, which ends with the proof's
    // `continuous` flag (`NipopowProof.scala:208`). sigma-rust writes the proof without it: the
    // flag belongs to the wire encoding, which the node adds. This tier proves with
    // `continuous = false` (runner-contract-nipopow §5), so the runner appends a 0.
    let mut proof_bytes = proof.scorex_serialize_bytes().expect("serialize proof");
    proof_bytes.push(0);
    serde_json::json!({ "proofHex": bytes_to_hex(&proof_bytes), "error": J::Null })
}
