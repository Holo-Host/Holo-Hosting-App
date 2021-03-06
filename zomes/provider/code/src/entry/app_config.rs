use boolinator::Boolinator;

use hdk::{
    self,
    entry_definition::ValidatingEntryType,
    holochain_core_types::{dna::entry_types::Sharing, validation::EntryValidationData},
    holochain_json_api::{error::JsonError, json::JsonString},
    holochain_persistence_api::hash::HashString,
};

#[derive(Serialize, Deserialize, Debug, Clone, DefaultJson)]
pub struct AppConfig {
    pub happ_hash: HashString,
}

pub fn definitions() -> ValidatingEntryType {
    entry!(
        name: "app_config",
        description: "config for an app",
        sharing: Sharing::Public,
        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },
        validation: |validation_data: hdk::EntryValidationData<AppConfig>| {
            match validation_data
            {
                EntryValidationData::Create{entry:_app_config,validation_data:_} =>
                {
                    Ok(())
                },
                EntryValidationData::Modify{new_entry,old_entry,old_entry_header:_,validation_data:_} =>
                {
                   (new_entry.happ_hash != old_entry.happ_hash)
                   .ok_or_else(|| String::from("Trying to modify with same data"))
                },
                EntryValidationData::Delete{old_entry:_,old_entry_header:_,validation_data:_} =>
                {
                  Ok(())
                }

            }

        },

        links: [
            from!(
                "%agent_id",
                link_type: "my_registered_apps_tag",

                validation_package: || {
                    hdk::ValidationPackageDefinition::Entry
                },

                validation: | _validation_data: hdk::LinkValidationData | {
                    Ok(())
                }
            ),
            to!(
                "%agent_id",
                link_type: "host_enabled",

                validation_package: || {
                    hdk::ValidationPackageDefinition::Entry
                },

                validation: | _validation_data: hdk::LinkValidationData | {
                    Ok(())
                }
            ),
            to!(
                "%agent_id",
                link_type: "recently_enabled_app_tag",

                validation_package: || {
                    hdk::ValidationPackageDefinition::Entry
                },

                validation: | _validation_data: hdk::LinkValidationData | {
                    Ok(())
                }
            ),
            to!(
                "%agent_id",
                link_type: "recently_disabled_app_tag",

                validation_package: || {
                    hdk::ValidationPackageDefinition::Entry
                },

                validation: | _validation_data: hdk::LinkValidationData | {
                    Ok(())
                }
            ),
            to!(
                "%agent_id",
                link_type: "need_updates_enabled_from_kv_store",

                validation_package: || {
                    hdk::ValidationPackageDefinition::Entry
                },

                validation: | _validation_data: hdk::LinkValidationData | {
                    Ok(())
                }
            ),
            to!(
                "%agent_id",
                link_type: "need_updates_disabled_from_kv_store",

                validation_package: || {
                    hdk::ValidationPackageDefinition::Entry
                },

                validation: | _validation_data: hdk::LinkValidationData | {
                    Ok(())
                }
            ),
            from!(
                "%agent_id",
                link_type: "apps_enabled",

                validation_package: || {
                    hdk::ValidationPackageDefinition::Entry
                },

                validation: | _validation_data: hdk::LinkValidationData | {
                    Ok(())
                }
            ),
            from!(
                "anchor",
                link_type: "all_apps_tag",

                validation_package: || {
                    hdk::ValidationPackageDefinition::Entry
                },

                validation: | _validation_data: hdk::LinkValidationData | {
                    Ok(())
                }
            )
        ]
    )
}
