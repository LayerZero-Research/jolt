//! Temporary: time Akita's guided grouped planning against the full search
//! for every grouped row a planning input provisions.

use std::time::Instant;

use akita_config::{policy_of, CommitmentConfig};
use akita_params::{PolynomialGroupLayout, ScheduleLookupKey};
use akita_planner::emit::{GroupedGenerationRequest, PrecommittedProducer};
use akita_planner::{find_adapted_schedule, find_schedule};
use jolt_akita::configs::{
    JoltDenseBounded, JoltOneHotK16, JoltOneHotK16W2R2, JoltOneHotK16W4R2, JoltOneHotK16W8R2,
};
use jolt_akita::schedule_registry::dense_group_profile;
use jolt_akita::AkitaChunkProfile;
use jolt_akita_fuzz::artifacts;
use jolt_akita_fuzz::input::Reader;
use jolt_akita_fuzz::shape::{self, Shape};

pub fn run(path: &str, full: bool) -> Result<(), String> {
    jolt_akita_fuzz::env::init();
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    let shape = Shape::decode(&mut Reader::new(&data));
    let request = shape.setup_request().map_err(|f| f.to_string())?;
    println!("{shape}");
    match request.profile {
        AkitaChunkProfile::Single => go::<JoltOneHotK16>(&request, full),
        AkitaChunkProfile::Two => go::<JoltOneHotK16W2R2>(&request, full),
        AkitaChunkProfile::Four => go::<JoltOneHotK16W4R2>(&request, full),
        AkitaChunkProfile::Eight => go::<JoltOneHotK16W8R2>(&request, full),
    }
}

fn go<Cfg: CommitmentConfig>(request: &shape::SetupRequest, full: bool) -> Result<(), String> {
    let catalogs = artifacts::catalogs(request.one_hot_k, request.profile);
    let final_group =
        PolynomialGroupLayout::new(request.setup_shape.num_vars, request.setup_shape.num_polys);
    let producer = |vars: usize| {
        let profile =
            dense_group_profile(&catalogs.dense, PolynomialGroupLayout::new(vars, 1)).unwrap();
        PrecommittedProducer::try_new(profile, JoltDenseBounded::committed_source_contract().unwrap())
            .unwrap()
    };
    let mandatory: Vec<_> = request.program.iter().map(|p| producer(shape::arity(p))).collect();
    let u = request.untrusted.as_ref().map(|p| producer(shape::arity(p)));
    let t = request.trusted.as_ref().map(|p| producer(shape::arity(p)));
    let mut combos: Vec<(String, Vec<PrecommittedProducer>)> = Vec::new();
    for (name, advice) in [("U", vec![u]), ("T", vec![t]), ("U+T", vec![u, t])] {
        if advice.iter().all(Option::is_some) {
            let mut c: Vec<_> = advice.into_iter().flatten().collect();
            c.extend(mandatory.iter().copied());
            combos.push((format!("{name}+M{}", mandatory.len()), c));
        }
    }
    if !mandatory.is_empty() {
        combos.push((format!("M{}", mandatory.len()), mandatory.clone()));
    }
    let main_row = catalogs
        .one_hot
        .resolve_key(&ScheduleLookupKey::single(final_group))
        .map_err(|e| e.to_string())?;
    let contract = Cfg::committed_source_contract().unwrap();
    for (name, producers) in combos {
        let req = GroupedGenerationRequest::new(final_group, producers);
        let started = Instant::now();
        let adapted = find_adapted_schedule(
            main_row,
            &req,
            contract,
            &policy_of::<Cfg>(),
            Cfg::ring_challenge_config,
        );
        let adapted_s = started.elapsed().as_secs_f64();
        let mut line = format!(
            "{name:>8}: adapted {} in {adapted_s:.1}s",
            if adapted.is_ok() { "ok" } else { "ERR" }
        );
        if full {
            let started = Instant::now();
            let searched = find_schedule(
                &req.key(),
                contract,
                &req.source_contracts(),
                &policy_of::<Cfg>(),
                Cfg::ring_challenge_config,
            );
            let full_s = started.elapsed().as_secs_f64();
            let same = match (&adapted, &searched) {
                (Ok(a), Ok(b)) => format!("{}", a.schedule == b.schedule),
                _ => "n/a".into(),
            };
            line += &format!(
                "; full {} in {full_s:.1}s; identical={same}",
                if searched.is_ok() { "ok" } else { "ERR" }
            );
        }
        println!("{line}");
    }
    Ok(())
}
