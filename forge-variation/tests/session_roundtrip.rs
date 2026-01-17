use forge_variation::session::{load_session, save_session};
use forge_variation::{
    AssetClass, BaseInputRefV1, BaseInputType, DimensionsCm, ExportSettingsV1, Seed, SessionV1,
};

#[test]
fn session_roundtrip() {
    let mut s = SessionV1::new(
        AssetClass::ArenaProp,
        BaseInputRefV1 {
            input_type: BaseInputType::Drawn,
            source_path: "input.png".into(),
        },
        Seed(42),
    );

    s.push_intent("stone pillar, more damaged");
    s.generate_variations(5, "stone pillar, more damaged");

    let variation_id = s.variations[0].variation_id.clone();
    let _ = s
        .approve_variation(
            &variation_id,
            DimensionsCm {
                height: 250.0,
                width: 60.0,
                depth: 60.0,
            },
            ExportSettingsV1::default(),
            Some("pillar_a".into()),
        )
        .unwrap();

    let path = std::path::Path::new("target/test_session.forge.json");
    save_session(path, &s).unwrap();
    let s2 = load_session(path).unwrap();

    assert_eq!(s.session_id, s2.session_id);
    assert_eq!(s.variations.len(), s2.variations.len());
    assert_eq!(s.approvals.len(), s2.approvals.len());
}
