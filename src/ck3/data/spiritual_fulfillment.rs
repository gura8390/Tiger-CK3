//! CK3 1.20 spiritual fulfillment types, parameters, and bounded thresholds.
use crate::block::Block;
use crate::ck3::modif::ModifKinds;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::validate_modifs;
use crate::report::{ErrorKey, err};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct SpiritualFulfillmentType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::SpiritualFulfillmentType, SpiritualFulfillmentType::add)
}
impl SpiritualFulfillmentType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::SpiritualFulfillmentType, key, block, Box::new(Self {}));
    }
}
impl DbKind for SpiritualFulfillmentType {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        for level in block.get_field_blocks("level") {
            if let Some(flags) = level.get_field_block("flags") {
                for flag in flags.iter_values() {
                    db.add_flag(Item::FulfillmentParameter, flag.clone());
                }
            }
        }
    }
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_list_items("religions", Item::Religion);
        let min = data
            .get_defined_string("NReligion|MIN_SPIRITUAL_FULFILLMENT")
            .and_then(Token::get_fixed_number);
        let max = data
            .get_defined_string("NReligion|MAX_SPIRITUAL_FULFILLMENT")
            .and_then(Token::get_fixed_number);
        let mut thresholds = std::collections::BTreeMap::new();
        vd.multi_field_validated_block("level", |level, data| {
            let mut vd = Validator::new(level, data);
            if let (Some(min), Some(max)) = (min, max) {
                vd.field_numeric_range("threshold", min as f64 / 100_000.0..=max as f64 / 100_000.0);
            } else {
                vd.field_numeric("threshold");
            }
            let token = level.get_field_value("threshold");
            let value = token.map_or(Some(0), Token::get_fixed_number);
            if let Some(value) = value {
                let location = token.unwrap_or(key);
                if let Some(previous) = thresholds.insert(value, location.clone()) {
                    err(ErrorKey::Validation)
                        .msg("duplicate spiritual fulfillment threshold (omitted threshold defaults to 0)")
                        .loc(location).loc_msg(&previous, "another level uses this threshold").push();
                }
            }
            vd.field_validated_block("modifier", |block, data| {
                let vd = Validator::new(block, data);
                validate_modifs(block, data, ModifKinds::Character, vd);
            });
            vd.field_item("icon", Item::File);
            vd.field_list("flags");
        });
    }
}
