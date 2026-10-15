use crate::block::Block;
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
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct DoctrineGroup {}
#[derive(Clone, Debug)]
pub struct Doctrine {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::DoctrineGroup, DoctrineGroup::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Doctrine, Doctrine::add)
}

impl DoctrineGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DoctrineGroup, key, block, Box::new(Self {}));
    }
}
impl Doctrine {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Doctrine, key, block, Box::new(Self {}));
    }
}

impl DbKind for DoctrineGroup {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca = format!("{key}_name");
        data.verify_exists_implied(Item::Localization, &loca, key);

        data.verify_icon("NGameIcons|DOCTRINE_GROUP_TYPE_ICON_PATH", key, ".dds");

        vd.field_value("category");
        vd.field_bool("main_rite");
        vd.field_choice("divergence", &["category", "sequence"]);
        vd.field_choice("doctrine_lock", &["none", "rite", "faith", "religion"]);
        // Like laws, doctrine indices are unique within the final loaded group.
        let mut doctrines: Vec<_> = data.database.iter_key_block(Item::Doctrine).collect();
        doctrines.sort_by_key(|(key, _)| key.loc);
        let mut indices = std::collections::BTreeMap::new();
        for (doctrine, definition) in doctrines {
            if definition.get_field_value("doctrine_group_type") != Some(key) {
                continue;
            }
            let token = definition.get_field_value("index");
            let value = token.map_or(Some(0), Token::get_integer);
            if let Some(value) = value {
                let location = token.unwrap_or(doctrine);
                if let Some(previous) = indices.insert(value, location) {
                    crate::report::err(crate::report::ErrorKey::Validation)
                        .msg(format!("duplicate doctrine index {value} in doctrine group {key}"))
                        .loc(location)
                        .loc_msg(previous, "another doctrine uses this index")
                        .push();
                }
            }
        }

        vd.field_integer("number_of_picks");
        vd.field_trigger_rooted("is_available_on_create", Tooltipped::No, Scopes::Faith);

        vd.field_list_items("doctrine_types", Item::Doctrine);
    }
}

impl DbKind for Doctrine {
    fn has_property(
        &self,
        _key: &Token,
        block: &Block,
        property: &str,
        _data: &Everything,
    ) -> bool {
        if property == "unreformed"
            && let Some(parameters) = block.get_field_block("special_parameters")
        {
            for (key, _) in parameters.iter_assignments() {
                if key.is("unreformed") {
                    return true;
                }
            }
        }
        false
    }

    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(block) = block.get_field_block("parameters") {
            for key in block.iter_values() {
                db.add_flag(Item::DoctrineParameter, key.clone());
                db.add_flag(Item::DoctrineBooleanParameter, key.clone());
            }
            for (key, value) in block.iter_assignments() {
                if value.is("yes") || value.is("no") {
                    db.add_flag(Item::DoctrineBooleanParameter, key.clone());
                }
                db.add_flag(Item::DoctrineParameter, key.clone());
            }
        }
        if let Some(parameters) = block.get_field_block("special_parameters") {
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
        vd.req_field("doctrine_group_type");
        vd.field_item("doctrine_group_type", Item::DoctrineGroup);
        vd.field_integer("index");
        vd.field_numeric("divergence");
        sc.define_list("selected_doctrines", Scopes::Doctrine, key);
        sc.define_name("actor", Scopes::Character, key);

        let icon = vd.field_value("icon").unwrap_or(key);
        data.verify_icon("NGameIcons|DOCTRINE_TYPE_ICON_PATH", icon, ".dds");

        if !vd.field_validated_sc("name", &mut sc, validate_desc) {
            let loca = format!("{key}_name");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        if !vd.field_validated_sc("desc", &mut sc, validate_desc) {
            let loca = format!("{key}_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        vd.field_bool("visible");
        vd.field_validated_sc("historical_glossary", &mut sc, validate_desc);
        vd.field_validated_block("parameters", validate_parameters);
        let mut cost_sc = ScopeContext::new(Scopes::Rite, key);
        vd.field_script_value("piety_cost", &mut cost_sc);
        vd.field_trigger("is_shown", Tooltipped::No, &mut sc);
        vd.field_trigger("can_pick", Tooltipped::Yes, &mut sc);

        vd.field_validated_block("character_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("name", Item::Localization);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        // Not documented, but used in vanilla
        vd.field_validated_block("clergy_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        // In the docs but not used in vanilla
        vd.field_validated_block("doctrine_character_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("doctrine", Item::Doctrine);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        vd.field_validated_block("traits", validate_traits);
        vd.field_validated_block(
            "special_parameters",
            crate::ck3::data::rites::validate_special_parameters,
        );
    }
}

pub fn validate_parameters(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    vd.values(); // Bare parameter flags are documented and used in 1.20.
    vd.unknown_value_fields(|_, value| {
        if value.is("yes") || value.is("no") {
            return;
        }
        value.expect_number();
    });
}
