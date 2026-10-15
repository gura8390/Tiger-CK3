//! CK3 1.20 rite and tenet schemas. Source: local *_types.info, not permissive fallbacks.
use crate::block::{BV, Block};
use crate::ck3::data::religions::{validate_doctrine_list, validate_tenet_selection};
use crate::ck3::modif::ModifKinds;
use crate::ck3::validate::validate_traits;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::validate_modifs;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_possibly_named_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Rite {}
inventory::submit! { ItemLoader::Normal(GameFlags::Ck3, Item::Rite, Rite::add) }
impl Rite {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Rite, key, block, Box::new(Self {}));
    }
}
impl DbKind for Rite {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Rite, key);
        if !vd.field_validated_sc("name", &mut sc, validate_desc) {
            data.verify_exists(Item::Localization, key);
        }
        if !vd.field_validated_sc("desc", &mut sc, validate_desc) {
            data.verify_exists_implied(Item::Localization, &format!("{key}_desc"), key);
        }
        vd.field_icon("icon", "NGameIcons|DYNAMIC_RITE_ICON_PATH", ".dds");
        vd.field_validated("color", validate_possibly_named_color);
        vd.field_item("founder", Item::Title);
        vd.field_item("faith", Item::Faith);
        vd.field_validated_block(
            "localization",
            crate::ck3::data::religions::validate_localization,
        );
        vd.field_list_items("cultures", Item::Culture);
        vd.field_bool("convert");
        vd.field_bool("create");
        vd.field_bool("history");
        vd.field_list_items("tenets", Item::Tenet);
        validate_doctrine_list("rite", block, data, &mut vd);
        validate_tenet_selection(&mut vd);
    }
}

#[derive(Clone, Debug)]
pub struct Tenet {}
inventory::submit! { ItemLoader::Normal(GameFlags::Ck3, Item::Tenet, Tenet::add) }
impl Tenet {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Tenet, key, block, Box::new(Self {}));
    }
}
impl DbKind for Tenet {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(parameters) = block.get_field_block("personal_tenet_parameters") {
            for key in parameters.iter_values() {
                db.add_flag(Item::PersonalTenetParameter, key.clone());
            }
        }
        if let Some(parameters) = block.get_field_block("parameters") {
            for key in parameters.iter_values() {
                db.add_flag(Item::DoctrineParameter, key.clone());
                db.add_flag(Item::DoctrineBooleanParameter, key.clone());
            }
            for (key, value) in parameters.iter_assignments() {
                db.add_flag(Item::DoctrineParameter, key.clone());
                if value.is("yes") || value.is("no") {
                    db.add_flag(Item::DoctrineBooleanParameter, key.clone());
                }
            }
        }
    }
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Faith, key);
        sc.define_list("selected_tenets", Scopes::Tenet.union(Scopes::Flag), key);
        sc.define_list("selected_doctrines", Scopes::Tenet.union(Scopes::Doctrine), key);
        sc.define_name("actor", Scopes::Character, key);
        // The .info describes faith descriptions, while vanilla triggered names/descs
        // also call rite-scoped liturgical-language triggers. Keep both concrete
        // contexts here, without widening can_pick/is_shown to arbitrary scopes.
        let mut description_sc = ScopeContext::new(Scopes::Faith.union(Scopes::Rite), key);
        if !vd.field_validated_sc("name", &mut description_sc, validate_desc) {
            data.verify_exists_implied(Item::Localization, &format!("{key}_name"), key);
        }
        if !vd.field_validated_sc("desc", &mut description_sc, validate_desc) {
            data.verify_exists_implied(Item::Localization, &format!("{key}_desc"), key);
        }
        let mut glossary_sc = ScopeContext::new(Scopes::Rite, key);
        vd.field_validated_sc("historical_glossary", &mut glossary_sc, validate_desc);
        if !vd.multi_field_validated("icon", |bv, data| match bv {
            BV::Value(icon) => data.verify_icon("NGameIcons|TENET_TYPE_ICON_PATH", icon, ".dds"),
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.field_trigger_rooted("trigger", Tooltipped::No, Scopes::Faith);
                vd.req_field("reference");
                vd.field_icon("reference", "NGameIcons|TENET_TYPE_ICON_PATH", ".dds");
            }
        }) {
            data.verify_icon("NGameIcons|TENET_TYPE_ICON_PATH", key, ".dds");
        }
        vd.field_bool("visible");
        vd.field_item("requires_dlc_flag", Item::DlcFeature);
        vd.field_validated_block("parameters", crate::ck3::data::doctrines::validate_parameters);
        vd.field_script_value_no_breakdown_rooted("piety_cost", Scopes::Rite);
        vd.field_trigger("is_shown", Tooltipped::No, &mut sc);
        vd.field_trigger("can_pick", Tooltipped::Yes, &mut sc);
        vd.field_trigger_rooted("can_pick_as_personal_tenet", Tooltipped::Yes, Scopes::Character);
        vd.field_numeric("divergence_multiplier");
        for field in ["character_modifier", "personal_tenet_modifier"] {
            vd.field_validated_block(field, |block, data| {
                let vd = Validator::new(block, data);
                validate_modifs(block, data, ModifKinds::Character, vd);
            });
        }
        vd.field_validated_block("county_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::County, vd);
        });
        vd.field_list("personal_tenet_parameters");
        vd.field_validated_block("traits", validate_traits);
        vd.field_validated_block("special_parameters", validate_special_parameters);
    }
}

pub fn validate_special_parameters(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for field in [
        "number_of_spouses",
        "number_of_consorts",
        "divine_marriage_opinion",
        "minimum_fervor",
        "bonus_heresy_protection",
    ] {
        vd.field_integer(field);
    }
    for field in [
        "fervor_per_holy_site",
        "bonus_fervor_gain",
        "divergence_protection",
        "heresy_threshold",
        "hostility_towards_us_mult",
        "piety_from_long_peace",
        "increased_dynasty_prestige_birth_mult",
        "increased_dynasty_prestige_marriage_mult",
        "increased_dynasty_prestige_long_reign_mult",
    ] {
        vd.field_numeric(field);
    }
    for field in ["unreformed", "warmonger"] {
        vd.field_bool(field);
    }
    vd.field_list_items("allowed_holding_types", Item::HoldingType);
    vd.field_choice("same_hof_hostility_override", &["righteous", "astray", "hostile", "evil"]);
    vd.field_validated_block("hostility_levels", |block, data| {
        let mut vd = Validator::new(block, data);
        for field in ["same_religion", "same_family", "unrelated"] {
            vd.field_choice(field, &["righteous", "astray", "hostile", "evil"]);
        }
    });
    vd.field_validated_block("hostility_level_override", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.unknown_value_fields(|key, value| {
            data.verify_exists(Item::Doctrine, key);
            if !["righteous", "astray", "hostile", "evil"].contains(&value.as_str()) {
                crate::report::err(crate::report::ErrorKey::Choice)
                    .msg("unknown hostility level")
                    .loc(value)
                    .push();
            }
        });
    });
    vd.field_validated_block("parameter_opinion_override", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.unknown_value_fields(|key, value| {
            data.verify_exists(Item::DoctrineParameter, key);
            value.expect_number();
        });
    });
}
