use std::{sync::Arc, time::{SystemTime, UNIX_EPOCH, Duration}, collections::HashMap};
use tokio::sync::{mpsc, mpsc::{Sender, Receiver, WeakSender}};

use temporal_sdk_core::api::telemetry::metrics::{CoreMeter, MetricsAttributesOptions, MetricAttributes, Counter, Gauge, Histogram};

pub(super) type MetricsExporterConsumer = Receiver<MetricEventEntry>;
pub(super) type MetricsExporterProducer = Sender<MetricEventEntry>;

#[derive(Debug)]
pub enum MetricEventEntryChange {
    Delta(u64),
    Value(u64),
}

/// A single metric change event
#[derive(Debug)]
pub struct MetricEventEntry {
    /// The time at which this metric event was generated (not when it was exported to lang)
    pub timestamp: SystemTime,

    pub metric_name: String,
    pub change: MetricEventEntryChange,

    /// Arbitrary k/v pairs (span k/vs are collapsed with event k/vs here).
    pub attributes: HashMap<String, serde_json::Value>,
}

// FIXME: Determine if can get away a single TypeScriptMetricsExporterMetric struct, rather than distinct structs for Counter + Histogram + Gauge.
//        Anyway, lang side implementation may even not align itself on these three types...
// FIXME: Consider forwarding metric initialization to lang, so that lang can provide
//        some "configuration" that would allow for more efficient metrics collection
//

// struct CoreMetricsFieldStorage(HashMap<String, serde_json::Value>);

pub struct TypeScriptMetricsExporter {
    sender: MetricsExporterProducer,
    receiver: MetricsExporterConsumer,
}
impl TypeScriptMetricsExporter {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(8192);

        TypeScriptMetricsExporter {
            sender,
            receiver,
        }
    }
}

impl std::fmt::Debug for TypeScriptMetricsExporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypeScriptMetricsExporter").finish()
    }
}

impl CoreMeter for TypeScriptMetricsExporter {
    fn new_attributes(&self, _: MetricsAttributesOptions) -> MetricAttributes {
        MetricAttributes::Lang {
            id: 0,
            new_attributes: vec![],
        }
    }

    fn counter(&self, name: &str) -> Arc<dyn Counter> {
        Arc::new(TypeScriptMetricsExporterCounter {
            name: name.to_string(),
            sender: self.sender.downgrade(),
        })
    }

    fn histogram(&self, name: &str) -> Arc<dyn Histogram> {
        // FIXME: Get default histogram config for this metric, by calling 'default_buckets_for(name)'.
        //        Prefix must be stripped from 'name' before calling 'default_buckets_for'.
        Arc::new(TypeScriptMetricsExporterHistogram {
            name: name.to_string(),
            sender: self.sender.downgrade(),
        })
    }

    fn gauge(&self, name: &str) -> Arc<dyn Gauge> {
        Arc::new(TypeScriptMetricsExporterGauge {
            name: name.to_string(),
            sender: self.sender.downgrade(),
        })
    }
}

pub struct TypeScriptMetricsExporterCounter {
    name: String,
    sender: WeakSender<MetricEventEntry>,
}
impl Counter for TypeScriptMetricsExporterCounter {
    fn add(&self, delta: u64, _: &MetricAttributes) {
        let entry = MetricEventEntry {
            timestamp: SystemTime::now(),
            change: MetricEventEntryChange::Delta(delta),
            metric_name: self.name.clone(),
            attributes: HashMap::new(),
        };
        self.sender.upgrade().map(|s| s.try_send(entry));
    }
}

pub struct TypeScriptMetricsExporterHistogram {
    name: String,
    sender: WeakSender<MetricEventEntry>,
}
impl Histogram for TypeScriptMetricsExporterHistogram {
    fn record(&self, value: u64, _: &MetricAttributes) {
        let entry = MetricEventEntry {
            timestamp: SystemTime::now(),
            change: MetricEventEntryChange::Value(value),
            metric_name: self.name.clone(),
            attributes: HashMap::new(),
        };
        self.sender.upgrade().map(|s| s.try_send(entry));
    }
}

pub struct TypeScriptMetricsExporterGauge {
    name: String,
    sender: WeakSender<MetricEventEntry>,
}
impl Gauge for TypeScriptMetricsExporterGauge {
    fn record(&self, value: u64, _: &MetricAttributes) {
        let entry = MetricEventEntry {
            timestamp: SystemTime::now(),
            change: MetricEventEntryChange::Value(value),
            metric_name: self.name.clone(),
            attributes: HashMap::new(),
        };
        self.sender.upgrade().map(|s| s.try_send(entry));
    }
}
