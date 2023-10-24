use ydb_grpc_bindings::generated::ydb::r#type::PrimitiveTypeId;

use super::*;

#[test]
fn synt() {
    match val(r#"(
    (return ())
    )"#) { 
        Ok((_, r)) => println!("ok: {r:?}"),
        Err(e) => println!("e: {e:?}"),
    }
}

fn check(s: &str) {
    let mut v = val(s).unwrap().1;
    v.eval();
    let params = invoke_outputs(&v).unwrap();
    println!("columns:");
    for (name, typ, is_optional) in params {
        let is_optional = if is_optional {"optional "} else {""};
        println!("{name}: {is_optional}{typ}");
    }
}

#[test]
fn it_works1() {
    check(AST1);
}
#[test]
fn it_works2() {
    check(AST2);
}
#[test]
fn it_works3() {
    check(AST3);
}
#[test]
fn it_works4() {
    check(AST4);
}

fn invoke_outputs<'a, 'b: 'a>(node: &'b Node<'a>) -> Option<Vec<(&'a str, &'a str, bool)>> {
    let outputs = node.find("return")?.find("KqpPhysicalQuery")?.find("KqpTxResultBinding")?.find("StructType")?.list()?;
    let result = outputs.into_iter().filter_map(|item| {
        let mut iter = item.list()?.into_iter();
        let mut is_optional = false;
        let name = iter.next()?.text()?;
        let typ = iter.next()?;
        let typ = if typ.text()? == "OptionalType" {
            is_optional = true;
            typ.list()?.into_iter().next()?
        } else {
            typ
        }.list()?.into_iter().next()?;
        let _t = PrimitiveTypeId::from_str_name(&typ.text()?.to_ascii_uppercase());
        Some((name, typ.text()?, is_optional))
    }).collect();
    Some(result)
}





//===============test values
static AST1: &str = r#"(
    (let $1 (DqPhyStage '() (lambda '() (Iterator (AsList (AsStruct '('"one" (Int32 '"1")) '('"three" (RandomUuid (DependsOn (Int32 '0)))) '('"two" (String '"x")))))) '('('"_logical_id" '233) '('"_id" '"9758b54e-1a50ed2b-43aefc24-cd9d7bc5"))))
    (let $2 '('"one" '"two" '"three"))
    (let $3 (DqCnResult (TDqOutput $1 '0) $2))
    (return (
        KqpPhysicalQuery 
            '( (KqpPhysicalTx '($1) '($3) '() '('('type '"compute"))) ) 
            '( (KqpTxResultBinding (ListType (StructType 
                                                '('"one" (DataType 'Int32)) 
                                                '('"three" (DataType 'Uuid)) 
                                                '('"two" (DataType 'String))
                                            )) '0 '0) ) 
            '('('type '"data_query"))
        )
    )
    )"#;
static AST2: &str = r#"(
    (let $1 (KqpTable '"episodes" '"72075186224045943:83859" '"" '1))
    (let $2 '('"episode_id" '"season_id" '"title" '"series_id"))
    (let $3 (Uint64 '1))
    (let $4 (KqpRowsSourceSettings $1 $2 '() '((KqlKeyExc $3 $3) (KqlKeyInc $3))))
    (let $5 (Uint64 '"3"))
    (let $6 (DqPhyStage '((DqSource (DataSource '"KqpReadRangesSource") $4)) (lambda '($12) (block '(
      (let $13 (Bool 'true))
      (return (FromFlow (TopSort (OrderedMap (OrderedFilter (ToFlow $12) (lambda '($14) (And (Exists (Member $14 '"season_id")) (Exists (Member $14 '"series_id"))))) (lambda '($15) (AsStruct '('"episode_id" (Member $15 '"episode_id")) '('"season_id" (Member $15 '"season_id")) '('"title" (Member $15 '"title"))))) $5 '($13 $13) (lambda '($16) '((Member $16 '"season_id") (Member $16 '"episode_id"))))))
    ))) '('('"_logical_id" '842) '('"_id" '"14388682-e03b28dd-60f91f84-d7cd002f"))))
    (let $7 (DqCnMerge (TDqOutput $6 '0) '('('"season_id" '"Asc") '('"episode_id" '"Asc"))))
    (let $8 (DqPhyStage '($7) (lambda '($17) (FromFlow (Take (ToFlow $17) $5))) '('('"_logical_id" '855) '('"_id" '"295c0459-34d4b026-b467e71f-35402430"))))
    (let $9 '('"season_id" '"episode_id" '"title"))
    (let $10 (DqCnResult (TDqOutput $8 '0) $9))
    (let $11 (OptionalType (DataType 'Uint64)))
    (return (KqpPhysicalQuery '((KqpPhysicalTx '($6 $8) '($10) '() '('('"type" '"data")))) '((KqpTxResultBinding (ListType (StructType '('"episode_id" $11) '('"season_id" $11) '('"title" (OptionalType (DataType 'String))))) '0 '0)) '('('"type" '"data_query"))))
    )"#;
static AST3: &str = r#"(
    (declare %kqp%tx_result_binding_0_0 (ListType (StructType '('"bot_id" (DataType 'Int32)) '('"user" (DataType 'Int32)) '('"username" (OptionalType (DataType 'Utf8))))))
    (declare %kqp%tx_result_binding_0_1 (DataType 'Bool))
    (declare %kqp%tx_result_binding_0_2 (ListType (StructType '('"bot_id" (DataType 'Int32)) '('"user" (DataType 'Int32)))))
    (declare %kqp%tx_result_binding_1_0 (DataType 'Bool))
    (declare %kqp%tx_result_binding_2_0 (ListType (StructType '('"bot_id" (DataType 'Int32)) '('"user" (DataType 'Int32)) '('"username" (OptionalType (DataType 'Utf8))))))
    (let $1 (DataType 'Int32))
    (let $2 '('"bot_id" $1))
    (let $3 '('"user" $1))
    (let $4 (ListType (StructType $2 $3 '('"username" (OptionalType (DataType 'Utf8))))))
    (let $5 (ListType (StructType $2 $3)))
    (let $6 (DataType 'Bool))
    (let $7 (Uint64 '1))
    (let $8 (DqPhyStage '() (lambda '() (block '(
      (let $44 '('"bot_id" (Int32 '1)))
      (let $45 '('"user" (Int32 '2)))
      (let $46 (VariantType (TupleType $4 $5 $6)))
      (let $47 (Variant (AsList (AsStruct $44 $45 '('"username" (Just (Utf8 '"bgg"))))) '"0" $46))
      (let $48 (ToDict (List $5 (AsStruct $44 $45)) (lambda '($51) $51) (lambda '($52) (Void)) '('"One" '"Hashed")))
      (let $49 (Variant (DictKeys $48) '1 $46))
      (let $50 (Variant (== $7 (Length $48)) '2 $46))
      (return (Iterator (AsList $47 $49 $50)))
    ))) '('('"_logical_id" '528) '('"_id" '"6075b854-1d647c42-2c2a318b-71c9560f"))))
    (let $9 (DqCnValue (TDqOutput $8 '"0")))
    (let $10 (DqCnValue (TDqOutput $8 '2)))
    (let $11 (DqCnValue (TDqOutput $8 '1)))
    (let $12 '($9 $10 $11))
    (let $13 '('('"type" '"compute")))
    (let $14 (KqpPhysicalTx '($8) $12 '() $13))
    (let $15 '"/ru-central1/b1gtv82sacrcnutlfktm/etn8sgrgdbp7jqv64k9f/bot_admins")
    (let $16 (KqpTable $15 '"72075186321596424:567" '"" '2))
    (let $17 '"%kqp%tx_result_binding_0_2")
    (let $18 (Bool 'false))
    (let $19 (DqPhyStage '() (lambda '() (block '(
      (let $53 (KqpLookupTable $16 (Iterator %kqp%tx_result_binding_0_2) '()))
      (return (FromFlow (Map (Take $53 $7) (lambda '($54) $18))))
    ))) '('('"_logical_id" '675) '('"_id" '"8d07adb-9aed067e-ac73002c-e91716eb"))))
    (let $20 (DqCnUnionAll (TDqOutput $19 '"0")))
    (let $21 (Bool 'true))
    (let $22 (DqPhyStage '($20) (lambda '($55) (block '(
      (let $56 (lambda '($57 $58) $18))
      (return (FromFlow (Condense (ToFlow $55) $21 $56 $56)))
    ))) '('('"_logical_id" '658) '('"_id" '"69fa87d1-dc8d6d84-7895842a-b119b59a"))))
    (let $23 (DqCnValue (TDqOutput $22 '"0")))
    (let $24 (KqpTxResultBinding $5 '"0" '2))
    (let $25 '('"type" '"data"))
    (let $26 (KqpPhysicalTx '($19 $22) '($23) '('($17 $24)) '($25)))
    (let $27 '"%kqp%tx_result_binding_0_1")
    (let $28 '"%kqp%tx_result_binding_1_0")
    (let $29 '"%kqp%tx_result_binding_0_0")
    (let $30 (DqPhyStage '() (lambda '() (block '(
      (let $59 (KqpEnsure $21 %kqp%tx_result_binding_0_1 '"2012" (Utf8 '"Duplicated keys found.")))
      (let $60 (KqpEnsure $21 %kqp%tx_result_binding_1_0 '"2012" (Utf8 '"Conflict with existing key.")))
      (return (Iterator (If (And $59 $60) %kqp%tx_result_binding_0_0 (List $4))))
    ))) '('('"_logical_id" '821) '('"_id" '"d2d8f383-1c08162e-ad538563-9a96ccd"))))
    (let $31 (DqCnUnionAll (TDqOutput $30 '"0")))
    (let $32 (DqPhyStage '($31) (lambda '($61) $61) '('('"_logical_id" '847) '('"_id" '"43e6c9f1-eee1ac7-ec362adc-b266ef93"))))
    (let $33 (DqCnResult (TDqOutput $32 '"0") '()))
    (let $34 (KqpTxResultBinding $4 '"0" '"0"))
    (let $35 (KqpTxResultBinding $6 '"0" '1))
    (let $36 (KqpTxResultBinding $6 '1 '"0"))
    (let $37 '('($29 $34) '($27 $35) '($28 $36)))
    (let $38 (KqpPhysicalTx '($30 $32) '($33) $37 $13))
    (let $39 '"%kqp%tx_result_binding_2_0")
    (let $40 (DqPhyStage '() (lambda '() (block '(
      (let $62 '('"bot_id" '"user" '"username"))
      (return (KqpEffects (KqpUpsertRows $16 (Iterator %kqp%tx_result_binding_2_0) $62 '())))
    ))) '('('"_logical_id" '945) '('"_id" '"55bf0a72-5d89e24f-c9a63e15-166c2899"))))
    (let $41 (KqpTxResultBinding $4 '2 '"0"))
    (let $42 (KqpPhysicalTx '($40) '() '('($39 $41)) '($25 '('"with_effects"))))
    (let $43 '($14 $26 $38 $42))
    (return (KqpPhysicalQuery $43 '() '('('"type" '"data_query"))))
    )"#;
static AST4: &str = r#"(
    (declare %kqp%tx_result_binding_0_0 (ListType (StructType '('"bot_id" (DataType 'Int32)) '('"username" (OptionalType (DataType 'Utf8))))))
    (declare %kqp%tx_result_binding_1_0 (ListType (StructType '('"bot_id" (DataType 'Int32)))))
    (let $1 (DqPhyStage '() (lambda '() (block '(
      (let $28 (KqpTable '"/Root/test/bot_info" '"72075186224037889:4" '"" '1))
      (let $29 (KqpWideReadTableRanges $28 (Void) '('"bot_id" '"username") '() '()))
      (return (FromFlow (NarrowMap $29 (lambda '($30 $31) (AsStruct '('"bot_id" $30) '('"username" $31))))))
    ))) '('('"_logical_id" '1294) '('"_id" '"de8956f0-e8c03901-22bc7765-a55a6314"))))
    (let $2 (DqCnUnionAll (TDqOutput $1 '0)))
    (let $3 (DqPhyStage '($2) (lambda '($32) $32) '('('"_logical_id" '1312) '('"_id" '"e2677c81-aba1ef15-20cbc46f-6315c6c1"))))
    (let $4 (DqCnResult (TDqOutput $3 '0) '()))
    (let $5 '('('type '"data")))
    (let $6 (KqpPhysicalTx '($1 $3) '($4) '() $5))
    (let $7 '"%kqp%tx_result_binding_0_0")
    (let $8 (DataType 'Int32))
    (let $9 '('"bot_id" $8))
    (let $10 (OptionalType (DataType 'Utf8)))
    (let $11 (ListType (StructType $9 '('"username" $10))))
    (let $12 %kqp%tx_result_binding_0_0)
    (let $13 (DqPhyStage '() (lambda '() (ToStream (Just (PartitionByKey $12 (lambda '($33) (Member $33 '"bot_id")) (Void) (Void) (lambda '($34) (FlatMap $34 (lambda '($35) (Map (Take (Nth $35 '1) (Uint64 '1)) (lambda '($36) (AsStruct '('"bot_id" (Member $36 '"bot_id")))))))))))) '('('"_logical_id" '1438) '('"_id" '"7721cf9c-2dca099f-2302962d-8fe9d53f"))))
    (let $14 (DqCnValue (TDqOutput $13 '0)))
    (let $15 (KqpTxResultBinding $11 '0 '0))
    (let $16 '($7 $15))
    (let $17 (KqpPhysicalTx '($13) '($14) '($16) '('('type '"compute"))))
    (let $18 '"%kqp%tx_result_binding_1_0")
    (let $19 (ListType (StructType $9)))
    (let $20 (Uint64 '"1001"))
    (let $21 (DqPhyStage '() (lambda '() (block '(
      (let $37 '('Many 'Hashed 'Compact))
      (let $38 (SqueezeToDict (Map (ToFlow $12) (lambda '($39) '((Member $39 '"bot_id") $39))) (lambda '($40) (Nth $40 '0)) (lambda '($41) (Nth $41 '1)) $37))
      (return (FromFlow (Take (FlatMap $38 (lambda '($42) (block '(
        (let $43 (KqpTable '"/Root/test/bot_admins" '"72075186224037889:3" '"" '1))
        (let $44 (KqpLookupTable $43 (Iterator %kqp%tx_result_binding_1_0) '('"bot_id" '"user")))
        (let $45 '('"bot_id" '"i.bot_id" '"username" '"i.username"))
        (return (MapJoinCore (Map (Filter $44 (lambda '($46) (== (Member $46 '"user") (Int32 '1)))) (lambda '($47) (AsStruct '('"bot_id" (Member $47 '"bot_id"))))) $42 'Inner '('"bot_id") '() $45))
      )))) $20)))
    ))) '('('"_logical_id" '1248) '('"_id" '"1fe0ac30-86e2a470-f7959514-ae97b8d"))))
    (let $22 (DqCnUnionAll (TDqOutput $21 '0)))
    (let $23 (DqPhyStage '($22) (lambda '($48) (FromFlow (Take (ToFlow $48) $20))) '('('"_logical_id" '1261) '('"_id" '"2033ac6b-d8b890ec-904e0ff4-ea03bd9"))))
    (let $24 (DqCnResult (TDqOutput $23 '0) '('"i.bot_id" '"i.username")))
    (let $25 (KqpTxResultBinding $19 '1 '0))
    (let $26 (KqpPhysicalTx '($21 $23) '($24) '($16 '($18 $25)) $5))
    (let $27 '($6 $17 $26))
    (return (KqpPhysicalQuery $27 '((KqpTxResultBinding (ListType (StructType '('"i.bot_id" $8) '('"i.username" $10))) '"2" '0)) '('('type '"data_query"))))
    )"#;
