use std::sync::Arc;

use esp_idf_svc::bt::ble::{ gap::{ AdvConfiguration, EspBleGap }, gatt::server::EspGatts };

use super::{ BLEApp, ExBtDriver, ExEspBleGap, ExEspGatts };

#[derive(Clone, Default)]
pub struct BLEAppBuilder<'a, State: Sync + Send + Clone = ()> {
    pub app_id: Option<u16>,
    pub device_name: Option<&'a str>,
    pub adv_configuration: Option<AdvConfiguration<'a>>,
    pub gap: Option<ExEspBleGap<'a>>,
    pub gatts: Option<ExEspGatts<'a>>,
    pub state: Option<State>,
}

impl<'a, State: Sync + Send + Clone> BLEAppBuilder<'a, State> {
    pub fn new() -> Self {
        Self {
            app_id: None,
            device_name: None,
            adv_configuration: None,
            gap: None,
            gatts: None,
            state: None,
        }
    }

    pub fn app_id(&mut self, app_id: u16) -> &mut Self {
        self.app_id = Some(app_id);
        self
    }

    pub fn device_name(&mut self, device_name: &'a str) -> &mut Self {
        self.device_name = Some(device_name);
        self
    }

    pub fn adv_configuration(&mut self, adv_configuration: AdvConfiguration<'a>) -> &mut Self {
        self.adv_configuration = Some(adv_configuration);
        self
    }

    pub fn state(&mut self, state: State) -> &mut Self {
        self.state = Some(state);
        self
    }

    pub fn driver(&mut self, driver: ExBtDriver<'a>) -> anyhow::Result<&mut Self> {
        let bt = Arc::new(driver);
        self.gap = Some(Arc::new(EspBleGap::new(bt.clone())?));
        self.gatts = Some(Arc::new(EspGatts::new(bt.clone())?));
        Ok(self)
    }

    pub fn build(&self) -> BLEApp<'a, State> {
        BLEApp::new(
            self.app_id.unwrap(),
            self.state.clone().unwrap(),
            self.gap.clone().unwrap(),
            self.gatts.clone().unwrap(),
            self.adv_configuration.clone().unwrap(),
            self.device_name
        )
    }
}
