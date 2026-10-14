use localclip_core::{
    domain::{Payload, Source},
    fixtures,
    store::{HistoryStore, Ingest, Options, SearchRequest, Snapshot},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{path::PathBuf, sync::Arc, time::Instant};

fn metrics(samples: &[f64]) -> Value {
    let mut sorted = samples.to_vec();
    sorted.sort_by(f64::total_cmp);
    json!({"samplesMs":samples,"n":samples.len(),"p95Ms":sorted[(samples.len()*95).div_ceil(100)-1],"meanMs":samples.iter().sum::<f64>()/samples.len() as f64,"maxMs":sorted.last()})
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("provide an empty benchmark output directory")?,
    );
    if output.exists() {
        return Err("benchmark output already exists; preserve previous measurements".into());
    }
    std::fs::create_dir_all(&output)?;
    let store = HistoryStore::open(
        output.join("history.sqlite"),
        Options {
            now: Arc::new(|| fixtures::NOW),
            available_space: Arc::new(|| Ok(u64::MAX)),
            ..Default::default()
        },
    )?;
    let ingest = |payload, index| {
        store.ingest(Snapshot {
            capture_id: format!("benchmark-{index}"),
            captured_at: fixtures::NOW + index as i64,
            payload,
            source: Source::default(),
            policy_revision: 0,
            sensitive: false,
        })
    };
    let mut digest = Sha256::new();
    let mut image_manifest = Vec::new();
    let mut text_bytes = 0;
    let mut image_bytes = 0;
    for i in 0..10100 {
        let payload = if i < 10000 {
            let text = fixtures::text(i);
            text_bytes += text.len();
            Payload::text(&text)
        } else {
            fixtures::image(i - 10000)?
        };
        for rep in &payload.items[0].representations {
            digest.update(rep.format.as_bytes());
            digest.update(&rep.data);
            if i >= 10000 {
                image_bytes += rep.data.len();
                image_manifest.push(json!({"index":i-10000,"format":rep.format,"bytes":rep.data.len(),"sha256":localclip_core::domain::hash(&rep.data),"width":1920,"height":1080}));
            }
        }
        if !matches!(ingest(payload, i)?, Ingest::Inserted(_)) {
            return Err("fixture insertion failed".into());
        }
        if i % 1000 == 0 {
            println!("seeded-count={i}");
        }
    }
    let manifest = json!({"generator":fixtures::VERSION,"seed":"0xC11F","texts":10000,"distribution":[9000,900,100],"textBytes":text_bytes,"images":image_manifest,"imageBytes":image_bytes,"datasetHash":format!("{:x}",digest.finalize()),"sqliteBytes":store.usage()?.database,"difference":"TIFF slots become Windows DIBV5; PNG/JPEG codec bytes differ from ImageIO"});
    std::fs::write(
        output.join("fixture-manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    let queries = [
        "需求",
        "代",
        "foo_bar",
        "50%",
        "a->b",
        "?a=1&b=2",
        "END-9999",
        "fixture-1234",
        "missing-literal",
        "The local",
    ];
    for query in queries {
        store.search(SearchRequest {
            literal: query.into(),
            ..Default::default()
        })?;
    }
    let (mut search, mut first, mut warm, mut writes, mut images) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for i in 0..60 {
        let request = SearchRequest {
            literal: queries[i % queries.len()].into(),
            ..Default::default()
        };
        let began = Instant::now();
        store.search(request.clone())?;
        search.push(began.elapsed().as_secs_f64() * 1000.0);
        let payload = Payload::text(&format!("new-generated-benchmark-text-{i}"));
        let began = Instant::now();
        let result = ingest(payload, 11000 + i)?;
        writes.push(began.elapsed().as_secs_f64() * 1000.0);
        if !matches!(result, Ingest::Inserted(_)) {
            return Err("text measurement did not insert".into());
        }
        let began = Instant::now();
        store.search(request.clone())?;
        first.push(began.elapsed().as_secs_f64() * 1000.0);
        let began = Instant::now();
        store.search(request)?;
        warm.push(began.elapsed().as_secs_f64() * 1000.0);
    }
    for i in 0..60 {
        let payload = fixtures::image(100 + (i * 13) % 100)?;
        let began = Instant::now();
        let result = ingest(payload, 12000 + i)?;
        images.push(began.elapsed().as_secs_f64() * 1000.0);
        if !matches!(result, Ingest::Inserted(_)) {
            return Err("image measurement did not insert".into());
        }
    }
    let first_metrics = metrics(&first);
    let warm_metrics = metrics(&warm);
    let gap =
        first_metrics["p95Ms"].as_f64().unwrap() / warm_metrics["p95Ms"].as_f64().unwrap() - 1.0;
    let report = json!({"layer":"SQLite service; excludes capture and UI","build":"Release","os":std::env::consts::OS,"arch":std::env::consts::ARCH,"method":"monotonic Instant; nearest rank ceil(0.95*n)","search":metrics(&search),"textIngest":metrics(&writes),"imageIngest":metrics(&images),"firstSearchAfterCommit":first_metrics,"warmSearch":warm_metrics,"firstSearchP95Gap":gap,"firstSearchTargetPassed":gap<0.2,"integrity":store.integrity_check()?,"fixture":manifest["datasetHash"]});
    std::fs::write(
        output.join("service-results.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!(
        "benchmark samples-per-series=60 search-p95-ms={} text-ingest-p95-ms={} image-ingest-p95-ms={} first-search-gap={gap:.4}",
        report["search"]["p95Ms"], report["textIngest"]["p95Ms"], report["imageIngest"]["p95Ms"]
    );
    drop(store);
    std::fs::remove_file(output.join("history.sqlite"))?;
    Ok(())
}
