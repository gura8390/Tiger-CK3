//! CK3 1.20 puppet types and actions, validated against native .info and definitions.
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_possibly_named_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct PuppetType;
#[derive(Clone, Debug)]
pub struct PuppetAction;
inventory::submit! { ItemLoader::Normal(GameFlags::Ck3, Item::PuppetType, PuppetType::add) }
inventory::submit! { ItemLoader::Normal(GameFlags::Ck3, Item::PuppetAction, PuppetAction::add) }
impl PuppetType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PuppetType, key, block, Box::new(Self));
    }
}
impl PuppetAction {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PuppetAction, key, block, Box::new(Self));
    }
}
impl DbKind for PuppetType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("puppet", Scopes::Character, key);
        vd.field_validated("color", validate_possibly_named_color);
        vd.field_item("interaction", Item::CharacterInteraction);
        vd.field_trigger("can_have", Tooltipped::No, &mut sc);
        vd.field_trigger("is_valid", Tooltipped::No, &mut sc);
        vd.field_script_value("priority", &mut sc);
        vd.field_list_items("actions", Item::PuppetAction);
        vd.multi_field_validated_block("asset", |b, data| {
            let mut vd = Validator::new(b, data);
            vd.field_trigger("trigger", Tooltipped::No, &mut sc);
            vd.field_item("icon", Item::File);
            vd.field_item("background", Item::File);
        });
        vd.multi_field_validated_block("animation", |b, data| {
            let mut vd = Validator::new(b, data);
            vd.field_trigger("trigger", Tooltipped::No, &mut sc);
            vd.field_item("reference", Item::PortraitAnimation);
        });
    }
}
impl DbKind for PuppetAction {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("puppet", Scopes::Character, key);
        // Native also uses "trigger", although the .info's list omits it.
        vd.field_choice("type", &["character_interaction", "decision", "great_project", "trigger"]);
        vd.field_item("interaction", Item::CharacterInteraction);
        vd.field_item("decision", Item::Decision);
        vd.field_item("great_project", Item::GreatProjectType);
        vd.field_trigger("is_enabled", Tooltipped::No, &mut sc);
        vd.field_bool("is_important");
    }
}
