import { expect, assert } from 'chai';

import * as ergo from "..";
let sigmaRust;
beforeEach(async () => {
  sigmaRust = await ergo;
});

// const sigmaRust = await import("ergo-lib-wasm-browser");

const unspentBoxes = [
        {
          boxId:
            "490148afdc36f5459bbfd84922a446abea9a1077e031822f377b0ff3a6e467e3",
          transactionId:
            "c82615aa845d8159b7a9e33401c0d4c56535a8f40c3b40b4d86fcbc15084bf0f",
          blockId:
            "a0f0d9bc488e35a06b4a30cd3c6a88b98525b1f407b02142a7c9b91701875d75",
          value: 3621882,
          index: 3,
          globalIndex: 18315915,
          creationHeight: 778745,
          settlementHeight: 778748,
          ergoTree:
            "0008cd02f469fd785dbbc9112514655250c414d0c122d7df5eabc6966985b45b34542c02",
          address: "9gNhk4UDgxAmYmzuXf9Gpte3SJ4rfkm8Amwy3CfCEntTTXh6vbS",
          assets: [
            {
              tokenId:
                "31d6f93435540f52f067efe2c5888b8d4c4418a4fd28156dd834102c8336a804",
              index: 0,
              amount: 1,
              name: "ERGYOROI #066 Teru-Teru Bozu The Rain Stopper",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "953885cee2acbe335088fa7458b4bf2cb2126c6017bc81ca42c032715e2f1a0b",
              index: 1,
              amount: 1,
              name: "Ergo Mummy #25",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "9b28a759f46342497fb142ee557bc9a1707f4568707e25c2973359dc856c0762",
              index: 2,
              amount: 1,
              name: "ERGYOROI #079 Bake-Zori The Forgotten Straw Sandals",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "627bcf567bf08bd45e145c19d259a362dc4986fa656a6372ace739acb3528160",
              index: 3,
              amount: 1,
              name: "Crypto Stonks #687",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "381d373ab010dd69e0812f658f1742fc070889d91e8366635d1241416e627265",
              index: 4,
              amount: 1,
              name: "ERGYOROI #121 Daruma The Good Luck Talisman",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "59f2191c5d83a00f17b8bb82a2390afed43d86b2039fb55bff917584f0d5fa45",
              index: 5,
              amount: 1,
              name: "Ergoats #00580",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "102bf940efb3db3f993a8ec44f483b5efd114ca8d262558ab7cfbfd70fdf09e1",
              index: 6,
              amount: 1,
              name: "DinoEgg #111",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "b12d466ebbd914e9741d8d450f9fb65d8171e36f3d76cd4f6f467334544857a2",
              index: 7,
              amount: 1,
              name: "ERGYOROI #111 Kameosa The Endless Jar",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "04fd82ebc5e2b4f8621292ac8aa08b9d6060fcf82567035ce600488d831c2ef3",
              index: 8,
              amount: 1,
              name: "GoofBall #3",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "158d36f68957fb175c90b29394fb8f7172b9627ec0e9ffe08c563d0eca11ca51",
              index: 9,
              amount: 1,
              name: "Crypto Stonks #1131",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "e787996162900c65e06cd6e994e3e8d322b752d78203f255dfa5c0a4d43ab4e9",
              index: 10,
              amount: 1,
              name: "ERGOZX tERGminal Edition #1-5",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "f1a460b50b054b536ec83e773f3d43a3bbbb06b66629f120e6ab887483698a9b",
              index: 11,
              amount: 1,
              name: "ERGYOROI #085 Bake-Zori The Forgotten Straw Sandals",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "9f126722da9f04f10b97de8f766bfa8f154c5fc6d34b113c64260e65fe465f10",
              index: 12,
              amount: 1,
              name: "ERGYOROI #109 Pomellto The Fakes Patrol",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "4bdafc19f427fde7e335a38b1fac384143721249f037e0c2e2716631fdcc6741",
              index: 13,
              amount: 1,
              name: "DinoEgg #506",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "5614535ba46927145c3d30fed8f14b08bd48a143b24136809f9e47afc40643c4",
              index: 14,
              amount: 1,
              name: 'Ergnome #1109 "Glasgowius the Sigmanaut"',
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "44da627f0f5c99c841fe92348928bf99e2129a1d283dc4a39c9c90079686342a",
              index: 15,
              amount: 1,
              name: "DinoEgg #464",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "2c793e3ac69a7d152a745ed2ef02c311fdc7851b36d4e9fc1a74bbde83692806",
              index: 16,
              amount: 1,
              name: "Crypto Stonks #571",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "b917ca0ad02c8a1941f41cbdda226116f0df0ce11117bcce1bdfc084481f3ca1",
              index: 17,
              amount: 1,
              name: "Head of ERGXODIA",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "333b029ba8dd7e612b169cdef9a434260c88683422aaaa4de7eb6ff24dbfd3c6",
              index: 18,
              amount: 1,
              name: "DinoEgg #345",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "3b4d6b9b46b7d0b8e42bac43419936814121cfe9e7b8119f6b6d16f53e94e6be",
              index: 19,
              amount: 1,
              name: "Sigma Village #187",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "e91cbc48016eb390f8f872aa2962772863e2e840708517d1ab85e57451f91bed",
              index: 20,
              amount: 1200,
              name: "Ergold",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "20d7f899864eaeb201ecfe9bb60d95e7d10206aee03964fc99cbae30593d9a87",
              index: 21,
              amount: 1,
              name: "Circle",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "a1fc479e92c4a395bf48ed4d254050d14da3536129abb30cd7b8636d3288f93f",
              index: 22,
              amount: 1,
              name: "Ergoats #00767",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "dc2f241f3484f1545a8832df0ef63fda2546569d655353d10822b8dce62ecccd",
              index: 23,
              amount: 1,
              name: "Enigma 037 - Genesis Launch: 37 of 50",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "294aa338c03f64e7082c2b52c9045b6e987dabe8ff6e628f50c1a350cff4a4b7",
              index: 24,
              amount: 1,
              name: "Crypto Stonks #581",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "e033a68bdb699e386f2047f5884796565114d35bbe8893a9605efc27729b0f48",
              index: 25,
              amount: 1,
              name: "Ergosaurs #23: Helic O'Prion",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "55afa9997744bd3c56f5dfdcdb02fea608837eeca69917127a19f6dcfad07df3",
              index: 26,
              amount: 1,
              name: "DinoEgg #585",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "26eadec0db898195443534aa5c428d3b67bad9d5e522e69ee25421cc2a298b1b",
              index: 27,
              amount: 1,
              name: "Crypto Stonks #35",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "2416e34f5ccc2ac2caac6684b32213701b67c2d15e8d2a5113a6f490eba25be8",
              index: 28,
              amount: 1,
              name: "DinoEgg #239",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "db38c31459c34ebc3aabf0a0c6de3ac7ffc9b56c7355106573eeb3b713985331",
              index: 29,
              amount: 1,
              name: "Crypto Stonks #725",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "abba6ce28d029df63887a09567461b557c5787e301da72d8c765aff163755443",
              index: 30,
              amount: 1,
              name: "Ergoats #00795",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "68494a272c0e93cad91613be42c530792ccf535eac0b9b44a2db1252f9b09e6f",
              index: 31,
              amount: 1,
              name: "Crypto Stonks #630",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "7d1568dce627141e6c9945328ef2bae5b7cc5c4f67fd873e288b808e68447db3",
              index: 32,
              amount: 1,
              name: "Head of ERGXODIA",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "08883f07ac01eaa04e0099ce90eaacc6bafc69adb6744f54336775db49c3da88",
              index: 33,
              amount: 1,
              name: "Crypto Stonks #1162",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "0bb1d6e9a12da5febd268414c3d084f5adfbfabf83fd45f280fbf37bc7c78cbd",
              index: 34,
              amount: 1,
              name: "DinoEgg #071",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "186a1dc4b3015f7fb3802b86f260594399c50676f2b7d8d1576677f63cb8e7ca",
              index: 35,
              amount: 1,
              name: "DinoEgg #164",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "10af8682f5caaef7f6bc6b2708b8fe38e89ee02838f7ee8babbb36e2b8bc1bc8",
              index: 36,
              amount: 1,
              name: "Right Leg of ERGXODIA",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "1bf6f5d52fbd7ae1c6104c9e5c4510a3507f18318b3f9d925b225be434dc5f4f",
              index: 37,
              amount: 1,
              name: "DinoEgg #183",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "a59e565dbd6845177721477c802f24c079ed777f4d3a83faa1c9194e4920e96a",
              index: 38,
              amount: 1,
              name: "Crypto Stonks #1168",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "64ef95c5fb1ba4c9a4b2dd889d66bd5d434cd23c54e8cb1519becac6fd7995eb",
              index: 39,
              amount: 1,
              name: "ERGYOROI #034 Kasa-Obake The Abandoned Umbrella",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "4cbb7978fe6b9a5b9ca83adb7c095cf7d8269125727a8bec6e135990dc1ce159",
              index: 40,
              amount: 1,
              name: "DinoEgg #509",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "e8ea3b890cab60bf7e9f5b6bdf1c02c5ce6f048cb59748bd5db365d0bf3eb460",
              index: 41,
              amount: 1,
              name: "ERGOZX #71-100L",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "4aae9de83116e6bff1c96b098d3c0bcb87faba1b4fe1d314a157fffa7f3f65ea",
              index: 42,
              amount: 1,
              name: "Cybercitizen #702",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "73dd267e6306bd4a12fcba095ee93a51f1dbc0047665ed03b511c2cdb4676e69",
              index: 43,
              amount: 1,
              name: "Crypto Stonks #542",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "a024ea8dde6e97e29a57835ca98e4b83ccceb2421901d45dc2b61dce77fba156",
              index: 44,
              amount: 1,
              name: "Ergoats #00762",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "fd223ee9469451e36e617ca7a4df13292691a2c6cf45979b4edb5769c180b562",
              index: 45,
              amount: 1,
              name: "Plasma - Alpha: 3 of 25",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "fa7b453f9a9ea84cfe3e2df3d38a5028252cc55ebf84638f441034620186036f",
              index: 46,
              amount: 1,
              name: "Crypto Stonks #43",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "766a449dd0a6924cbd8ad60e9fa167a58a5223bd505e835085e4c4fef2483a8f",
              index: 47,
              amount: 1,
              name: "Aneta #0448",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "0bf15bf32faa940c738b38604a65df8e0adbbc371e27a28b09c7f9def8f94c48",
              index: 48,
              amount: 1,
              name: "ERGYOROI #053 Chochin-Obake The Spooky Lantern",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "f46b0b1ca603f9eddabf33ef5660bd4494d1cdfee1705648ffa3a052f479a62b",
              index: 49,
              amount: 1,
              name: "Plasma - Alpha: 10 of 25",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "4392f5a7cbf5c543253570436307e6d24019119fc360cfeb25b68f407515ec54",
              index: 50,
              amount: 1,
              name: "ERGYOROI #062 Teru-Teru Bozu The Rain Stopper",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "3385d569ba94ab965694601741d4833b5e16283b2b37254877a0d75f62fe87dc",
              index: 51,
              amount: 1,
              name: "DinoEgg #350",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "b97c5956c68e49993e6e21a69119b0c358134a3575dd1d0fc22f1bc1191c6407",
              index: 52,
              amount: 1,
              name: "AngryFood#3-TCR",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "0621276b36ff13654f7141494a2c477d804ab80fa90945cad28fe5c5d70e3d76",
              index: 53,
              amount: 1,
              name: "DinoEgg #040",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "35af3b9167768964b7460ac754ba18716172cb9810b94c27c8edcf56d383730d",
              index: 54,
              amount: 1,
              name: "DinoEgg #368",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "3e748d39bf1b52f49755d2cbdfc8e51238c5f7a31ef2ea1b053344d4c5abc8f0",
              index: 55,
              amount: 1,
              name: "DinoEgg #426",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "36aba4b4a97b65be491cf9f5ca57b5408b0da8d0194f30ec8330d1e8946161c1",
              index: 56,
              amount: 1,
              name: "Erdoge",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "a94ae3d5e0cd86341c41393383f24254a08dd43940a1ca8f590294405c1b5806",
              index: 57,
              amount: 1,
              name: "Ergoats #00789",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "1c2b179dc1c661e3ccf690fb2306f77cc61df41604dbfbb1488006a6ce5ac1cd",
              index: 58,
              amount: 1,
              name: "DinoEgg #185",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "0bf612f6265cf26cc187c0fcde87002555fed3701e65230a227d7e4f46dc3de2",
              index: 59,
              amount: 1,
              name: "DinoEgg #076",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "d11a7688ecbf35624b3a9aabec54404b9f17bf4b97319f9a091fa9f2db5d4537",
              index: 60,
              amount: 1,
              name: "Gnomekin #1399 ERGnomes Halloween '21 Special",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "8698f34733fe4e6524f23fbad61a4e4929f054b8a490779cb711f8d64947f8c5",
              index: 61,
              amount: 1,
              name: "Crypto Stonks #1226",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "347c058ba58aa3b65ed9165cdd32a06d03a52d88f50002bcd851950789ad63f3",
              index: 62,
              amount: 1,
              name: "DinoEgg #359",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "0c88274ad33c608688a544f3f083ed2029900436970daaa50315ee305887b4e6",
              index: 63,
              amount: 1,
              name: "DinoEgg #082",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "0779ec04f2fae64e87418a1ad917639d4668f78484f45df962b0dec14a2591d2",
              index: 64,
              amount: 2000,
              name: "Mi Goreng ",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "b7a670867ccbe41f9a17a795f7e4d6762fc34efb2a04c4506390e3a5ad16bbdc",
              index: 65,
              amount: 1,
              name: "Plasma - Alpha: 7 of 25",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "003bd19d0187117f130b62e1bcab0939929ff5c7709f843c5c4dd158949285d0",
              index: 66,
              amount: 20000,
              name: "SigRSV",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "253d5dc93224e160f1e206c23bb2bcb708b1391f467a05a386451234be12d195",
              index: 67,
              amount: 1,
              name: "DinoEgg #250",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "9e018e8234305d8b0942cf8bd534baeb4e3357ae0d70852fb4df3ada7d04a46c",
              index: 68,
              amount: 1,
              name: "AngryFood#3-E",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "2730e157a8c224c012ea093435d37cf406e38d280e80ea9c4dc11f6afe1e8027",
              index: 69,
              amount: 1,
              name: "Enigma - Genesis Launch Appreciation: 13 of 20",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "32285519fae2a007db88e2587fdc29323d396084edc1e0bb8304eb613d50cb3b",
              index: 70,
              amount: 1,
              name: "DinoEgg #339",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "2d18cca197c0529bc25e57d9977d394f5eb73c13ba258e6e327d0bd6d679e2cf",
              index: 71,
              amount: 1,
              name: "Ergosaurs #19: Hoskysaur",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "1a77f5e1f185318c16c13402b4df133d439ab0e85382eab0b5598c7a8aa6a6c9",
              index: 72,
              amount: 1,
              name: "Right Leg of ERGXODIA",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "4151ddd9faa9e010ac344e7a564b06fcc762e0b156135c2baa34276f1a14f78f",
              index: 73,
              amount: 1,
              name: "DinoEgg #451",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "bf2ca9932bbf41d6f543767f2516105f14208cfdec70fdc8f7484944c7088517",
              index: 74,
              amount: 1,
              name: "Ergoats #00850",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "4041ede42b6d20c3e4eb9ca5fc603fcaff887eac32614e8c2c4b7749bfaf5f7a",
              index: 75,
              amount: 1,
              name: "DinoEgg #441",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "632027f26786437d84e021fca54cf75cc074ea8b684894d9787e15ddececf0c7",
              index: 76,
              amount: 1,
              name: "Aneta #2393",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "01c1cacdcde5a3409542f9285348e6d205f918b643132102c8c656e98269b29e",
              index: 77,
              amount: 1,
              name: "DinoEgg #003",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "56a32f001d6d59fa610288a80a2b3a3997c94aeb3919cfe54bd9bdd400e766de",
              index: 78,
              amount: 1,
              name: "Crypto Stonks #541",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "2a9859eb568a40b098fa60d7ad5a3c816e1b272d58f630093b28304fdedad70b",
              index: 79,
              amount: 1,
              name: "DinoEgg #289",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "299588d22c61cc95a216114890ea952bab9d691c5dad4dfdc18d7d6996701c3d",
              index: 80,
              amount: 1,
              name: "Crypto Stonks #1393",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "2a0a42023d34d8af0a505c05731f586b1c27f6a03df0a048ccd3289ba294edfb",
              index: 81,
              amount: 1,
              name: "Crypto Stonks #592",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "3f08a8d967db04e17c9ba6e12ba750d1057de5d965196151234de22b2e3fe4df",
              index: 82,
              amount: 1,
              name: "DinoEgg #434",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "256057eed3a97bbdf8b4d2f6b4f81f28a1939b0dd54afbcc15dbcd0462d17048",
              index: 83,
              amount: 1,
              name: "DinoEgg #252",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "fbbaac7337d051c10fc3da0ccb864f4d32d40027551e1c3ea3ce361f39b91e40",
              index: 84,
              amount: 2000,
              name: "kushti",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "6a4d02be6c5e1ded3b63f085233678dcb24bcf062818a6b72353f487d1a869a4",
              index: 85,
              amount: 1,
              name: "Ergoats #00620",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "e8614cf303a6c2aedc7c00052301fd71b01c08f29cd0dd834fac6f1b62b1c183",
              index: 86,
              amount: 1,
              name: "Crypto Stonks #82",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "3b53beb91e4985efa3a6375d275a6857a700bd950ab53fcd726d1f16ae8c9817",
              index: 87,
              amount: 1,
              name: "Crypto Stonks #985",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "9512fb50d85510a0e2a2e419f13d93a88ce4d47ccf505b4e5fc4b169f696ba6b",
              index: 88,
              amount: 1,
              name: "Sky Sloth #PET_021_SKYHARBOR [Blue Sky Variation]",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "147c78505689450d7df8e78437e4b90a2ebdd3e34df4324344504fe3efffb2e2",
              index: 89,
              amount: 1,
              name: "Crypto Stonks #188",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "45a773bb751f4a1cb652624026a054b1fd7e526896d9b566938398d9bbdf2716",
              index: 90,
              amount: 1,
              name: "Left Arm of ERGXODIA",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "121c475e37122eab26f94576aaf3c27a760985a838305831d087edbd39c78835",
              index: 91,
              amount: 1,
              name: "DinoEgg #131",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "1dc443c7380173675a880a4864232c58d85ba3ada3d224f7e6e627584aeb1e68",
              index: 92,
              amount: 1,
              name: "DinoEgg #198",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "28fb011b57a6964b46f9186a1b4ac8a0589602cb17d677ebc25e7e5ee2404752",
              index: 93,
              amount: 1,
              name: "DinoEgg #277",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "12178bfd9ccacfa579380e32619642b0ee4c3d12c492dd130fd487e549e87806",
              index: 94,
              amount: 1,
              name: "DinoEgg #129",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "3bbda2ff2ab7a3a56b2b5c7f1f3901ee2bdb79ed77f55aae191a50a02292c734",
              index: 95,
              amount: 1,
              name: "Crypto Stonks #988",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "483765688858a2a3a761ae2d6ce7f523b7a203060ce17934cc5c206d99963add",
              index: 96,
              amount: 1,
              name: "DinoEgg #487",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "7901bd97ea250ecf81433e696b26bccf5efc3a8d454e0a251defed5e9e7ae1ab",
              index: 97,
              amount: 1,
              name: "ERGYOROI #110 Kameosa The Endless Jar",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "3b74c06148ded43c5a4043af18946b318cb22e3fa3e70b6388362e312e75a7d4",
              index: 98,
              amount: 1,
              name: "ERGYOROI #103 Akabeko The Red Cow",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "0ac45af034079b1480c9c4e61b0e970ee27cfc8537c97f4afbc94ee2d39abc00",
              index: 99,
              amount: 1,
              name: "DinoEgg #059",
              decimals: 0,
              type: "EIP-004",
            },
          ],
          additionalRegisters: {},
          spentTransactionId:
            "6f2e8307f74cec87566ae59e9b743c799e48c6645633f30d3c9df827bbb0c573",
          mainChain: true,
        },
        {
          boxId:
            "a9ca7b041922d1051c01196610b64eb61f17dbe1951a4e638620da7fba0a2855",
          transactionId:
            "c82615aa845d8159b7a9e33401c0d4c56535a8f40c3b40b4d86fcbc15084bf0f",
          blockId:
            "a0f0d9bc488e35a06b4a30cd3c6a88b98525b1f407b02142a7c9b91701875d75",
          value: 3621884,
          index: 2,
          globalIndex: 18315914,
          creationHeight: 778745,
          settlementHeight: 778748,
          ergoTree:
            "0008cd02f469fd785dbbc9112514655250c414d0c122d7df5eabc6966985b45b34542c02",
          address: "9gNhk4UDgxAmYmzuXf9Gpte3SJ4rfkm8Amwy3CfCEntTTXh6vbS",
          assets: [
            {
              tokenId:
                "8df02eb98db7057dfbf8292e770f06ebefd75c703bba87a5514fe0563755dd3e",
              index: 0,
              amount: 1,
              name: "Ergosaurs #25: L'Amarga",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "7345af1aa4a56ba9efd18a88505f1d6549e905b8830a86bd7e2fd97358d11ad1",
              index: 1,
              amount: 1,
              name: "Crypto Stonks #462",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "cda3f8538eea6a4791fb9e22025f08d735973dae207dffa6393fa2ef331b90e7",
              index: 2,
              amount: 1,
              name: "Gnomekin #1067 ERGnomes Halloween '21 Special",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "350f7d1369beb454d8abc1da3af9e69b5602f2ea38ccb6c96237702b55333901",
              index: 3,
              amount: 1,
              name: "ERGYOROI #131 Morinji-no-Okama The 100 Years Kettle",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "2f653a0a45e0d6726b2de80aee00473357556a020b8776640c5329bae9b0faad",
              index: 4,
              amount: 1,
              name: "DinoEgg #318",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "7bbeaf51cee6549641c47e4a673cb09cdaa140df27a43e1453de29433aada12e",
              index: 5,
              amount: 1,
              name: "Crypto Stonks #1138",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "10780a9bb17b386d27ac4c0eaa7e2e747bfe1e09b32266cc5d25f928733eedec",
              index: 6,
              amount: 1,
              name: "DinoEgg #114",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "13c506721c71062251e3ebcf49dca132936fb10c423d69be70aed0db56531f62",
              index: 7,
              amount: 1,
              name: "Hazey's Test NFT's #009",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "b37e9351cdd351bb84b68b70b3cc55bcc8552d5c6e3a167f1e74c1edd1ce17db",
              index: 8,
              amount: 1,
              name: "Ergoats #00813",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "739807fc054d0462734f23ced4e0144a6eac090a0ed1085b7d42fe2f24a214e0",
              index: 9,
              amount: 1,
              name: "ERGYOROI #115 Kameosa The Endless Jar",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "3af61d8fafee83a4c02ef34fd2d1f77152d488711e8877336e0af3dc045e372e",
              index: 10,
              amount: 1,
              name: "DinoEgg #403",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "28726c7b63de86dc157064e17f92147413327f88005be33f8766fc919898271d",
              index: 11,
              amount: 1,
              name: "Crypto Stonks #267",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "6ee317ee0a40697672e246a309aebba7571ec0020fe9a038d7cb2842c5a71a08",
              index: 12,
              amount: 1,
              name: "Crypto Stonks #916",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "938c06c20a60fcf9e43f51ea75a06fb8c9fa3190223022e424fc239c73d69c22",
              index: 13,
              amount: 1,
              name: "Left Arm of ERGXODIA",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "23c0ff31e8ffa8001535118857f9b389ec1b00e367a7b59e056ecda250acf588",
              index: 14,
              amount: 1,
              name: "Cybercitizen #282",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "0e242338a9ebfb95bddb1175d104c2ed3b08dbe0c84be1d94bfeaae9c48df43b",
              index: 15,
              amount: 1,
              name: "ERGYOROI #075 Heikegani The Samurai Crab",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "0de16f6f565c1d24b7bf6789bd4f9637e47ddaa448607e8c3872eb326b51a779",
              index: 16,
              amount: 1,
              name: "DinoEgg #092",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "135f383694f3c08698ebcd19aa2b65435609e177d6ce83d1b40cf6e34fefa5a3",
              index: 17,
              amount: 1,
              name: "DinoEgg #137",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "aeb79fd6d004b6c2a0f7c6219aebe8f943311e716506e14f2c084c8295f98713",
              index: 18,
              amount: 6,
              name: "GNOMECOIN",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "3e26b494c4a6776ed9f980c91889ccf6ce17c1fc164224e122745ac2ed3c3837",
              index: 19,
              amount: 1,
              name: "DinoEgg #424",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "be3d37e8416f45ea7a84023f8537917721844c031f8699ab08f9a04a840e7876",
              index: 20,
              amount: 1,
              name: "Ergoats #00848",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "0aaefe11f89e871c3ebe91432412debdd98e64c28978cf265fd903c6c1be8ea1",
              index: 21,
              amount: 1,
              name: "ERGYOROI #089 Hanzaki The Giant Salamander",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "d46bb5c2e9adba06651a21044af144f44bd6c8a907bb3c69fd2d32fa46a62e78",
              index: 22,
              amount: 1,
              name: 'Ergnome #1341 "Chris JR the Scholar"',
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "24e7a9c70b1551f34522dcf08d418dfea842ecb6ca2295f1aa91c64524b418bd",
              index: 23,
              amount: 1,
              name: "DinoEgg #245",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "4d0e45faed39280364bf10975e37481f299a073e02df314c90306724d7e8347f",
              index: 24,
              amount: 1,
              name: "DinoEgg #511",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "66c684bdff2d6323e3839f07cb496d79de01a32c802218ed1d452fa7efa0b406",
              index: 25,
              amount: 1,
              name: "Enigma 062 - Epiphany Series: 12 of 50",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "ed9e20fb1ea03664abc1170ff8ca6149b3c36712c75cdfcda255497c6d3b4c1f",
              index: 26,
              amount: 1,
              name: "Aneta #0921",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "90f8e027e3c734e86e20a8cab77156da1e85091817f65437e14bd5d630063498",
              index: 27,
              amount: 1,
              name: "Cybercitizen #1479",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "46445b4c268ba93f2722ede8f7b524d4823cec7e231856ca577a551266bf5827",
              index: 28,
              amount: 1,
              name: "DinoEgg #473",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "574844fe7412977f3491bf804c3f8dfb173a716fd32e1725fa3b2b85fce8ca89",
              index: 29,
              amount: 1,
              name: "Ergnome #920 El Eggplanto the SEC Agent",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "8ba10ab846b65f0da0bff5dac24e1e0045e53dde3d40b74d742c16735b14e44f",
              index: 30,
              amount: 1,
              name: "Sigma Village #175",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "44250fa304bdf573d28729376661fdbd096b1dc474ea19e832748cb35ab9eff9",
              index: 31,
              amount: 1,
              name: "DinoEgg #460",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "ba3997edd1e61bf27a9e68477d21f08fe56bbcf2162d8a6b76fe7793c4ff1e69",
              index: 32,
              amount: 1,
              name: "Gnomekin #0488 ERGnomes Halloween '21 Special",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "9f1b57cdcac19012985c1d631e7e7ce289eead8c909e6461601085b1a6758e98",
              index: 33,
              amount: 1,
              name: "$COMET Beggar",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "9ff11af1d3ef0f4d81fdd2808fd47536b0f853900e121e1e7bd5d1005cabbd42",
              index: 34,
              amount: 1,
              name: "ERGYOROI #021 Kappa The River Child",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "17646936955210cd70380566d66f76f76ebc0d95c0fa3097f7a4eadb68e211de",
              index: 35,
              amount: 1,
              name: "Ergoats #00419",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "f38ac4008c8245d7c954b2cba4824f3473658c63fb6b503643c69b5185a85d6c",
              index: 36,
              amount: 1,
              name: "ERGYOROI #112 Kameosa The Endless Jar",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "55950d991dcabdfa324ba74ef38e790438e58eaf2f334a3e0909c5fc8ce50fa5",
              index: 37,
              amount: 1,
              name: "Cybercitizen #830",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "eb4bcc52feadd0491cf1a375c305f8974bc85b16077f3703d7846895e2f979ae",
              index: 38,
              amount: 1,
              name: "Crypto Stonks #980",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "30f45f057333601b817b8c2e10cfb7e1c5c30bb57411cca3dc7fd2487d7eb1b9",
              index: 39,
              amount: 1,
              name: "Flame Pop #0122",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "afa2cab5868cccbdf19a1cebaf37b3a93d1e5d52ee8e7fbd39aab234c0767953",
              index: 40,
              amount: 2,
              name: "AngryFood#3-TCC",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "335ffded753fdd908de9fb2ef3d19e8bef1eefdf87fac38b09233fbb2c796613",
              index: 41,
              amount: 1,
              name: "Ergoats #00488",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "348c75941a2097de8d6046787a649b834d3285c427ce2a65693ed878ca626290",
              index: 42,
              amount: 1,
              name: "DinoEgg #360",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "9b3a00d92f94775b0f646d59aaf343a387b97d8bb1c06753328fd7dcdd2bace1",
              index: 43,
              amount: 1,
              name: "Crypto Stonks #1292",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "101e2cbdda2d34991753f4a91abf219b5188ea2a7e9cbf7ec2f75c631e5bcdad",
              index: 44,
              amount: 1,
              name: "DinoEgg #110",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "30974274078845f263b4f21787e33cc99e9ec19a17ad85a5bc6da2cca91c5a2e",
              index: 45,
              amount: 1200000000000,
              name: "WT_ADA",
              decimals: 8,
              type: "EIP-004",
            },
            {
              tokenId:
                "e04c61430999a270656f2e3008578565e8d8ec9966abfc1deedbdde085c5197e",
              index: 46,
              amount: 1,
              name: "Crypto Stonks #1282",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "3c27da96a91b95db63557818b136273a06dab97869af5b2fbf14e26c1617d461",
              index: 47,
              amount: 1,
              name: "Aneta #2239",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "50d2bbe769bb0213f55cae8099c88ffb63d10e005cdf3c8293cb4b23fd4c4773",
              index: 48,
              amount: 1,
              name: "DinoEgg #545",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "002080fce20f4dca2099eb2ddf8d8442de00b5555075a73e96cffbb876546bf1",
              index: 49,
              amount: 1,
              name: "Ergoats #00346",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "84375901c645fd9bfd6f8bd4abab6f48e2987cb5049bffa3b033a4f245823da3",
              index: 50,
              amount: 1,
              name: "Enigma 099 - Epiphany Series: 49 of 50",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "7520f75a1beb760d108405af1aa38930cd000201835e23380cccffda362c8034",
              index: 51,
              amount: 1,
              name: "Crypto Stonks #485",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "db6954a47ac7f187782f8bf4da49a77f5d29e786c4915d09a5fdd3f5b4ea182b",
              index: 52,
              amount: 1,
              name: "Hazey's Test NFT's #064",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "acd1df21d20649aaff38b825de18678e2b14f2f3af226292b3688c292153d5a5",
              index: 53,
              amount: 1,
              name: "ERGYOROI #116 Kameosa The Endless Jar",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "ed03db810bd5dc02e3e86410d080df1ce01466afef36d045fb9e78dc8d6c1d05",
              index: 54,
              amount: 1,
              name: "Crypto Stonks #180",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "954a33680540286cf2639b0daae489b2204408fea1622ddcd568f7c625aa9a90",
              index: 55,
              amount: 1,
              name: 'Glezcón "The Chicken Series" #10',
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "c418c84106a337e9a0b544687e6d0267b0a9fd33ba5e36e5540504d994568f79",
              index: 56,
              amount: 1,
              name: "ERGYOROI #039 Ogama The Rainbow-Breath Toad",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "ef802b475c06189fdbf844153cdc1d449a5ba87cce13d11bb47b5a539f27f12b",
              index: 57,
              amount: 10500000000000,
              name: "WT_ERG",
              decimals: 9,
              type: "EIP-004",
            },
            {
              tokenId:
                "11831c51c7b051fc5ef18709594908246c99a827b62c917950ca42bf89b60cf6",
              index: 58,
              amount: 1,
              name: "DinoEgg #119",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "347ab0218c28abd65774df9e1a6f32daf16c601638cbd73e68c4fad057089b88",
              index: 59,
              amount: 1,
              name: "DinoEgg #358",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "4176a695b78fdb825bec6051aff75ce5b1cb1ce275e8f27d3be7e01d36755e83",
              index: 60,
              amount: 1,
              name: "Space Farmers - Nautilus Silver Moon 7/12",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "6b4bb2d883873635981d03bad6c26fe542e0238dbcbf96a72ee90f58664f6b83",
              index: 61,
              amount: 1,
              name: "Right Arm of ERGXODIA",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "ad59ffa4c30b1d04d1fcc7be1fc62c9a49196e3a0009aa75252e0bd2db73bd79",
              index: 62,
              amount: 1,
              name: "ERGYOROI #107 Nue The Black Cloud",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "22c357d4b0e18300ef32349f5958e81b651b56d541295041fa4ac058f4bb0acb",
              index: 63,
              amount: 1,
              name: "DinoEgg #230",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "4fce6baa47e099ff518ae5ce6e94bc836fb66d90b3bcab66c535f567877c7379",
              index: 64,
              amount: 1,
              name: "DinoEgg #539",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "4ea732b8e57555a2c64be2d86597bf6cb0f5771b132dca514e912571249eb22a",
              index: 65,
              amount: 1,
              name: "Gnomekin #0342 ERGnomes Halloween '21 Special",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "ea77f6e7adbd1de7774ac2cb45e94eccdc2c990bb40dc95fef4cfc9fd495b293",
              index: 66,
              amount: 1,
              name: "[Cyberia] Apartment #6",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "1cd3242b9cbf75a0510963c43abb60c7ca104b8f6853df78a3f5c3109e4f31c2",
              index: 67,
              amount: 1,
              name: "Crypto Stonks #640",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "7349463f792f4dd7e6c6099b6521875aedfc2a6c2ae9c6223b536231dab20683",
              index: 68,
              amount: 1,
              name: "Cybercitizen #1160",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "633dcdff23cce3cb16169bd618dc1e2c88c6d157d142a4f932d7fa8b6fb464f3",
              index: 69,
              amount: 1,
              name: "Cybercitizen #985",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "19f377095bff2228d72fe5cfc72b65f34819eeb9a64e27b16638a91245cc7f05",
              index: 70,
              amount: 1,
              name: "Crypto Stonks #486",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "c3a062c568870dcca174f8ff28a432cf53e179dce514b5923e5a6f3e45e9cc73",
              index: 71,
              amount: 1,
              name: "ERGYOROI #058 Basan The Ghost-Fire Rooster",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "4df13aee958966f031be25139a1d64bda91d3ebb1161f9886ff696707a60fdb8",
              index: 72,
              amount: 1,
              name: "DinoEgg #524",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "3fddc0d0aee92aa7f42b91fcdc438ca47c2c6e6fee0f0600fce156cb024162ba",
              index: 73,
              amount: 1,
              name: "Ergoats #00513",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "00b1e236b60b95c2c6f8007a9d89bc460fc9e78f98b09faec9449007b40bccf3",
              index: 74,
              amount: 10000000,
              name: "EGIO",
              decimals: 4,
              type: "EIP-004",
            },
            {
              tokenId:
                "020db2544899d73833b6c3b964d75f35c859caaf6c7b402a52ef6de94fbe4a3f",
              index: 75,
              amount: 1,
              name: "DinoEgg #009",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "6cb2bd1360083a05a51291ecd3e0062a3f4d94594abac765f1b4ed5617ff4dfc",
              index: 76,
              amount: 1,
              name: "Crypto Stonks #298",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "6b365cc6df06680a65eb705718fa238238663919a83c6ad1281606e2f0126e0d",
              index: 77,
              amount: 1,
              name: "Crypto Stonks #1347",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "51d32a89056c4119459a28af113b8cee7fb777c70fd6fee1272197ae3caf9b5f",
              index: 78,
              amount: 1,
              name: "DinoEgg #553",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "05d51c7813b97c00448b7dd7463a1d0b47d82d6b4325ea5a80dea9127bdd8838",
              index: 79,
              amount: 1,
              name: "ERGYOROI #117 Kameosa The Endless Jar",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "84226cacf25b9697500bba8ef8d2ba897eb4af3c0a10efda18be740b0f5451e4",
              index: 80,
              amount: 1,
              name: "Crypto Stonks #961",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "ac7efbeb9abe2509e27bb5a6cea84a1dce1b1a54110e7b34daf4cb19fac58a26",
              index: 81,
              amount: 1,
              name: "Crypto Stonks #357",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "b49d2f4b340fa1ca96d83638b52c270d1ad47d784abba1ffb4d026f046c89786",
              index: 82,
              amount: 1,
              name: "Aneta #0685",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "e227742f4d1e10cbaac63720c7e476e88bfbe6a0377efa9cde643f963a43efd6",
              index: 83,
              amount: 1,
              name: "Ergoats #00933",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "a2bdcb85c9c9f078c67f91915d5fc57d7035506b47b37408153fbfc822624529",
              index: 84,
              amount: 1,
              name: "ERGYOROI #120 Daruma The Good Luck Talisman",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "40538d18e0c92fe87abfb4781113003fa3332254518184746901806e9a421588",
              index: 85,
              amount: 1,
              name: "Ergoats #00514",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "5bd9460e785704754423a8b25127ab403ff4db32d5724cfbdeddd76b7ec9f7e9",
              index: 86,
              amount: 1,
              name: "CyberPetz 12 - Bob the Goanna - 0 POINTS",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "5360d08d4850ae27a4f086955c361204f0ee3b17e64bd9940d516c8969660823",
              index: 87,
              amount: 1,
              name: "DinoEgg #561",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "901f29c703192ad696598c5a4dd6152002fd7cb7f2a763c1cc2ad6e3464c5aa8",
              index: 88,
              amount: 1,
              name: "ERGYOROI #098 Ittan-Momen The Possessed Cotton",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "e21d27d9bbfec0a385e33c8987f62d14955ade38b3030d97681868b7492c76ad",
              index: 89,
              amount: 1,
              name: "Right Arm of ERGXODIA",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "8d8bee48399a240013c07e1eaea08df615e684280712286b8207a6637e7fc812",
              index: 90,
              amount: 1,
              name: "ERGYOROI #050 Chochin-Obake The Spooky Lantern",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "200e96f82294a468033037124fa2f149538b999de9742fdfb261ed9cac4d19f6",
              index: 91,
              amount: 1,
              name: "DinoEgg #210",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "5121ae6269e944edb1ad690197aab4cc5c8455046e3fceee0c3b6cb34f911c35",
              index: 92,
              amount: 1,
              name: "ERGYOROI #051 Chochin-Obake The Spooky Lantern",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "465732d1aab4a94f4719fb18850dada577d7afab0d8227804138ddac56b2b2fa",
              index: 93,
              amount: 1,
              name: "DinoEgg #475",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "b82b97ede17ecec012611b9bbf6f7ace58ad1cae2a51b157ba34544fddc87664",
              index: 94,
              amount: 1,
              name: "Cybercitizen #1906",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "1afcf9d36c411cc994490f938260c36fdf1e9bb268a8bf4b1041d3d026bf3ed6",
              index: 95,
              amount: 1,
              name: "Ergoats #00426",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "15c9518c7365a5bd57da9518930ccdc1f0e9bd78d47fffb549267d64c1ccf104",
              index: 96,
              amount: 1,
              name: "DinoEgg #150",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "227038d64724920f3a7d99f79812eec9085c29cd32097cea0c9e3edd2ccae4d0",
              index: 97,
              amount: 1,
              name: "DinoEgg #227",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "b40b9dded5fb5f6406d10643955db39d94ad9e3a6a4b6c899c441b13cfc509a9",
              index: 98,
              amount: 1,
              name: "BFT Rally 0270",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "37634891fb5f62074bc720e8c5d0eaae5141f24fc432b399c82b9feea886c4ef",
              index: 99,
              amount: 1,
              name: "DinoEgg #382",
              decimals: 0,
              type: "EIP-004",
            },
          ],
          additionalRegisters: {},
          spentTransactionId:
            "6f2e8307f74cec87566ae59e9b743c799e48c6645633f30d3c9df827bbb0c573",
          mainChain: true,
        },
        {
          boxId:
            "f0b6ff61c41692ddbb4a57acd8ff5ca7f113bbd89278c28535f8347b96279967",
          transactionId:
            "7f7d8b36ab959f87471f99d0f8fb5f96492451437f07f698e414c33a5303f77d",
          blockId:
            "bff28f3837fa4d6a2f4787747216f4621a37d427ebe0b4e57a5e07e45be7dcd6",
          value: 2621882,
          index: 0,
          globalIndex: 18760324,
          creationHeight: 787316,
          settlementHeight: 787318,
          ergoTree:
            "0008cd02f469fd785dbbc9112514655250c414d0c122d7df5eabc6966985b45b34542c02",
          address: "9gNhk4UDgxAmYmzuXf9Gpte3SJ4rfkm8Amwy3CfCEntTTXh6vbS",
          assets: [
            {
              tokenId:
                "fb7f62449b8faf496c4b0e6e0f80a3d8970527bbbf17ce5809ab8527d1add85d",
              index: 0,
              amount: 1,
              name: "Screaming ERGoat Gen 1 Membership Card",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "8565b6d9b72d0cb8ca052f7e5b8cdf32905333b9e026162e3a6d585ae78e697b",
              index: 1,
              amount: 1,
              name: "Ergoats #00692",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "4d522a6143f33ec8f6dfb2c9b32b742ee40cd948bc8df10aca5733d11cb4c8b1",
              index: 2,
              amount: 1,
              name: "DinoEgg #516",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "b028b5d544e0fe2430c0b1d44a7612486058a0d11f1ba3ce9f74177895fa1fb9",
              index: 3,
              amount: 1,
              name: "Ergoats #00804",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "0cd3bdb56aa3bf7cf5cbd7976fa29f21aef2e34690dbcfdfb8a823f6d35de910",
              index: 4,
              amount: 1,
              name: "DinoEgg #084",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "0db36a7d851d42931ba5a8d537d77f47b7b0f70bc55152380d671f94d66ae2c0",
              index: 5,
              amount: 1,
              name: "Ergoats #00380",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "0fd98c8d1d2d0c1a5de36c172d314e86344a1112ebff7fcba7d06b2e11522379",
              index: 6,
              amount: 1,
              name: "ERGYOROI #125 Morinji-no-Okama The 100 Years Kettle",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "0f2a60a971b5b01eb2661692f5c1303aa006fd8258974a776f5534f45560e157",
              index: 7,
              amount: 1,
              name: "Gnomekin #1075 ERGnomes Halloween '21 Special",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "6c927c01cf01d9ed354888928a4b22f5b1e5ebbc3e798a8a17b3e4c2d420f9c0",
              index: 8,
              amount: 1,
              name: "Cybercitizen #1085",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "4f44713ed8bdc566949f5722c0158e0490c0919452bb440e2415c38810f65622",
              index: 9,
              amount: 1,
              name: "DinoEgg #535",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "0f037f9e869774b63ee1e8f0b90d83f0ed6d92e25c298854e4acd74db06cd250",
              index: 10,
              amount: 1,
              name: "COMET WL",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "32abb2e866c1b7558baa47e4a840f02b6f684e9518cb21de7c7e52b56f0c2487",
              index: 11,
              amount: 1,
              name: "DinoEgg #341",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "d29f0f2e5b3aeca0fd5a1ef2d6e8ca8d13eba887818b9c0ecab853ca6bc6d2c1",
              index: 12,
              amount: 1,
              name: "ERGYOROI #060 Amemasu The Mutant Whale",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "9e00b15e6ee77063bb61c9006f45345da045e9e9da0dcd8c574b54ce440610a7",
              index: 13,
              amount: 1,
              name: "BFT Rally 0268",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "e3b7c6929882d779f8ad6cd6ebb1078243e3819e11176bf2a47820824b092d1b",
              index: 14,
              amount: 1,
              name: "BFT Rally 0301",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "e0722083d0c41b2c5ad250c158da5106d753ef6cf4f5c05724ccfe4eba7af86a",
              index: 15,
              amount: 1,
              name: "Crypto Stonks #1399",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "3df91f666f6d10faa83a56b56f469597f6157c7eba1254cc025a63aa7583c859",
              index: 16,
              amount: 1,
              name: "DinoEgg #423",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "04c9142fcb4cfce3b5dd1051192e5e1a7298266049c1309510913d20711c9c89",
              index: 17,
              amount: 1,
              name: "ERGYOROI #055 Basan The Ghost-Fire Rooster",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "bfb88e98798e41c8e18f388620a6b626642cddd842c6893ceb906d0afb1439d9",
              index: 18,
              amount: 1,
              name: "AngryFood#6-TCC",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "538178c5bc25b2f1745d2fd2c9be3e71d2da873e2f7b78c2e6cd82bdbc39173a",
              index: 19,
              amount: 1,
              name: "DinoEgg #564",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "2ccbff5ce69d5c6666ae29742ba61ca0409b07dffac6f6ba4c0f8453fc93951e",
              index: 20,
              amount: 1,
              name: "DinoEgg #301",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "e5610cdfd8f9150e6e25674ed6ed6ae6bd84776f8d850c7a48c275b4a04421ed",
              index: 21,
              amount: 1,
              name: "Aneta #0892",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "ac28335528dd71bd095e6d5b7a140497fc30b70b4a2a1cc10f8bf7a2c076cf46",
              index: 22,
              amount: 1,
              name: "Aneta #3671",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "0640b514697c9059b6007c80ef42e70ae505facf9843cdb90a9d525594d14e89",
              index: 23,
              amount: 1,
              name: "ERGYOROI #020 Kappa The River Child",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "71848074c2e1e68fb043879acf89df5158feb46a1e1968ed165c391ffe4971f5",
              index: 24,
              amount: 1,
              name: "Ergoats #00640",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "469a6fffec763ed7365eeef1090dcec103377845da563f2de1dca8b7490b9cc4",
              index: 25,
              amount: 1,
              name: "DinoEgg #480",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "3367b3dffe6a9e324d3fbc1d16949d091843830f660de861cb8160d8468c354f",
              index: 26,
              amount: 1,
              name: "DinoEgg #348",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "4fcaa6dac8d7ea51af861a1f68c528dcdbcfc79426afe1f912ed71d571c7fe64",
              index: 27,
              amount: 1,
              name: "Crypto Stonks #958",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "8c25c5fb4eeb04250997f9bcc06870257722bd16a957ea3f1a017be96b8ec222",
              index: 28,
              amount: 1,
              name: "Aneta #3569",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "40563c86b3024905f03e97b31cd99eaadaf1f2f0bfe5911e3718b569f9d64dc1",
              index: 29,
              amount: 1,
              name: "DinoEgg #442",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "d70208c6cbd956492ba2b7899c46d0cce7861d7e834134e5dc446687f840fb26",
              index: 30,
              amount: 1,
              name: "Crypto Stonks #1024",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "94f39559cc1da7d23ccd1fe71a855cb1713651c73bb967d46351daccbd9eb03d",
              index: 31,
              amount: 1,
              name: "Plasma - Alpha: 4 of 25",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "47098f63057b29c2973beadd10b7a3d3abc99dd42d32750795960c002356a969",
              index: 32,
              amount: 1,
              name: "AngryFood#3",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "472c3d4ecaa08fb7392ff041ee2e6af75f4a558810a74b28600549d5392810e8",
              index: 33,
              amount: 2604000000,
              name: "NETA",
              decimals: 6,
              type: "EIP-004",
            },
            {
              tokenId:
                "65e1c8159b9a9af4c44e449236f3c3313134165fdf7302b5b89a9e1075dbff59",
              index: 34,
              amount: 1,
              name: "Crypto Stonks #191",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "f49b2708f3c6205b57cb4f41427bde51aee7ab04c39adb4896088afbf3c246c1",
              index: 35,
              amount: 1,
              name: "CyberPetz - Upgrade Wings",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "207b7356a003da53de1328a5124add84a76041c8666e24a28335398a72d67a93",
              index: 36,
              amount: 1,
              name: "ERGYOROI #008 Hyottoko The Classic Clown",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "7649e944ad25a0a7235795a3a62a5719fa8e4e919d4c07031af22e0d6460c265",
              index: 37,
              amount: 1,
              name: "Ergoats #00653",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "7b2ce79ab6f8af2af5177a34c617e05139c4e4273d23aeef19eb1d9a564d7b0c",
              index: 38,
              amount: 1,
              name: "ERGYOROI #091 Shachihoko The Tiger Koi",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "81c320ac5d619206120e05e96322c8a9e1097cf01353f97a4377be5fa3f260ec",
              index: 39,
              amount: 1,
              name: "Monster #3",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "28887213f626c369759112b178a521044190d46a31b58d8974e7587b23a975aa",
              index: 40,
              amount: 1,
              name: "Hazey's Test NFT's #015",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "2d5c546e94fc159012c3efacf70722e9bb8cd26c4f4e5c0857b0ccee8b38f722",
              index: 41,
              amount: 1,
              name: "DinoEgg #304",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "4fb4e3a49effb8e00d6d677bd892e1da69794aadf92ab521597e164b2999c5b7",
              index: 42,
              amount: 1,
              name: "Ergoats #00559",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "e019db06310fc0c239b3872df254e1c30731711bef18d9414d5b0a17db776410",
              index: 43,
              amount: 1,
              name: 'Ergnome #1357 "Christopher the Ergolend Nurturer"',
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "30d97e1de35a5642f73d56a8a44c75f6de03a88f4f7a2af4ec77b8055f09dc54",
              index: 44,
              amount: 1,
              name: "DinoEgg #332",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "25907b0a8bdc6bc6d6ea7bc89e6fb2babb9be56b952987ac54a3093d9351e332",
              index: 45,
              amount: 1,
              name: "DinoEgg #254",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "2c06586a6e5368eefefb583b576220d9a03ace49fa2191ced397ad4aa957bfb8",
              index: 46,
              amount: 1,
              name: "DinoEgg #296",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "2b2fa1cbe8603b12c7a314a9ddb2799656277a0b29297b7513008d6cad4f76a3",
              index: 47,
              amount: 1,
              name: "Ergnome #758 Lichnyy the BPSAA Privacyshroom",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "281f5a42a59080fda5cbc1d682fdfe55387d126166db4ed13e342b39ed4df7cb",
              index: 48,
              amount: 1,
              name: "AngryFood#4-TCC",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "e9903f81c8c53977e2909a2d4efa6cf61a5377a3de90e255e318dbf23a9a3372",
              index: 49,
              amount: 1,
              name: "Plasma - Alpha: 6 of 25",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "25e0d943ec1b44c85534e1d5bd09072b37f07250e27fba46e4dfa27e83f1c814",
              index: 50,
              amount: 1,
              name: "Privacy Hackagnome #GN_034_ERGOHACKIII",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "4dfe077604cb6be1b553e9e771d0e1c4b8549d7256535465c471ca0b3b00e933",
              index: 51,
              amount: 1,
              name: "Aneta #4424",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "7d441ce250ffd0d70511899e5e9090f20a0443260983f76ef89a32cbb3eabda4",
              index: 52,
              amount: 1,
              name: "Left Leg of ERGXODIA",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "d45235f2ec8a30cde863ad6452a2de40b0aaa540505f541995eca954ad6753e1",
              index: 53,
              amount: 1,
              name: "Left Leg of ERGXODIA",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "08871d0781d25b4d073419e5c466a738b13d93ec11def2d80d5a9dc6eb9d2d79",
              index: 54,
              amount: 1,
              name: "Crypto Stonks #634",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "d953dfd88433d4287c988e98d62ec5a677955e9baf971b7fc14e12a5eed9e2be",
              index: 55,
              amount: 1,
              name: "Plasma - Alpha: 9 of 25",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "455d56abaf87183f4b51bb4eddea2921668d251dc958422b5fd523219761aae8",
              index: 56,
              amount: 1,
              name: "DinoEgg #467",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "56c19596edba6c408d1fe95d43d1b3ef190d1f0cb1db357c5840bf0f09f4429f",
              index: 57,
              amount: 1,
              name: "DinoEgg #592",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "20cc2ddf07874ac4d810c66b98a29a3d34f403d23734398f217235f6f30c4229",
              index: 58,
              amount: 1,
              name: "DinoEgg #215",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "80f0c11567692a5045c5d994c3a5f635d39bb1ab1555d7fb90ae13d7a151537c",
              index: 59,
              amount: 1,
              name: "Crypto Stonks #874",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "0cd8c9f416e5b1ca9f986a7f10a84191dfb85941619e49e53c0dc30ebf83324b",
              index: 60,
              amount: 312501,
              name: "COMET",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "78d81eb8429817827fccbb22d0629972746a42b8c7cdc2326c7d57f914e8fe1b",
              index: 61,
              amount: 1,
              name: "ERGYOROI #096 Ittan-Momen The Possessed Cotton",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "41cff4d21fc73df86d29f9b3b4a50abfa2cba9d3aa08b903d3ec5916ff7a5468",
              index: 62,
              amount: 1,
              name: "DinoEgg #452",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "47c626aec3f69b835609902616322e2d9ff0c906dafb5bac9f36e9e1bfebb08b",
              index: 63,
              amount: 1,
              name: "DinoEgg #485",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "16e3f483f562dc48a249cc8582dc1c9c0ab8228feea5eb1f9108a95f7fff9354",
              index: 64,
              amount: 1,
              name: "DinoEgg #154",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "532f998902378a0ae6c11c176de7712a49dac04252e4e2aa5515cb359554885a",
              index: 65,
              amount: 1,
              name: "DinoEgg #560",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "067582c88162ea3e34f539464fb62d0dbc822ddfacd3d6faa70ec48ef8c8be2e",
              index: 66,
              amount: 1,
              name: "Ergoats #00364",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "fdad8af90587bda873785624ad7b8109481806d7f0d3b5df5979ad53aa54d096",
              index: 67,
              amount: 1,
              name: "ERGYOROI #086 Hanzaki The Giant Salamander",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "9d7be44cc3772375ba662b20eb206cb24e5355acf2e19e4edd50e6e75cd29e19",
              index: 68,
              amount: 1,
              name: 'Ergnome #1240 "DeCus the Turniprof"',
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "3f65cfb7cee3ad874c254a0fd796404b3845be617f76621ffbe1d4d7eaed82f6",
              index: 69,
              amount: 1,
              name: "DinoEgg #438",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "e26174711b76537faa3d4a6008ccaf1ae1d63f53c08859bea9e2394a3d56a29f",
              index: 70,
              amount: 1,
              name: "ERGYOROI #141 Nurikabe The Mr. Wall",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "3c32847471975542f68d43cd426b9f5f0af671ed7c1ac4e7b4e8218acb45dca4",
              index: 71,
              amount: 1,
              name: "DinoEgg #411",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "296fff96f55fdd894914ef58664e41083a8c0951c2ab00c443324bd087cbb420",
              index: 72,
              amount: 1,
              name: "DinoEgg #281",
              decimals: 0,
              type: "EIP-004",
            },
            {
              tokenId:
                "4b9a62b2dd36b15f5f4c17bdc941cb9104c37ffb4296c298196464d7b501aaac",
              index: 73,
              amount: 1,
              name: "ErgoSapien Alpha #51",
              decimals: 0,
              type: "EIP-004",
            },
          ],
          additionalRegisters: {},
          spentTransactionId:
            "6f2e8307f74cec87566ae59e9b743c799e48c6645633f30d3c9df827bbb0c573",
          mainChain: true,
        },
      ];



it('i590 accidental token burn in change box', async () => {
  const height = 788658;

  const recipientAddress = sigmaRust.Address.from_mainnet_str(
    "9hS38ddo6GmP31xvCDkBpWCcD1saNgszsmK4FNgEdieGb27PUmi"
  );
  const changeAddress = sigmaRust.Address.from_mainnet_str(
    "9gNhk4UDgxAmYmzuXf9Gpte3SJ4rfkm8Amwy3CfCEntTTXh6vbS"
  );

  // INPUTS
  const inputs = sigmaRust.ErgoBoxes.from_boxes_json(unspentBoxes);

  // OUTPUTS
  const outputValue = sigmaRust.BoxValue.from_i64(
    sigmaRust.I64.from_str("1000000")
  );

  const tokens = new sigmaRust.Tokens();
  tokens.add(
    new sigmaRust.Token(
      sigmaRust.TokenId.from_str(
        "7d1568dce627141e6c9945328ef2bae5b7cc5c4f67fd873e288b808e68447db3"
      ),
      sigmaRust.TokenAmount.from_i64(sigmaRust.I64.from_str("1"))
    )
  );
  tokens.add(
    new sigmaRust.Token(
      sigmaRust.TokenId.from_str(
        "45a773bb751f4a1cb652624026a054b1fd7e526896d9b566938398d9bbdf2716"
      ),
      sigmaRust.TokenAmount.from_i64(sigmaRust.I64.from_str("1"))
    )
  );
  tokens.add(
    new sigmaRust.Token(
      sigmaRust.TokenId.from_str(
        "7d441ce250ffd0d70511899e5e9090f20a0443260983f76ef89a32cbb3eabda4"
      ),
      sigmaRust.TokenAmount.from_i64(sigmaRust.I64.from_str("1"))
    )
  );
  tokens.add(
    new sigmaRust.Token(
      sigmaRust.TokenId.from_str(
        "e21d27d9bbfec0a385e33c8987f62d14955ade38b3030d97681868b7492c76ad"
      ),
      sigmaRust.TokenAmount.from_i64(sigmaRust.I64.from_str("1"))
    )
  );
  tokens.add(
    new sigmaRust.Token(
      sigmaRust.TokenId.from_str(
        "1a77f5e1f185318c16c13402b4df133d439ab0e85382eab0b5598c7a8aa6a6c9"
      ),
      sigmaRust.TokenAmount.from_i64(sigmaRust.I64.from_str("1"))
    )
  );

  const builder = new sigmaRust.ErgoBoxCandidateBuilder(
    outputValue,
    sigmaRust.Contract.pay_to_address(recipientAddress),
    height
  );

  for (let i = 0; i < tokens.len(); i++) {
    builder.add_token(tokens.get(i).id(), tokens.get(i).amount());
  }

  const outputs = new sigmaRust.ErgoBoxCandidates(builder.build());
  const fee = sigmaRust.TxBuilder.SUGGESTED_TX_FEE();

  // TX BUILDING
  const targetBalance = sigmaRust.BoxValue.from_i64(
    outputValue.as_i64().checked_add(fee.as_i64())
  );
  const boxSelector = new sigmaRust.SimpleBoxSelector();
  const boxSelection = boxSelector.select(inputs, targetBalance, tokens);

  // Not selected tokens should be in the change box
  assert(boxSelection.change().get(0).tokens().len() != 0);

  // UNSIGNET TX
  const unsignedTx = sigmaRust.TxBuilder.new(
    boxSelection,
    outputs,
    height,
    fee,
    changeAddress
  ).build();
});
