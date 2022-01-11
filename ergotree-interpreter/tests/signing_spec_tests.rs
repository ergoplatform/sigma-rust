use ergotree_interpreter::eval::context::Context;
use ergotree_interpreter::eval::env::Env;
use ergotree_interpreter::sigma_protocol::private_input::DlogProverInput;
use ergotree_interpreter::sigma_protocol::verifier::{TestVerifier, Verifier};
use ergotree_ir::ergo_tree::ErgoTree;
use ergotree_ir::mir::atleast::Atleast;
use ergotree_ir::mir::collection::Collection;
use ergotree_ir::mir::constant::{Constant, Literal};
use ergotree_ir::mir::expr::Expr;
use ergotree_ir::mir::sigma_and::SigmaAnd;
use ergotree_ir::mir::sigma_or::SigmaOr;
use ergotree_ir::mir::value::CollKind;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::sigma_protocol::sigma_boolean::{ProveDhTuple, SigmaProp};
use ergotree_ir::types::stype::SType;
use lazy_static::__Deref;
use num_bigint::BigUint;
use sigma_test_util::force_any_val;
use std::convert::TryInto;
use std::rc::Rc;

#[test]
fn sig_test_vector_provedlog() {
    // test vector data from
    // https://github.com/ScorexFoundation/sigmastate-interpreter/blob/6c51c13f7a494a191a7ea5645e56b04fb46a418d/sigmastate/src/test/scala/sigmastate/crypto/SigningSpecification.scala#L14-L30
    let msg = base16::decode(b"1dc01772ee0171f5f614c673e3c7fa1107a8cf727bdf5a6dadb379e93c0d1d00")
        .unwrap();
    let sk = DlogProverInput::from_biguint(
        BigUint::parse_bytes(
            b"109749205800194830127901595352600384558037183218698112947062497909408298157746",
            10,
        )
        .unwrap(),
    )
    .unwrap();
    let signature = base16::decode(b"bcb866ba434d5c77869ddcbc3f09ddd62dd2d2539bf99076674d1ae0c32338ea95581fdc18a3b66789904938ac641eba1a66d234070207a2").unwrap();

    // check expected public key
    assert_eq!(
        base16::encode_lower(&sk.public_image().sigma_serialize_bytes().unwrap()),
        "03cb0d49e4eae7e57059a3da8ac52626d26fc11330af8fb093fa597d8b93deb7b1"
    );

    let expr: Expr = Expr::Const(sk.public_image().into());
    let verifier = TestVerifier;
    let ver_res = verifier.verify(
        &expr.try_into().unwrap(),
        &Env::empty(),
        Rc::new(force_any_val::<Context>()),
        signature.into(),
        msg.as_slice(),
    );
    assert!(ver_res.unwrap().result);
}

#[test]
fn sig_test_vector_prove_dht() {
    // corresponding sigmastate test
    // in SigningSpecification.property("ProveDHT signature test vector")
    let msg = base16::decode(b"1dc01772ee0171f5f614c673e3c7fa1107a8cf727bdf5a6dadb379e93c0d1d00")
        .unwrap();
    let pdht = ProveDhTuple::sigma_parse_bytes(&base16::decode(b"0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f817980280c66feee88d56e47bf3f47c4109d9218c60c373a472a0d9537507c7ee828c4802a96f19e97df31606183c1719400682d1d40b1ce50c9a1ed1b19845e2b1b551bf0255ac02191cb229891fb1b674ea9df7fc8426350131d821fc4a53f29c3b1cb21a").unwrap()).unwrap();
    // let pdht = random_pdht_input.public_image().clone();
    dbg!(base16::encode_lower(&pdht.sigma_serialize_bytes().unwrap()));
    let signature = base16::decode(b"eba93a69b28cfdea261e9ea8914fca9a0b3868d50ce68c94f32e875730f8ca361bd3783c5d3e25802e54f49bd4fb9fafe51f4e8aafbf9815").unwrap();
    let expr: Expr = Expr::Const(pdht.into());

    // let random_pdht_input = DhTupleProverInput::random();
    // let tree: ErgoTree = expr.clone().into();
    // let prover = TestProver {
    //     secrets: vec![random_pdht_input.into()],
    // };
    // let res = prover.prove(
    //     &tree,
    //     &Env::empty(),
    //     Rc::new(force_any_val::<Context>()),
    //     msg.as_slice(),
    //     &HintsBag::empty(),
    // );
    // let proof: Vec<u8> = res.unwrap().proof.into();
    // dbg!(base16::encode_lower(&proof));

    let verifier = TestVerifier;
    let ver_res = verifier.verify(
        &expr.try_into().unwrap(),
        &Env::empty(),
        Rc::new(force_any_val::<Context>()),
        signature.into(),
        msg.as_slice(),
    );
    assert!(ver_res.unwrap().result);
}

#[test]
fn sig_test_vector_conj_and() {
    // corresponding sigmastate test
    // in SigningSpecification.property("AND signature test vector")
    let msg = base16::decode(b"1dc01772ee0171f5f614c673e3c7fa1107a8cf727bdf5a6dadb379e93c0d1d00")
        .unwrap();
    let sk1 = DlogProverInput::from_biguint(
        BigUint::parse_bytes(
            b"109749205800194830127901595352600384558037183218698112947062497909408298157746",
            10,
        )
        .unwrap(),
    )
    .unwrap();
    let sk2 = DlogProverInput::from_biguint(
        BigUint::parse_bytes(
            b"50415569076448343263191022044468203756975150511337537963383000142821297891310",
            10,
        )
        .unwrap(),
    )
    .unwrap();

    let signature = base16::decode(b"9b2ebb226be42df67817e9c56541de061997c3ea84e7e72dbb69edb7318d7bb525f9c16ccb1adc0ede4700a046d0a4ab1e239245460c1ba45e5637f7a2d4cc4cc460e5895125be73a2ca16091db2dcf51d3028043c2b9340").unwrap();

    let expr: Expr = SigmaAnd::new(vec![
        Expr::Const(sk1.public_image().into()),
        Expr::Const(sk2.public_image().into()),
    ])
    .unwrap()
    .into();
    let tree: ErgoTree = expr.try_into().unwrap();

    // let prover = TestProver {
    //     secrets: vec![sk1.into(), sk2.into()],
    // };
    // let res = prover.prove(
    //     &tree,
    //     &Env::empty(),
    //     Rc::new(force_any_val::<Context>()),
    //     msg.as_slice(),
    //     &HintsBag::empty(),
    // );
    // let proof: Vec<u8> = res.unwrap().proof.into();
    // dbg!(base16::encode_lower(&proof));

    let verifier = TestVerifier;
    let ver_res = verifier.verify(
        &tree,
        &Env::empty(),
        Rc::new(force_any_val::<Context>()),
        signature.into(),
        msg.as_slice(),
    );
    assert!(ver_res.unwrap().result);
}

#[test]
fn sig_test_vector_conj_or() {
    // corresponding sigmastate test
    // in SigningSpecification.property("OR signature test vector")
    let msg = base16::decode(b"1dc01772ee0171f5f614c673e3c7fa1107a8cf727bdf5a6dadb379e93c0d1d00")
        .unwrap();
    let sk1 = DlogProverInput::from_biguint(
        BigUint::parse_bytes(
            b"109749205800194830127901595352600384558037183218698112947062497909408298157746",
            10,
        )
        .unwrap(),
    )
    .unwrap();
    let sk2 = DlogProverInput::from_biguint(
        BigUint::parse_bytes(
            b"50415569076448343263191022044468203756975150511337537963383000142821297891310",
            10,
        )
        .unwrap(),
    )
    .unwrap();

    let signature = base16::decode(b"ec94d2d5ef0e1e638237f53fd883c339f9771941f70020742a7dc85130aaee535c61321aa1e1367befb500256567b3e6f9c7a3720baa75ba6056305d7595748a93f23f9fc0eb9c1aaabc24acc4197030834d76d3c95ede60c5b59b4b306cd787d010e8217f34677d046646778877c669").unwrap();

    let expr: Expr = SigmaOr::new(vec![
        Expr::Const(sk1.public_image().into()),
        Expr::Const(sk2.public_image().into()),
    ])
    .unwrap()
    .into();
    let tree: ErgoTree = expr.try_into().unwrap();

    // let prover = TestProver {
    //     secrets: vec![sk1.into()],
    // };
    // let res = prover.prove(
    //     &tree,
    //     &Env::empty(),
    //     Rc::new(force_any_val::<Context>()),
    //     msg.as_slice(),
    //     &HintsBag::empty(),
    // );
    // let proof: Vec<u8> = res.unwrap().proof.into();
    // dbg!(base16::encode_lower(&proof));

    let verifier = TestVerifier;
    let ver_res = verifier.verify(
        &tree,
        &Env::empty(),
        Rc::new(force_any_val::<Context>()),
        signature.into(),
        msg.as_slice(),
    );
    assert!(ver_res.unwrap().result);
}

#[test]
fn sig_test_vector_conj_or_prove_dht() {
    // corresponding sigmastate test
    // in SigningSpecification.property("OR with ProveDHT signature test vector")
    let msg = base16::decode(b"1dc01772ee0171f5f614c673e3c7fa1107a8cf727bdf5a6dadb379e93c0d1d00")
        .unwrap();
    let sk1 = DlogProverInput::from_biguint(
        BigUint::parse_bytes(
            b"109749205800194830127901595352600384558037183218698112947062497909408298157746",
            10,
        )
        .unwrap(),
    )
    .unwrap();
    let pdht = ProveDhTuple::sigma_parse_bytes(&base16::decode(b"0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f817980214487635ebffa60b13a166bd0721c5f0ab603fc74168d7764d7ec5ef2107f5d40334c5b7efa5a4a22b83d102d2e6521eaa660fa911c5a213af63c8460f2327513b026a0be2a277291d42daad3830cb16a4ef20e4f1f7c36384f3fee065f0f143a355").unwrap()).unwrap();
    // let random_pdht_input = DhTupleProverInput::random();
    // let pdht = random_pdht_input.public_image().clone();
    // dbg!(base16::encode_lower(&pdht.sigma_serialize_bytes()));
    let signature = base16::decode(b"a80daebdcd57874296f49fd9910ddaefbf517ca076b6e16b97678e96a20239978836e7ec5b795cf3a55616d394f07c004f85e0d3e71880d4734b57ea874c7eba724e8887280f1affadaad962ee916b39207af2d2ab2a69a2e6f4d652f7389cc4f582bbe6d7937c59aa64cf2965a8b36a").unwrap();
    let expr: Expr = SigmaOr::new(vec![
        Expr::Const(sk1.public_image().into()),
        Expr::Const(pdht.into()),
    ])
    .unwrap()
    .into();

    // let tree: ErgoTree = expr.clone().into();
    // let prover = TestProver {
    //     secrets: vec![random_pdht_input.into()],
    // };
    // let res = prover.prove(
    //     &tree,
    //     &Env::empty(),
    //     Rc::new(force_any_val::<Context>()),
    //     msg.as_slice(),
    //     &HintsBag::empty(),
    // );
    // let proof: Vec<u8> = res.unwrap().proof.into();
    // dbg!(base16::encode_lower(&proof));

    let verifier = TestVerifier;
    let ver_res = verifier.verify(
        &expr.try_into().unwrap(),
        &Env::empty(),
        Rc::new(force_any_val::<Context>()),
        signature.into(),
        msg.as_slice(),
    );
    assert!(ver_res.unwrap().result);
}

#[test]
fn sig_test_vector_conj_and_or() {
    // corresponding sigmastate test
    // in SigningSpecification.property("AND with OR signature test vector")
    let msg = base16::decode(b"1dc01772ee0171f5f614c673e3c7fa1107a8cf727bdf5a6dadb379e93c0d1d00")
        .unwrap();
    let sk1 = DlogProverInput::from_biguint(
        BigUint::parse_bytes(
            b"109749205800194830127901595352600384558037183218698112947062497909408298157746",
            10,
        )
        .unwrap(),
    )
    .unwrap();
    let sk2 = DlogProverInput::from_biguint(
        BigUint::parse_bytes(
            b"50415569076448343263191022044468203756975150511337537963383000142821297891310",
            10,
        )
        .unwrap(),
    )
    .unwrap();

    let sk3 = DlogProverInput::from_biguint(
        BigUint::parse_bytes(
            b"34648336872573478681093104997365775365807654884817677358848426648354905397359",
            10,
        )
        .unwrap(),
    )
    .unwrap();

    let signature = base16::decode(b"397e005d85c161990d0e44853fbf14951ff76e393fe1939bb48f68e852cd5af028f6c7eaaed587f6d5435891a564d8f9a77288773ce5b526a670ab0278aa4278891db53a9842df6fba69f95f6d55cfe77dd7b4bdccc1a3378ac4524b51598cb813258f64c94e98c3ef891a6eb8cbfd2e527a9038ca50b5bb50058de55a859a169628e6ae5ba4cb0332c694e450782d6f").unwrap();

    let expr: Expr = SigmaAnd::new(vec![
        Expr::Const(sk1.public_image().into()),
        SigmaOr::new(vec![
            Expr::Const(sk2.public_image().into()),
            Expr::Const(sk3.public_image().into()),
        ])
        .unwrap()
        .into(),
    ])
    .unwrap()
    .into();
    let tree: ErgoTree = expr.try_into().unwrap();

    // let prover = TestProver {
    //     secrets: vec![sk1.into(), sk2.into()],
    // };
    // let res = prover.prove(
    //     &tree,
    //     &Env::empty(),
    //     Rc::new(force_any_val::<Context>()),
    //     msg.as_slice(),
    //     &HintsBag::empty(),
    // );
    // let proof: Vec<u8> = res.unwrap().proof.into();
    // dbg!(base16::encode_lower(&proof));

    let verifier = TestVerifier;
    let ver_res = verifier.verify(
        &tree,
        &Env::empty(),
        Rc::new(force_any_val::<Context>()),
        signature.into(),
        msg.as_slice(),
    );
    assert!(ver_res.unwrap().result);
}

#[test]
fn sig_test_vector_conj_or_and() {
    // corresponding sigmastate test
    // in SigningSpecification.property("OR with AND signature test vector")
    let msg = base16::decode(b"1dc01772ee0171f5f614c673e3c7fa1107a8cf727bdf5a6dadb379e93c0d1d00")
        .unwrap();
    let sk1 = DlogProverInput::from_biguint(
        BigUint::parse_bytes(
            b"109749205800194830127901595352600384558037183218698112947062497909408298157746",
            10,
        )
        .unwrap(),
    )
    .unwrap();
    let sk2 = DlogProverInput::from_biguint(
        BigUint::parse_bytes(
            b"50415569076448343263191022044468203756975150511337537963383000142821297891310",
            10,
        )
        .unwrap(),
    )
    .unwrap();

    let sk3 = DlogProverInput::from_biguint(
        BigUint::parse_bytes(
            b"34648336872573478681093104997365775365807654884817677358848426648354905397359",
            10,
        )
        .unwrap(),
    )
    .unwrap();

    let signature = base16::decode(b"a58b251be319a9656c21876b1136a59f42b18835dec6076c92f7a925ba28d2030218c177ab07563003eff5250cfafeb631ef610f4d710ab8e821bf632203adf23f4376580eaa17ddb36c0138f73a88551f45d92cde2b66dfbb5906c02e4d48106ff08be4a2fc29ec242f495468692f9ddeeb029dc5d8f38e2649cf09c44b67cbcfb3de4202026fb84d23ce2b4ff0f69b").unwrap();

    let expr: Expr = SigmaOr::new(vec![
        Expr::Const(sk1.public_image().into()),
        SigmaAnd::new(vec![
            Expr::Const(sk2.public_image().into()),
            Expr::Const(sk3.public_image().into()),
        ])
        .unwrap()
        .into(),
    ])
    .unwrap()
    .into();
    let tree: ErgoTree = expr.try_into().unwrap();

    // let prover = TestProver {
    //     secrets: vec![sk1.into(), sk2.into()],
    // };
    // let res = prover.prove(
    //     &tree,
    //     &Env::empty(),
    //     Rc::new(force_any_val::<Context>()),
    //     msg.as_slice(),
    //     &HintsBag::empty(),
    // );
    // let proof: Vec<u8> = res.unwrap().proof.into();
    // dbg!(base16::encode_lower(&proof));

    let verifier = TestVerifier;
    let ver_res = verifier.verify(
        &tree,
        &Env::empty(),
        Rc::new(force_any_val::<Context>()),
        signature.into(),
        msg.as_slice(),
    );
    assert!(ver_res.unwrap().result);
}

#[test]
fn sig_test_vector_threshold() {
    // corresponding sigmastate test
    // in SigningSpecification.property("threshold signature test vector")
    let msg = base16::decode(b"1dc01772ee0171f5f614c673e3c7fa1107a8cf727bdf5a6dadb379e93c0d1d00")
        .unwrap();

    let mut sk1_bytes = BigUint::parse_bytes(
        b"416167686186183758173232992934554728075978573242452195968805863126437865059",
        10,
    )
    .unwrap()
    .to_bytes_be();
    // sk1 string is only 31 bytes so pad with zero so it fits the required 32 byte buffers
    sk1_bytes.insert(0, 0u8);

    let sk1 = DlogProverInput::from_bytes(sk1_bytes.as_slice().try_into().unwrap()).unwrap();
    let sk2 = DlogProverInput::from_biguint(
        BigUint::parse_bytes(
            b"34648336872573478681093104997365775365807654884817677358848426648354905397359",
            10,
        )
        .unwrap(),
    )
    .unwrap();

    let sk3 = DlogProverInput::from_biguint(
        BigUint::parse_bytes(
            b"50415569076448343263191022044468203756975150511337537963383000142821297891310",
            10,
        )
        .unwrap(),
    )
    .unwrap();

    let signature = base16::decode(b"0b6bf9bc42c7b509ab56c76318c0891b2c8d44ef5fafb1379cc6b72b89c53cd43f8ef10158ce08646301d09b450ea83a1cdbbfc3dc7438ece4bbe934919069c50ec5857209b0dbf120b325c88667bc84580720ff4b3c371ec752bc6874c933f7fa53fae411e65ae07b647d365caac8c6744276c04c0240dd55e1f62c0e17a093dd91493c68104b1e01a4069017668d3f").unwrap();

    let bound = Expr::Const(2i32.into());

    let inputs = Literal::Coll(
        CollKind::from_vec(
            SType::SSigmaProp,
            vec![
                SigmaProp::from(sk1.public_image()).into(),
                SigmaProp::from(sk2.public_image()).into(),
                SigmaProp::from(sk3.public_image()).into(),
            ],
        )
        .unwrap(),
    );

    let input = Constant {
        tpe: SType::SColl(SType::SSigmaProp.into()),
        v: inputs.clone(),
    }
    .into();

    let expr: Expr = Atleast::new(bound, input).unwrap().into();
    let verifier = TestVerifier;
    let ver_res = verifier.verify(
        &expr.try_into().unwrap(),
        &Env::empty(),
        Rc::new(force_any_val::<Context>()),
        signature.into(),
        msg.as_slice(),
    );
    assert!(ver_res.unwrap().result);
}
