pub trait PrometheusDataSource {
    fn get(&self) -> &prometheus::Registry;
}
