
import { BlockHeaders } from '../pkg/ergo_lib_wasm';

export const generate_block_headers = (transformFn, count=10) => {
  const headers = [];

  for (let i = 0; i < count; i++) {
    const header = {
      "extensionId": "d16f25b14457186df4c5f6355579cc769261ce1aebc8209949ca6feadbac5a3f",
      "difficulty": "626412390187008",
      "votes": "040000",
      "timestamp": 1618929697400,
      "size": "221",
      "stateRoot": "8ad868627ea4f7de6e2a2fe3f98fafe57f914e0f2ef3331c006def36c697f92713",
      "height": 471746,
      "nBits": 117586360,
      "version": 2,
      "id": "4caa17e62fe66ba7bd69597afdc996ae35b1ff12e0ba90c22ff288a4de10e91b",
      "adProofsRoot": "d882aaf42e0a95eb95fcce5c3705adf758e591532f733efe790ac3c404730c39",
      "transactionsRoot": "63eaa9aff76a1de3d71c81e4b2d92e8d97ae572a8e9ab9e66599ed0912dd2f8b",
      "extensionHash": "3f91f3c680beb26615fdec251aee3f81aaf5a02740806c167c0f3c929471df44",
      "powSolutions": {
        "pk": "02b3a06d6eaa8671431ba1db4dd427a77f75a5c2acbd71bfb725d38adc2b55f669",
        "w": "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
        "n": "5939ecfee6b0d7f4",
        "d": 0
      },
      "adProofsId": "86eaa41f328bee598e33e52c9e515952ad3b7874102f762847f17318a776a7ae",
      "transactionsId": "ac80245714f25aa2fafe5494ad02a26d46e7955b8f5709f3659f1b9440797b3e",
      "parentId": "6481752bace5fa5acba5d5ef7124d48826664742d46c974c98a2d60ace229a34"
    };

    if (transformFn) {
      transformFn(header);
    }

    headers.push(header);
  }

  return BlockHeaders.from_json(headers);
}