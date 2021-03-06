use std::time::Duration;
use hdk::{
    error::{ZomeApiError, ZomeApiResult},
    holochain_core_types::{
        entry::Entry,
        error::HolochainError,
        link::LinkMatch
    },
    holochain_json_api::{
        error::JsonError,
        json::{JsonString, RawString},
    },
    holochain_persistence_api::{cas::content::Address, hash::HashString},
    holochain_wasm_utils::api_serialization::{
        get_entry::{GetEntryOptions, GetEntryResultType},
        get_links::GetLinksResult,
    },
    utils,
};

use crate::entry::host_doc::HostDoc;
use crate::entry::payment_pref::PaymentPref;

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct AppConfig {
    pub happ_hash: HashString,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct DnaToHost {
    recently_enabled_apps: Vec<App2Host>,
    recently_disabled_apps: Vec<App2Host>,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct App2Host {
    app: HashString,
    host: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct AllApps {
    hash: HashString,
    details: String,
}

pub fn validate_host() -> ZomeApiResult<bool> {
    let check = handle_is_registered_as_host()?;
    if check.addresses().len() != 0 {
        Ok(true)
    } else {
        Err(ZomeApiError::Internal("Agent Not a Host".to_string()))
    }
}

pub fn handle_get_all_apps() -> ZomeApiResult<Vec<AllApps>> {
    validate_host()?;
    let all_apps = Entry::App("anchor".into(), RawString::from("ALL_APPS").into());
    let anchor_address = hdk::commit_entry(&all_apps)?;
    let all_apps_commit = hdk::get_links(
        &anchor_address,
        LinkMatch::Exactly("all_apps_tag"),
        LinkMatch::Any,
    )?;
    let app_address = all_apps_commit.addresses();

    let mut app_details_list: Vec<AllApps> = Vec::new();
    for x in app_address {
        let details = hdk::call(
            hdk::THIS_INSTANCE,
            "provider",
            Address::from(hdk::PUBLIC_TOKEN.to_string()),
            "get_app_details",
            json!({ "app_hash": x }).into(),
        )?;
        app_details_list.push(AllApps {
            hash: x.to_owned(),
            details: String::from(details.to_owned()),
        });
    }
    Ok(app_details_list)
}

pub fn is_enabled(app_hash: &HashString) -> ZomeApiResult<bool>
{
    validate_host()?;
    let links = hdk::get_links(&hdk::AGENT_ADDRESS,
        LinkMatch::Exactly("apps_enabled"),
        LinkMatch::Any)?;

    let result = links.addresses().into_iter().find(|x| x == app_hash);
    match result {
        Some(_) => Ok(true),
        None => Ok(false)
    }
}

fn check_link_exists(tag:String, app_hash: &HashString, from: Option<&HashString>) -> ZomeApiResult<bool>
{
    let links;
    validate_host()?;
    match from {
        Some ( from ) => {
            links = hdk::get_links(from,
                LinkMatch::Exactly(&tag),
                LinkMatch::Any)?;
        }
        None => {
            links = hdk::get_links(&hdk::AGENT_ADDRESS,
                LinkMatch::Exactly(&tag),
                LinkMatch::Any)?;
        }
    }

    let result = links.addresses().into_iter().find(|x| x == app_hash);
    match result {
        Some(_) => Ok(true),
        None => Ok(false)
    }
}

pub fn handle_enable_app(app_hash: HashString) -> ZomeApiResult<()> {
    if is_enabled(&app_hash)? == false {
        utils::link_entries_bidir(
            &app_hash,
            &hdk::AGENT_ADDRESS,
            "host_enabled",
            "apps_enabled",
            "",
            "",
        )?;

        hdk::link_entries(
            &app_hash,
            &hdk::AGENT_ADDRESS,
            "recently_enabled_app_tag",
            "",
        )?;

        // check if its a recently_disabled_app_tag
        if check_link_exists("recently_disabled_app_tag".to_string(), &app_hash, None)? == true {
            hdk::remove_link(
                &app_hash,
                &hdk::AGENT_ADDRESS,
                "recently_disabled_app_tag",
                "",
            )?;
        }

        // check if its a recently_disabled_app_tag
        if check_link_exists("need_updates_disabled_from_kv_store".to_string(), &app_hash, None)? == true {
            hdk::remove_link(
                &app_hash,
                &hdk::AGENT_ADDRESS,
                "need_updates_disabled_from_kv_store",
                "",
            )?;
        }
        // The sleep is because we need to wait for remove to propogate
        hdk::sleep(Duration::from_millis(100))?;


    }
    Ok(())
}

pub fn handle_disable_app(app_hash: HashString) -> ZomeApiResult<()> {

    if is_enabled(&app_hash)? == true {

        if check_link_exists("host_enabled".to_string(), &app_hash, None)? == true {
            hdk::remove_link(
                &app_hash,
                &hdk::AGENT_ADDRESS,
                "host_enabled",
                &"".to_owned(),
            )?;
        }
        if check_link_exists("host_enabled".to_string(), &hdk::AGENT_ADDRESS, Some(&app_hash))? == true {
            hdk::remove_link(&hdk::AGENT_ADDRESS, &app_hash, "apps_enabled", "")?;
        }
        // check if its a recently_disabled_app_tag
        if check_link_exists("recently_enabled_app_tag".to_string(), &app_hash, None)? == true {
            hdk::remove_link(
                &app_hash,
                &hdk::AGENT_ADDRESS,
                "recently_enabled_app_tag",
                "",
            )?;
        }
        // check if its a recently_disabled_app_tag
        if check_link_exists("need_updates_enabled_from_kv_store".to_string(), &app_hash, None)? == true {
            hdk::remove_link(
                &app_hash,
                &hdk::AGENT_ADDRESS,
                "need_updates_enabled_from_kv_store",
                "",
            )?;
        }

        // The sleep is because we need to wait for remove to propogate
        // hdk::sleep(Duration::from_millis(100))?;

        hdk::link_entries(
            &app_hash,
            &hdk::AGENT_ADDRESS,
            "recently_disabled_app_tag",
            "",
        )?;
    }
    Ok(())
}

fn handle_get_all_apps_addresses() -> ZomeApiResult<GetLinksResult> {
    validate_host()?;
    let all_apps = Entry::App("anchor".into(), RawString::from("ALL_APPS").into());
    let anchor_address = hdk::commit_entry(&all_apps)?;

    hdk::get_links(
        &anchor_address,
        LinkMatch::Exactly("all_apps_tag"),
        LinkMatch::Any,
    )
}

pub fn handle_get_kv_updates_dna_to_host() -> ZomeApiResult<DnaToHost> {
    // Get all the apps
    let got_apps: GetLinksResult = handle_get_all_apps_addresses()?;
    let all_apps = got_apps.addresses().to_vec();
    // Check the enabled tag
    let mut recently_enabled_apps: Vec<App2Host> = Vec::new();
    for app in all_apps.clone() {
        let app_copy = app.clone();
        let mut enabled_agents: Vec<ZomeApiResult<Entry>> = hdk::get_links_and_load(
            &app_copy,
            LinkMatch::Exactly("recently_enabled_app_tag"),
            LinkMatch::Any,
        )?;
        let enabled_agents_old: Vec<ZomeApiResult<Entry>> = hdk::get_links_and_load(
            &app_copy,
            LinkMatch::Exactly("need_updates_enabled_from_kv_store"),
            LinkMatch::Any,
        )?;

        enabled_agents.append(&mut enabled_agents_old.to_owned());

        let mut agent_address_list: Vec<String> = Vec::new();
        for a in enabled_agents {
            match a? {
                Entry::AgentId(a) => agent_address_list.push(a.pub_sign_key),
                _ => {}
            }
        }
        recently_enabled_apps.push(App2Host {
            app,
            host: agent_address_list.clone(),
        });

        // Remove the enable tag and add intransition apps
        for agent in agent_address_list {
            if check_link_exists("recently_enabled_app_tag".to_string(), &HashString::from(agent.clone()), Some(&app_copy))? == true {
                hdk::remove_link(
                        &app_copy,
                        &HashString::from(agent.clone()),
                        "recently_enabled_app_tag",
                        "",
                )?;
            }
            // The sleep is because we need to wait for remove to propogate
            // hdk::sleep(Duration::from_millis(100))?;
            // This check because as of hc v0.0.32-lapha2 hdk::link_entries adds a new link even if a link exists between two entire
            if check_agent_exist(enabled_agents_old.to_owned(), agent.to_owned().to_string())? == false {
                hdk::link_entries(
                    &app_copy,
                    &HashString::from(agent.clone()),
                    "need_updates_enabled_from_kv_store",
                    "",
                )?;
            }
        }
    }

    // Check the disabled tag
    let mut recently_disabled_apps: Vec<App2Host> = Vec::new();
    for app in all_apps.clone() {
        let app_copy = app.clone();
        let mut disabled_agents: Vec<ZomeApiResult<Entry>> = hdk::get_links_and_load(
            &app_copy,
            LinkMatch::Exactly("recently_disabled_app_tag"),
            LinkMatch::Any,
        )?;

        let disabled_agents_old: Vec<ZomeApiResult<Entry>> = hdk::get_links_and_load(
            &app_copy,
            LinkMatch::Exactly("need_updates_disabled_from_kv_store"),
            LinkMatch::Any,
        )?;

        disabled_agents.append(&mut disabled_agents_old.to_owned());

        // Data Refactored
        let mut agent_address_list: Vec<String> = Vec::new();
        for a in disabled_agents {
            match a? {
                Entry::AgentId(a) => agent_address_list.push(a.pub_sign_key),
                _ => {}
            }
        }
        recently_disabled_apps.push(App2Host {
            app,
            host: agent_address_list.clone(),
        });
        // Remove the disabled tag and add intransition apps
        for agent in agent_address_list {
            if check_link_exists("recently_disabled_app_tag".to_string(), &HashString::from(agent.clone()), Some(&app_copy))? == true {
                hdk::remove_link(
                    &app_copy,
                    &HashString::from(agent.clone()),
                    "recently_disabled_app_tag",
                    "",
                )?;
            }
            // The sleep is because we need to wait for remove to propogate
            // hdk::sleep(Duration::from_millis(100))?;
            // This check because as of hc v0.0.32-lapha2 hdk::link_entries adds a new link even if a link exists between two entire
            if check_agent_exist(disabled_agents_old.to_owned(), agent.to_owned().to_string())? == false {
                hdk::link_entries(
                    &app_copy,
                    &HashString::from(agent.clone()),
                    "need_updates_disabled_from_kv_store",
                    "",
                )?;
            }

        }
    }
    Ok(DnaToHost {
        recently_enabled_apps,
        recently_disabled_apps,
    })
}

fn check_agent_exist(agent_list: Vec<ZomeApiResult<Entry>>, agent:String) -> Result<bool, HolochainError> {
    let mut flag :bool = false;
    for a in agent_list {
        match a? {
            Entry::AgentId(a) => {
                if a.pub_sign_key == agent.to_owned() {
                    flag = true;
                    break;
                }
            },
            _ => {}
        }
    }
    return Ok(flag)
}

pub fn handle_kv_updates_host_completed(kv_bundle: Vec<App2Host>) -> ZomeApiResult<()> {
    for kv in kv_bundle {
        for host_address in kv.host {
            if check_link_exists("need_updates_enabled_from_kv_store".to_string(), &HashString::from(host_address.clone()), Some(&kv.app))? == true {
                hdk::remove_link(
                    &kv.app,
                    &HashString::from(host_address.clone()),
                    "need_updates_enabled_from_kv_store",
                    "",
                )?;
            }
            if check_link_exists("need_updates_disabled_from_kv_store".to_string(), &HashString::from(host_address.clone()), Some(&kv.app))? == true {
                hdk::remove_link(
                    &kv.app,
                    &HashString::from(host_address.clone()),
                    "need_updates_disabled_from_kv_store",
                    "",
                )?;
            }
            // The sleep is because we need to wait for remove to propogate
            // hdk::sleep(Duration::from_millis(100))?;
        }
    }
    Ok(())
}
pub fn handle_get_enabled_app_list() -> ZomeApiResult<Vec<hc_common::GetLinksLoadElement<AppConfig>>>
{
    validate_host()?;
    hc_common::get_links_and_load_type(&hdk::AGENT_ADDRESS, "apps_enabled".to_string())
}

pub fn handle_get_host_for_app(app_hash: Address) -> ZomeApiResult<Vec<ZomeApiResult<Entry>>> {
    hdk::get_links_and_load(
        &app_hash,
        LinkMatch::Exactly("host_enabled"),
        LinkMatch::Any,
    )
}

pub fn handle_register_as_host(host_doc: HostDoc) -> ZomeApiResult<Address> {
    // TODO : Validation
    let verified_entry = Entry::App("host_doc".into(), host_doc.into());
    utils::commit_and_link(
        &verified_entry,
        &hdk::AGENT_ADDRESS,
        "verified_host_tag",
        "",
    )
    // Ok(address)
}

pub fn handle_is_registered_as_host() -> ZomeApiResult<GetLinksResult> {
    hdk::get_links(
        &hdk::AGENT_ADDRESS,
        LinkMatch::Exactly("verified_host_tag"),
        LinkMatch::Any,
    )
}

/*************************/
/* Service Log Functions */
/*************************/

#[derive(Serialize, Deserialize, Debug, Clone, DefaultJson)]
pub struct HoloFuelAc {
    pub account_number: String,
}
pub fn handle_add_service_log_details(
    app_hash: Address,
    max_fuel_per_invoice: f64,
    max_unpaid_value: f64,
    price_per_unit: f64,
) -> ZomeApiResult<Address> {
    if let GetEntryResultType::Single(result) = hdk::get_entry_result(
        &app_hash,
        GetEntryOptions {
            headers: true,
            ..Default::default()
        },
    )?
    .result
    {
        let provider_address = result.headers[result.headers.len() - 1].provenances()[0].source();

        let provider_hf: Vec<HoloFuelAc> = utils::get_links_and_load_type(
            &provider_address,
            LinkMatch::Exactly("holofuel_account_details_tag"),
            LinkMatch::Any,
        )?;

        add_service_log_details(
            PaymentPref {
                provider_address: Address::from(provider_hf[0].account_number.to_owned()),
                dna_bundle_hash: app_hash.clone(),
                max_fuel_per_invoice,
                max_unpaid_value,
                price_per_unit,
            },
            app_hash,
        )
    } else {
        Err(ZomeApiError::Internal(
            "Providers HoloFuel Ac is not Registered".to_string(),
        ))
    }
}

fn add_service_log_details(payment_pref: PaymentPref, app_hash: Address) -> ZomeApiResult<Address> {
    let payment_pref_entry = Entry::App("payment_pref".into(), payment_pref.into());
    utils::commit_and_link(&payment_pref_entry, &app_hash, "payment_pref_tag", "")
}

pub fn handle_get_service_log_details(app_hash: Address) -> ZomeApiResult<PaymentPref> {
    let payment_details: Vec<PaymentPref> = utils::get_links_and_load_type(
        &app_hash,
        LinkMatch::Exactly("payment_pref_tag"),
        LinkMatch::Any,
    )?;
    // let payment_pref:PaymentPref = PaymentPref::try_from(payment_details[0].entry.to_owned());
    Ok(payment_details[0].to_owned())
}
