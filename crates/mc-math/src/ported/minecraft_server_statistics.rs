use std::collections::HashMap;

pub trait MinecraftServerMetrics {
    // JAVA: net.minecraft.server.MinecraftServer#getTickTimesNanos
    fn get_tick_times_nanos(&self) -> Vec<i64>;
    // JAVA: net.minecraft.server.MinecraftServer#getCurrentSmoothedTickTime
    fn get_current_smoothed_tick_time(&self) -> f32;
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    LongArray(Vec<i64>),
    Float(f32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: AttributeValue,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AttributeList(pub Vec<Attribute>);

#[derive(Debug, Clone, PartialEq)]
pub struct MBeanAttributeInfo {
    pub name: String,
    pub type_name: String,
    pub description: String,
    pub readable: bool,
    pub writable: bool,
    pub is: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MBeanNotificationInfo {
    pub name: String,
    pub description: String,
    pub notification_types: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MBeanInfo {
    pub class_name: String,
    pub description: String,
    pub attributes: Vec<MBeanAttributeInfo>,
    pub notifications: Vec<MBeanNotificationInfo>,
}

#[derive(Debug, Clone)]
pub struct AttributeDescription {
    pub name: String,
    pub getter: fn(&dyn MinecraftServerMetrics) -> AttributeValue,
    pub description: String,
    pub type_name: String,
}

impl AttributeDescription {
    // JAVA: net.minecraft.util.monitoring.jmx.MinecraftServerStatistics.AttributeDescription#asMBeanAttributeInfo
    fn as_m_bean_attribute_info(&self) -> MBeanAttributeInfo {
        MBeanAttributeInfo {
            name: self.name.clone(),
            type_name: self.type_name.clone(),
            description: self.description.clone(),
            readable: true,
            writable: false,
            is: false,
        }
    }
}

pub struct MinecraftServerStatistics<S> {
    server: S,
    m_bean_info: MBeanInfo,
    attribute_description_by_name: HashMap<String, AttributeDescription>,
}

impl<S: MinecraftServerMetrics> MinecraftServerStatistics<S> {
    // JAVA: net.minecraft.util.monitoring.jmx.MinecraftServerStatistics#MinecraftServerStatistics
    pub fn new(server: S) -> Self {
        let mut attribute_description_by_name = HashMap::new();
        attribute_description_by_name.insert(
            "tickTimes".to_string(),
            AttributeDescription {
                name: "tickTimes".to_string(),
                getter: |server| AttributeValue::LongArray(server.get_tick_times_nanos()),
                description: "Historical tick times (ms)".to_string(),
                type_name: "long[]".to_string(),
            },
        );
        attribute_description_by_name.insert(
            "averageTickTime".to_string(),
            AttributeDescription {
                name: "averageTickTime".to_string(),
                getter: |server| AttributeValue::Float(server.get_current_smoothed_tick_time()),
                description: "Current average tick time (ms)".to_string(),
                type_name: "long".to_string(),
            },
        );

        let attributes = attribute_description_by_name
            .values()
            .map(AttributeDescription::as_m_bean_attribute_info)
            .collect();
        let m_bean_info = MBeanInfo {
            class_name: "MinecraftServerStatistics".to_string(),
            description: "metrics for dedicated server".to_string(),
            attributes,
            notifications: Vec::new(),
        };

        Self {
            server,
            m_bean_info,
            attribute_description_by_name,
        }
    }

    // JAVA: net.minecraft.util.monitoring.jmx.MinecraftServerStatistics#registerJmxMonitoring
    pub fn register_jmx_monitoring(_server: S) {
        // JMX registration is unavailable in this self-contained port.
    }

    // JAVA: net.minecraft.util.monitoring.jmx.MinecraftServerStatistics#getAverageTickTime
    pub fn get_average_tick_time(&self) -> f32 {
        self.server.get_current_smoothed_tick_time()
    }

    // JAVA: net.minecraft.util.monitoring.jmx.MinecraftServerStatistics#getTickTimes
    pub fn get_tick_times(&self) -> Vec<i64> {
        self.server.get_tick_times_nanos()
    }

    // JAVA: net.minecraft.util.monitoring.jmx.MinecraftServerStatistics#getAttribute
    pub fn get_attribute(&self, name: &str) -> Option<AttributeValue> {
        self.attribute_description_by_name
            .get(name)
            .map(|description| (description.getter)(&self.server))
    }

    // JAVA: net.minecraft.util.monitoring.jmx.MinecraftServerStatistics#setAttribute
    pub fn set_attribute(&mut self, _attribute: Attribute) {}

    // JAVA: net.minecraft.util.monitoring.jmx.MinecraftServerStatistics#getAttributes
    pub fn get_attributes(&self, names: &[&str]) -> AttributeList {
        let attributes = names
            .iter()
            .filter_map(|name| self.attribute_description_by_name.get(*name))
            .map(|description| Attribute {
                name: description.name.clone(),
                value: (description.getter)(&self.server),
            })
            .collect();
        AttributeList(attributes)
    }

    // JAVA: net.minecraft.util.monitoring.jmx.MinecraftServerStatistics#setAttributes
    pub fn set_attributes(&mut self, _attributes: AttributeList) -> AttributeList {
        AttributeList(Vec::new())
    }

    // JAVA: net.minecraft.util.monitoring.jmx.MinecraftServerStatistics#invoke
    pub fn invoke(
        &self,
        _name: &str,
        _params: &[AttributeValue],
        _signature: &[String],
    ) -> Option<AttributeValue> {
        None
    }

    // JAVA: net.minecraft.util.monitoring.jmx.MinecraftServerStatistics#getMBeanInfo
    pub fn get_m_bean_info(&self) -> &MBeanInfo {
        &self.m_bean_info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeServer {
        tick_times: Vec<i64>,
        smoothed: f32,
    }

    impl MinecraftServerMetrics for FakeServer {
        fn get_tick_times_nanos(&self) -> Vec<i64> {
            self.tick_times.clone()
        }

        fn get_current_smoothed_tick_time(&self) -> f32 {
            self.smoothed
        }
    }

    #[test]
    fn tick_times_attribute_round_trips() {
        let server = FakeServer {
            tick_times: vec![-1, 0, i64::MAX],
            smoothed: 12.5,
        };
        let stats = MinecraftServerStatistics::new(server);
        assert_eq!(
            stats.get_attribute("tickTimes"),
            Some(AttributeValue::LongArray(vec![-1, 0, i64::MAX]))
        );
    }

    #[test]
    fn average_tick_time_preserves_float_bits() {
        let bits = 0x3f8ccccd_u32;
        let smoothed = f32::from_bits(bits);
        let server = FakeServer {
            tick_times: vec![],
            smoothed,
        };
        let stats = MinecraftServerStatistics::new(server);
        assert_eq!(stats.get_average_tick_time().to_bits(), bits);
        assert_eq!(
            stats.get_attribute("averageTickTime"),
            Some(AttributeValue::Float(smoothed))
        );
    }

    #[test]
    fn get_attributes_filters_unknown_names() {
        let server = FakeServer {
            tick_times: vec![7],
            smoothed: 3.0,
        };
        let stats = MinecraftServerStatistics::new(server);
        let result = stats.get_attributes(&["averageTickTime", "missing", "tickTimes"]);
        assert_eq!(result.0.len(), 2);
        assert_eq!(result.0[0].name, "averageTickTime");
        assert_eq!(result.0[1].name, "tickTimes");
    }

    #[test]
    fn set_attributes_returns_empty_list() {
        let server = FakeServer {
            tick_times: vec![],
            smoothed: 0.0,
        };
        let mut stats = MinecraftServerStatistics::new(server);
        let result = stats.set_attributes(AttributeList(vec![Attribute {
            name: "tickTimes".to_string(),
            value: AttributeValue::LongArray(vec![1]),
        }]));
        assert!(result.0.is_empty());
    }

    #[test]
    fn invoke_returns_none() {
        let server = FakeServer {
            tick_times: vec![],
            smoothed: 0.0,
        };
        let stats = MinecraftServerStatistics::new(server);
        assert_eq!(stats.invoke("anything", &[], &[]), None);
    }

    #[test]
    fn mbean_info_has_two_attributes() {
        let server = FakeServer {
            tick_times: vec![],
            smoothed: 0.0,
        };
        let stats = MinecraftServerStatistics::new(server);
        assert_eq!(stats.get_m_bean_info().attributes.len(), 2);
    }
}
