use crate::block::{BV, Block};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, warn};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Election {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::SuccessionElection, Election::add)
}

impl Election {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::SuccessionElection, key, block, Box::new(Self {}));
    }
}

impl DbKind for Election {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(candidates) = block.get_field_block("candidates")
            && let Some(sets) = candidates.get_field_block("character_sets")
        {
            for (key, _) in sets.iter_definitions() {
                db.add_flag(Item::ElectionCandidateSet, key.clone());
            }
        }
    }
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let loca = format!("{key}_candidates");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_candidates_tooltip");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_electors");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_electors_tooltip");
        data.verify_exists_implied(Item::Localization, &loca, key);

        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("title", Scopes::LandedTitle, key);
        sc.define_name("holder", Scopes::Character, key);

        vd.field_validated_block("candidates", |block, data| {
            let mut vd = Validator::new(block, data);
            validate_candidates(&mut vd, &mut sc);
        });

        vd.field_validated_block("electors", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_integer("max");
            vd.field_validated_block_sc("priority", &mut sc, validate_modifiers_with_base);
            validate_candidates(&mut vd, &mut sc);
        });

        vd.field_validated_block_sc("elector_vote_strength", &mut sc, validate_modifiers_with_base);

        vd.field_script_value("elector_vote_strength_value", &mut sc);

        sc.define_name("candidate", Scopes::Character, key);
        sc.define_name("holder_candidate", Scopes::Character, key);
        vd.field_validated_block_sc("candidate_score", &mut sc, validate_modifiers_with_base);
        vd.field_script_value("candidate_score_value", &mut sc);
    }
}

fn validate_candidates(vd: &mut Validator, sc: &mut ScopeContext) {
    vd.multi_field_validated("add", |bv, data| match bv {
        BV::Value(token) => {
            if !CANDIDATE_TYPES.contains(&token.as_str()) {
                let msg = "unknown candidate category";
                warn(ErrorKey::Choice).msg(msg).loc(token).push();
            }
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.field_choice("type", CANDIDATE_TYPES);
            vd.field_trigger("limit", Tooltipped::No, sc);
        }
    });
    vd.field_validated_block("character_sets", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.unknown_block_fields(|_key, block| {
            let mut vd = Validator::new(block, data);
            vd.req_field("type");
            vd.field_choice("type", CANDIDATE_TYPES);
            vd.field_trigger("limit", Tooltipped::No, sc);
            vd.field_script_value("priority_value", sc);
        });
    });
    vd.field_choice(
        "candidate_policy",
        &["include_all_candidates", "only_best_candidate", "one_candidate_per_set"],
    );
    vd.field_script_value("priority_value", sc);
    vd.field_trigger("limit", Tooltipped::No, sc);
}

const CANDIDATE_TYPES: &[&str] = &[
    "title_claimants",
    "title_dejure_vassals",
    "holder",
    "holder_direct_vassals",
    "holder_spouses",
    "holder_close_family",
    "holder_close_or_extended_family",
    "holder_dynasty",
    "holder_council_members",
    "holder_tributaries",
    "same_faith_clerical_electors",
];
