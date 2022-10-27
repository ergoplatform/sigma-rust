use ergotree_macro::ergo_tree;

#[test]
fn test_method_call_flatmap_0() {
    // For ergoscript:
    //  { (x: Coll[Box]) => x.flatMap({(b: Box) => b.propositionBytes }) }
    let _ = ergo_tree!(
        FuncValue(
            Vector((1, SCollectionType(SBox))),
            MethodCall.typed[Value[SCollection[SByte.type]]](
              ValUse(1, SCollectionType(SBox)),
              SCollection.getMethodByName("flatMap").withConcreteTypes(
                Map(STypeVar("IV") -> SBox, STypeVar("OV") -> SByte)
              ),
              Vector(FuncValue(Vector((3, SBox)), ExtractScriptBytes(ValUse(3, SBox)))),
              Map()
            )
          )
    );
}

#[test]
fn test_method_call_flatmap_1() {
    // For ergoscript:
    //   { (x: Coll[GroupElement]) => x.flatMap({ (b: GroupElement) => b.getEncoded }) }
    let _ = ergo_tree!(
        FuncValue(
            Vector((1, SCollectionType(SGroupElement))),
            MethodCall.typed[Value[SCollection[SByte.type]]](
              ValUse(1, SCollectionType(SGroupElement)),
              SCollection.getMethodByName("flatMap").withConcreteTypes(
                Map(STypeVar("IV") -> SGroupElement, STypeVar("OV") -> SByte)
              ),
              Vector(
                FuncValue(
                  Vector((3, SGroupElement)),
                  MethodCall.typed[Value[SCollection[SByte.type]]](
                    ValUse(3, SGroupElement),
                    SGroupElement.getMethodByName("getEncoded"),
                    Vector(),
                    Map()
                  )
                )
              ),
              Map()
            )
          )
    );
}

#[test]
fn test_method_call_flatmap_2() {
    let _ = ergo_tree!(
        FuncValue(
            Vector((1, SCollectionType(SGroupElement))),
            MethodCall.typed[Value[SCollection[SInt.type]]](
              ValUse(1, SCollectionType(SGroupElement)),
              SCollection.getMethodByName("flatMap").withConcreteTypes(
                Map(STypeVar("IV") -> SGroupElement, STypeVar("OV") -> SInt)
              ),
              Vector(
                FuncValue(
                  Vector((3, SGroupElement)),
                  MethodCall.typed[Value[SCollection[SInt.type]]](
                    MethodCall.typed[Value[SCollection[SByte.type]]](
                      ValUse(3, SGroupElement),
                      SGroupElement.getMethodByName("getEncoded"),
                      Vector(),
                      Map()
                    ),
                    SCollection.getMethodByName("indices").withConcreteTypes(
                      Map(STypeVar("IV") -> SByte)
                    ),
                    Vector(),
                    Map()
                  )
                )
              ),
              Map()
            )
          )
    );
}

#[test]
fn test_method_call_zip_0() {
    // { (x: Coll[Box]) => x.zip(x) }
    let _ = ergo_tree!(
        FuncValue(
            Vector((1, SCollectionType(SBox))),
            MethodCall.typed[Value[SCollection[STuple]]](
              ValUse(1, SCollectionType(SBox)),
              SCollection.getMethodByName("zip").withConcreteTypes(
                Map(STypeVar("IV") -> SBox, STypeVar("OV") -> SBox)
              ),
              Vector(ValUse(1, SCollectionType(SBox))),
              Map()
            )
        )
    );
}
