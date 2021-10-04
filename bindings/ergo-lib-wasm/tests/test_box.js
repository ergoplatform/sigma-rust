import { expect, assert } from 'chai';

import {
  ErgoBoxes, I64, ErgoBox, Tokens, BoxValue
} from '../pkg/ergo_lib_wasm';

it('ErgoBox.to_json_dapp() test', async () => {
    const box = ErgoBoxes.from_boxes_json([{
        "id": "3e762407d99b006d53b6583adcca08ef690b42fb0b2ed7abf63179eb6b9033b2",
        "txId": "93d344aa527e18e5a221db060ea1a868f46b61e4537e6e5f69ecc40334c15e38",
        "value": 2875858910,
        "index": 0,
        "creationHeight": 352126,
        "ergoTree": "101f0400040004020402040004000402050005000580dac4090580dac409050005c00c05c80104000e20b662db51cf2dc39f110a021c2a31c74f0a1a18ffffbf73e8a051a7b8c0f09ebc0580dac40904040404050005feffffffffffffffff01050005e807050005e807050005a0060101050005c00c05a006d81ed601b2db6501fe730000d602b2a5730100d603c17202d604db6308a7d605b27204730200d6068c720502d607db63087202d608b27207730300d6098c720802d60a9472067209d60bb27204730400d60c8c720b02d60db27207730500d60e8c720d02d60f94720c720ed610e4c6a70505d611e4c672020505d612e4c6a70405d613e4c672020405d614b2a5730600d615e4c672140405d61695720a73077215d61795720a72157308d61899c1a77309d619e4c672140505d61a997203730ad61be4c672010405d61ca172189c7212721bd61d9c7213721bd61e9593721d730b730c9d9c721a730d721dd1ededed938cb2db63087201730e0001730fedededed9272037310edec720a720fefed720a720fed939a720672109a72097211939a720c72129a720e7213eded939a721272167213939a721072177211939a72187219721aeded938c720d018c720b01938c7208018c720501938cb27207731100018cb272047312000193721995720f9ca1721b95937212731373149d721c72127216d801d61f997218721c9c9593721f7315731695937210731773189d721f7210721795720f95917216731992721e731a731b95917217731c90721e731d92721e731e",
        "address": "9aFbqNsmDwSxCdcLDKmSxVTL58ms2A39Rpn2zodVzkBN5MzB8zvW5PFX551W1A5vUdFJ3yxwvwgYTTS4JrPQcb5qxBbRDJkGNikuqHRXhnbniK4ajumEj7ot2o7DbcNFaM674fWufQzSGS1KtgMw95ZojyqhswUNbKpYDV1PhKw62bEMdJL9vAvzea4KwKXGUTdYYkcPdQKFWXfrdo2nTS3ucFNxqyTRB3VtZk7AWE3eeNHFcXZ1kLkfrX1ZBjpQ7qrBemHk4KZgS8fzmm6hPSZThiVVtBfQ2CZhJQdAZjRwGrw5TDcZ4BBDAZxg9h13vZ7tQSPsdAtjMFQT1DxbqAruKxX38ZwaQ3UfWmbBpbJEThAQaS4gsCBBSjswrv8BvupxaHZ4oQmA2LZiz4nYaPr8MJtR4fbM9LErwV4yDVMb873bRE5TBF59NipUyHAir7ysajPjbGc8aRLqsMVjntFSCFYx7822RBrj7RRX11CpiGK6vdfKHe3k14EH6YaNXvGSq8DrfNHEK4SgreknTqCgjL6i3EMZKPCW8Lao3Q5tbJFnFjEyntpUDf5zfGgFURxzobeEY4USqFaxyppHkgLjQuFQtDWbYVu3ztQL6hdWHjZXMK4VVvEDeLd1woebD1CyqS5kJHpGa78wQZ4iKygw4ijYrodZpqqEwTXdqwEB6xaLfkxZCBPrYPST3xz67GGTBUFy6zkXP5vwVVM5gWQJFdWCZniAAzBpzHeVq1yzaBp5GTJgr9bfrrAmuX8ra1m125yfeT9sTWroVu",
        "assets": [
            {
                "tokenId": "2d554219a80c011cc51509e34fa4950965bb8e01de4d012536e766c9ca08bc2c",
                "index": 0,
                "amount": 99999999998
            },
            {
                "tokenId": "bcd5db3a2872f279ef89edaa51a9344a6095ea1f03396874b695b5ba95ff602e",
                "index": 1,
                "amount": 99995619990
            },
            {
                "tokenId": "9f90c012e03bf99397e363fb1571b7999941e0862a217307e3467ee80cf53af7",
                "index": 2,
                "amount": 1
            }
        ],
        "additionalRegisters": {
            "R4": "0504",
            "R5": "05d4d59604"
        },
        "spentTransactionId": null,
        "mainChain": true
    }]).get(0);
    const j_obj = box.to_js_eip12();
    assert(j_obj instanceof Object);
    assert(j_obj.value == "2875858910", "should be a string of '2875858910'");
    assert(j_obj.assets[0].amount == "99999999998", "should be a string of '99999999998'");
});

