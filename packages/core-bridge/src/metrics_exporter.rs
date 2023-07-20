use std::sync::Arc;

use temporal_sdk_core::api::telemetry::metrics::{CoreMeter, MetricsAttributesOptions, MetricAttributes, Counter, Gauge, Histogram};

#[derive(Debug)]
pub struct TypeScriptMetricsExporter;
impl CoreMeter for TypeScriptMetricsExporter {
    fn new_attributes(&self, _: MetricsAttributesOptions) -> MetricAttributes {
        MetricAttributes::Lang {
            id: 0,
            new_attributes: vec![],
        }
    }

    fn counter(&self, name: &str) -> Arc<dyn Counter> {
        Arc::new(TypeScriptMetricsExporterCounter { name })
    }

    fn histogram(&self, name: &str) -> Arc<dyn Histogram> {
        Arc::new(TypeScriptMetricsExporterHistogram { name })
    }

    fn gauge(&self, name: &str) -> Arc<dyn Gauge> {
        Arc::new(TypeScriptMetricsExporterGauge { name })
    }
}

pub struct TypeScriptMetricsExporterCounter {
    name: &'static str,
}
impl Counter for TypeScriptMetricsExporterCounter {
    fn add(&self, _: u64, _: &MetricAttributes) {}
}

pub struct TypeScriptMetricsExporterHistogram {
        name: &'static str,
}
impl Histogram for TypeScriptMetricsExporterHistogram {
    fn record(&self, _: u64, _: &MetricAttributes) {}
}

pub struct TypeScriptMetricsExporterGauge {
        name: &'static str,
}
impl Gauge for TypeScriptMetricsExporterGauge {
    fn record(&self, _: u64, _: &MetricAttributes) {}
}
