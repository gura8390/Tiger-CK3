use crate::block::{BV, Block};
use crate::ck3::tables::misc::CUSTOM_RELIGION_LOCAS;
use crate::ck3::validate::validate_traits;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::fileset::FileKind;
use crate::game::GameFlags;
use crate::helpers::TigerHashMap;
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, err, warn};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_possibly_named_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Religion {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Religion, Religion::add)
}

impl Religion {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Religion, key, block, Box::new(Self {}));
    }
}

impl DbKind for Religion {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(details) = block.get_field_block("religion_details") {
            if let Some(token) = details.get_field_value("graphical_faith") {
                db.add_flag(Item::GraphicalFaith, token.clone());
            }
        }
        if let Some(icons) = block.get_field_block("custom_faith_icons") {
            for token in icons.iter_values() {
                db.add_flag(Item::FaithIcon, token.clone());
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        validate_religion_locas(key, data);
        let mut vd = Validator::new(block, data);
        vd.req_field("religion_details");
        vd.field_validated_block("religion_details", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("family");
            vd.field_item("family", Item::ReligionFamily);
            validate_details(&mut vd, data);
        });
        vd.field_item("main_holy_site", Item::HolySite);
        for field in
            ["eminent_holy_sites_max", "holy_sites_max", "eminent_holy_sites_min", "holy_sites_min"]
        {
            vd.field_integer(field);
        }
        vd.field_bool("pagan_roots");
        validate_doctrines("religion", data, &mut vd);
        vd.field_validated_block("traits", validate_traits);
        vd.field_validated_list("custom_faith_icons", |icon, data| {
            data.verify_icon("NGameIcons|FAITH_ICON_PATH", icon, ".dds");
        });
        validate_common(&mut vd);
        vd.field_list_items("holy_order_maa", Item::MenAtArms);
    }

    fn has_property(
        &self,
        _key: &Token,
        block: &Block,
        property: &str,
        _data: &Everything,
    ) -> bool {
        if let Some(block) = block.get_field_block("localization") {
            block.has_key(property)
        } else {
            false
        }
    }
}

fn validate_doctrines(iname: &str, data: &Everything, vd: &mut Validator) {
    // TODO: maybe cache doctrine_types and number_of_picks,
    // though the cache might only make sense if done globally instead of per religion/faith.
    let mut groups: TigerHashMap<&str, Vec<Token>> = TigerHashMap::default();
    vd.multi_field_validated_value("doctrine", |_, mut vd| {
        vd.item(Item::Doctrine);
        let doctrine = vd.value();
        for (group, block) in data.database.iter_key_block(Item::DoctrineGroup) {
            if let Some(doctrines) = block.get_field_block("doctrine_types") {
                if !doctrines.iter_values().any(|t| t == doctrine) {
                    continue;
                }
            } else {
                continue;
            }

            if let Some(seen) = groups.get_mut(group.as_str()) {
                let picks_token = block.get_field_value("number_of_picks");
                let picks = picks_token.and_then(Token::get_integer).unwrap_or(1);
                #[allow(clippy::cast_possible_wrap)]
                if let Some(other_doctrine) = seen.iter().find(|&d| d == doctrine) {
                    let msg = format!("duplicate doctrine {doctrine}");
                    warn(ErrorKey::DuplicateField)
                        .msg(msg)
                        .loc(doctrine)
                        .loc_msg(other_doctrine, "earlier doctrine")
                        .push();
                } else if picks == 1 {
                    // SAFETY: we never push empty vecs into this hash
                    let other_doctrine = &seen[0];
                    let msg = format!("{doctrine} and {other_doctrine} are both from {group}");
                    let info = format!("{group} only allows 1 pick");
                    err(ErrorKey::Conflict)
                        .msg(msg)
                        .info(info)
                        .loc(doctrine)
                        .loc_msg(other_doctrine, "earlier doctrine")
                        .push();
                } else if picks == (seen.len() as i64) {
                    let msg = format!("{iname} has more than {picks} doctrines from {group}");
                    // SAFETY: picks_token can be unwrapped because Some(Token) is the only
                    // way to get picks > 1
                    err(ErrorKey::Conflict)
                        .msg(msg)
                        .loc(doctrine)
                        .loc_msg(picks_token.unwrap(), "picks")
                        .push();
                }
                seen.push(doctrine.clone());
            } else {
                groups.insert(group.as_str(), vec![doctrine.clone()]);
            }
        }
    });

    vd.multi_field_validated_block("doctrine_selection_pair", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_item("requires_dlc_flag", Item::DlcFeature);
        vd.field_item("doctrine", Item::Doctrine);
        vd.field_item("fallback_doctrine", Item::Doctrine);
    });
}

pub(crate) fn validate_localization(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for field in CUSTOM_RELIGION_LOCAS {
        vd.field_validated(field, |bv, data| match bv {
            BV::Value(token) => data.verify_exists(Item::Localization, token),
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                for token in vd.values() {
                    data.verify_exists(Item::Localization, token);
                }
            }
        });
    }
}

fn validate_holy_order_names(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for block in vd.blocks() {
        let mut vd = Validator::new(block, data);
        vd.req_field("name");
        vd.field_item("name", Item::Localization);
        vd.field_item("coat_of_arms", Item::Coa);
    }
}

#[derive(Clone, Debug)]
pub struct Faith {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Faith, Faith::add)
}
impl Faith {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Faith, key, block, Box::new(Self {}));
    }
}
impl DbKind for Faith {
    fn add_subitems(&self, key: &Token, block: &Block, db: &mut Db) {
        if let Some(details) = block.get_field_block("faith_details") {
            for field in ["icon", "reformed_icon"] {
                if let Some(token) = details.get_field_value(field) {
                    db.add_flag(Item::FaithIcon, token.clone());
                }
            }
            if !details.has_key("icon") {
                db.add_flag(Item::FaithIcon, key.clone());
            }
            if let Some(token) = details.get_field_value("graphical_faith") {
                db.add_flag(Item::GraphicalFaith, token.clone());
            }
        }
    }
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        validate_religion_locas(key, data);
        let mut vd = Validator::new(block, data);
        vd.req_field("faith_details");
        vd.field_validated_block("faith_details", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("religion");
            vd.field_item("religion", Item::Religion);
            vd.req_field("color");
            vd.field_validated("color", validate_possibly_named_color);
            let icon = vd.field_value("icon").unwrap_or(key);
            data.verify_icon("NGameIcons|FAITH_ICON_PATH", icon, ".dds");
            vd.field_icon("reformed_icon", "NGameIcons|FAITH_ICON_PATH", ".dds");
            vd.field_item("religious_head", Item::Title);
            vd.field_item("head_of_rite", Item::Title);
            validate_details(&mut vd, data);
        });
        vd.field_item("origin", Item::Faith);
        vd.field_list_items("holy_sites", Item::HolySite);
        vd.field_list_items("eminent_holy_sites", Item::HolySite);
        if let Some(sites) = block.get_field_list("holy_sites") {
            if let Some(eminent) = block.get_field_list("eminent_holy_sites") {
                for site in sites {
                    if eminent.contains(&site) {
                        err(ErrorKey::Conflict)
                            .msg("holy site is both regular and eminent")
                            .loc(site)
                            .push();
                    }
                }
            }
        }
        vd.field_list_items("tenets", Item::Tenet);
        validate_doctrine_list("faith", block, data, &mut vd);
        validate_tenet_selection(&mut vd);
        vd.field_list_items("cultures", Item::Culture);
        vd.field_bool("historical");
        vd.field_item("main_rite", Item::Rite);
        validate_common(&mut vd);
        // Keep inherited custom localization validation, using the new parent field.
        if let Some(details) = block.get_field_block("faith_details") {
            if let Some(religion) = details.get_field_value("religion") {
                for loca in CUSTOM_RELIGION_LOCAS {
                    if block.get_field_block("localization").is_some_and(|b| b.has_key(loca)) {
                        continue;
                    }
                    if data.item_has_property(Item::Religion, religion.as_str(), loca) {
                        continue;
                    }
                    warn(ErrorKey::MissingLocalization)
                        .msg(format!("faith or religion missing localization for {loca}"))
                        .loc(key)
                        .push();
                }
            }
        }
    }
    fn has_property(
        &self,
        key: &Token,
        _block: &Block,
        property: &str,
        _data: &Everything,
    ) -> bool {
        property == "is_modded" && key.loc.kind == FileKind::Mod
    }
}

fn validate_religion_locas(key: &Token, data: &Everything) {
    data.verify_exists(Item::Localization, key);
    for suffix in ["adj", "adherent", "adherent_plural", "desc"] {
        data.verify_exists_implied(Item::Localization, &format!("{key}_{suffix}"), key);
    }
}
fn validate_details(vd: &mut Validator, data: &Everything) {
    vd.field_value("graphical_faith");
    validate_piety_icon_group(vd.field_value("piety_icon_group"), data);
    for field in [
        "tenet_background_icon",
        "tenet_heretical_background_icon",
        "tenet_neutral_background_icon",
        "tenet_unknown_background_icon",
    ] {
        vd.field_icon(field, "NGameIcons|FAITH_DOCTRINE_BACKGROUND_PATH", "");
    }
    vd.field_item("theocracy_government_type", Item::GovernmentType);
}
fn validate_common(vd: &mut Validator) {
    vd.field_list("reserved_male_names");
    vd.field_list("reserved_female_names");
    vd.field_validated_block("localization", validate_localization);
    vd.field_validated_block("holy_order_names", validate_holy_order_names);
}
pub fn validate_tenet_selection(vd: &mut Validator) {
    vd.multi_field_validated_block("tenet_selection_pair", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.req_field("requires_dlc_flag");
        vd.req_field("tenet");
        vd.field_item("requires_dlc_flag", Item::DlcFeature);
        vd.field_item("tenet", Item::Tenet);
        vd.field_item("fallback_tenet", Item::Tenet);
    });
}
pub fn validate_doctrine_list(iname: &str, block: &Block, data: &Everything, vd: &mut Validator) {
    vd.field_list_items("doctrines", Item::Doctrine);
    if let Some(doctrines) = block.get_field_list("doctrines") {
        let mut groups: TigerHashMap<String, Vec<Token>> = TigerHashMap::default();
        let mut seen = std::collections::HashSet::new();
        for doctrine in doctrines {
            if !seen.insert(doctrine.as_str()) {
                warn(ErrorKey::DuplicateField)
                    .msg(format!("duplicate doctrine {doctrine}"))
                    .loc(&doctrine)
                    .push();
                continue;
            }
            // CK3 1.20 stores group membership on each doctrine definition.
            // Legacy groups can still specify a doctrine_types list.
            let new_group = data
                .database
                .get_key_block(Item::Doctrine, doctrine.as_str())
                .and_then(|(_, b)| b.get_field_value("doctrine_group_type"));
            for (group, definition) in data.database.iter_key_block(Item::DoctrineGroup) {
                if new_group.is_some_and(|g| g == group)
                    || definition
                        .get_field_list("doctrine_types")
                        .is_some_and(|v| v.contains(&doctrine))
                {
                    let entries = groups.entry(group.to_string()).or_default();
                    // Do not invent a default pick limit for 1.20 groups, whose .info
                    // no longer declares number_of_picks. Explicit limits remain checked.
                    if let Some(picks) =
                        definition.get_field_value("number_of_picks").and_then(Token::get_integer)
                        && entries.len() as i64 >= picks
                    {
                        err(ErrorKey::Conflict)
                            .msg(format!("{iname} has more than {picks} doctrines from {group}"))
                            .loc(&doctrine)
                            .push();
                    }
                    entries.push(doctrine.clone());
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ReligionFamily {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::ReligionFamily, ReligionFamily::add)
}

impl ReligionFamily {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ReligionFamily, key, block, Box::new(Self {}));
    }
}

impl DbKind for ReligionFamily {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(token) = block.get_field_value("graphical_faith") {
            db.add_flag(Item::GraphicalFaith, token.clone());
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        // let modif = format!("{key}_opinion");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);

        let name = vd.field_value("name").unwrap_or(key);
        data.verify_exists(Item::Localization, name);
        let loca = format!("{name}_desc");
        data.verify_exists_implied(Item::Localization, &loca, name);

        vd.field_bool("is_pagan");
        vd.field_value("graphical_faith");
        validate_piety_icon_group(vd.field_value("piety_icon_group"), data);
        for field in [
            "tenet_background_icon",
            "tenet_heretical_background_icon",
            "tenet_neutral_background_icon",
            "tenet_unknown_background_icon",
        ] {
            vd.field_icon(field, "NGameIcons|FAITH_DOCTRINE_BACKGROUND_PATH", "");
        }
        vd.field_item("hostility_doctrine", Item::Doctrine);
    }
}

#[derive(Clone, Debug)]
pub struct FervorModifier {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::FervorModifier, FervorModifier::add)
}

impl FervorModifier {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::FervorModifier, key, block, Box::new(Self {}));
    }
}

impl DbKind for FervorModifier {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Faith, key);
        vd.field_script_value("value", &mut sc);
        vd.field_trigger("trigger", Tooltipped::No, &mut sc);
    }
}

fn validate_piety_icon_group(group: Option<&Token>, data: &Everything) {
    if let Some(group) = group {
        if let Some(valid) = data.get_defined_array_warn(group, "NGameIcons|PIETY_GROUPS") {
            for valid_group in valid.iter_values() {
                if group == valid_group {
                    return;
                }
            }
        }
        let msg = "piety icon group not listed in PIETY_GROUPS define";
        warn(ErrorKey::Choice).msg(msg).loc(group).push();
    }
}
