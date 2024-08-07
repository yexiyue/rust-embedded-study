use anyhow::{ anyhow, bail };
use esp_idf_svc::bt::{
    ble::{
        gap::{ AdvConfiguration, BleGapEvent },
        gatt::{
            server::{ ConnectionId, GattsEvent, TransferId },
            GattConnParams,
            GattInterface,
            GattResponse,
            GattServiceId,
            GattStatus,
            Handle,
        },
    },
    BdAddr,
    BtStatus,
    BtUuid,
};
use std::{
    collections::{ HashMap, HashSet },
    fmt::Debug,
    hash::Hash,
    ops::Deref,
    sync::{ Arc, Condvar, Mutex },
};
use super::{ app_builder::BLEAppBuilder, ExEspBleGap, ExEspGatts, ReadExt, Service, WriteExt };

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connection {
    pub peer: BdAddr,
    pub conn_id: ConnectionId,
    pub mtu: Option<u16>,
}

#[derive(Debug, Clone, Default)]
pub struct ConnectedState {
    pub connections: Vec<Connection>,
    pub gatt_if: Option<GattInterface>,
    pub service_handle_map: HashMap<Handle, HashBtUuid>,
    pub attr_handle_map: HashMap<Handle, HashBtUuid>,
    pub notify_confirmed: Option<BdAddr>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HashBtUuid(pub BtUuid);

impl Hash for HashBtUuid {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(self.0.as_bytes())
    }
}

impl PartialEq<BtUuid> for HashBtUuid {
    fn eq(&self, other: &BtUuid) -> bool {
        self.0 == *other
    }
}

impl Deref for HashBtUuid {
    type Target = BtUuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<BtUuid> for HashBtUuid {
    fn from(value: BtUuid) -> Self {
        Self(value)
    }
}

#[derive(Clone)]
pub struct BLEApp<'a, State: Sync + Send + Clone = ()> {
    pub app_id: u16,
    pub device_name: Option<&'a str>,
    pub adv_configuration: AdvConfiguration<'a>,
    pub gap: ExEspBleGap<'a>,
    pub gatts: ExEspGatts<'a>,
    pub state: State,
    pub services: HashMap<HashBtUuid, Service<State>>,
    pub read_characteristics: HashMap<HashBtUuid, Arc<dyn ReadExt<State = State>>>,
    pub write_characteristics: HashMap<HashBtUuid, Arc<dyn WriteExt<State = State>>>,
    pub connected_state: Arc<Mutex<ConnectedState>>,
    condvar: Arc<Condvar>,
}

impl<'a, T: Sync + Send + Clone> BLEApp<'a, T> {
    pub fn builder() -> BLEAppBuilder<'a, T> {
        BLEAppBuilder::new()
    }

    pub fn new(
        app_id: u16,
        state: T,
        gap: ExEspBleGap<'a>,
        gatts: ExEspGatts<'a>,
        adv_configuration: AdvConfiguration<'a>,
        device_name: Option<&'a str>
    ) -> Self {
        Self {
            app_id,
            services: HashMap::new(),
            read_characteristics: HashMap::new(),
            write_characteristics: HashMap::new(),
            state,
            gap,
            gatts,
            connected_state: Arc::new(Mutex::new(ConnectedState::default())),
            adv_configuration,
            device_name,
            condvar: Arc::new(Condvar::new()),
        }
    }

    pub fn add_service(&mut self, service: Service<T>) {
        service.read_characteristics.iter().for_each(|i| {
            let characteristic = i.characteristic();
            self.read_characteristics.insert(characteristic.uuid.into(), i.clone());
        });
        service.write_characteristics.iter().for_each(|i| {
            let characteristic = i.characteristic();
            self.write_characteristics.insert(characteristic.uuid.into(), i.clone());
        });
        self.services.insert(service.service_id.id.uuid.clone().into(), service);
    }

    /// 在注册app 成功后，可以拿到gatt_if，这时候可以创建服务
    fn on_service_registered(&self, gatt_if: GattInterface) -> anyhow::Result<()> {
        // 配置蓝牙名称
        self.gap.set_device_name(self.device_name.unwrap_or("ESP32"))?;
        // 配置广播参数，会触发BleGapEvent::AdvertisingConfigured事件
        self.gap.set_adv_conf(&self.adv_configuration)?;

        // 创建服务
        for i in self.services.values() {
            self.gatts.create_service(gatt_if, &i.service_id, i.num_handles)?;
        }
        self.connected_state.lock().unwrap().gatt_if = Some(gatt_if);
        Ok(())
    }

    /// 在服务创建好的时候启动服务，并做好映射
    fn on_service_created(
        &self,
        service_handle: Handle,
        service_id: GattServiceId
    ) -> anyhow::Result<()> {
        let hash_bt_uuid: HashBtUuid = service_id.id.uuid.into();
        let Some(service) = self.services.get(&hash_bt_uuid) else { bail!("Service not found") };

        let mut connected_state = self.connected_state.lock().unwrap();
        connected_state.service_handle_map.insert(service_handle, hash_bt_uuid);
        // 添加特征
        let mut uuids: HashSet<HashBtUuid> = HashSet::new();

        for i in &service.write_characteristics {
            let characteristic = i.characteristic();
            if uuids.insert(i.characteristic().uuid.into()) {
                self.gatts.add_characteristic(service_handle, &characteristic, &[])?;
            }
        }

        for i in &service.read_characteristics {
            let characteristic = i.characteristic();
            if uuids.insert(i.characteristic().uuid.into()) {
                self.gatts.add_characteristic(service_handle, &characteristic, &[])?;
            }
        }

        Ok(())
    }

    // 在添加特征完成后，可以拿到attr_handle,这时候做好映射就OK
    fn on_characteristic_added(
        &self,
        attr_handle: Handle,
        service_handle: Handle,
        char_uuid: BtUuid
    ) -> anyhow::Result<()> {
        let hash_uuid: HashBtUuid = char_uuid.into();

        let descriptors = if let Some(c) = self.read_characteristics.get(&hash_uuid) {
            c.descriptors()
        } else if let Some(c) = self.write_characteristics.get(&hash_uuid) {
            c.descriptors()
        } else {
            bail!("No characteristic found for uuid {:?}", hash_uuid)
        };
        let mut connected_state = self.connected_state.lock().unwrap();
        connected_state.attr_handle_map.insert(attr_handle, hash_uuid.clone());
        for i in descriptors {
            self.gatts.add_descriptor(service_handle, &i)?;
        }
        Ok(())
    }

    fn on_write(&self, attr_handle: Handle, value: &[u8]) -> anyhow::Result<()> {
        let connect_state = self.connected_state.lock().unwrap();
        if let Some(uuid) = connect_state.attr_handle_map.get(&attr_handle) {
            let Some(characteristic) = self.write_characteristics.get(uuid) else {
                bail!("characteristic not found")
            };

            characteristic.on_write(self.state.clone(), value)?;
        }

        Ok(())
    }

    fn on_read(&self, attr_handle: Handle) -> anyhow::Result<&[u8]> {
        let connected_state = self.connected_state.lock().unwrap();
        let uuid = connected_state.attr_handle_map
            .get(&attr_handle)
            .ok_or(anyhow!("attr_handle not found"))?;
        let Some(characteristic) = self.read_characteristics.get(uuid) else {
            bail!("characteristic not found")
        };
        characteristic.on_read(self.state.clone())
    }

    fn on_peer_connected(
        &self,
        conn_id: ConnectionId,
        addr: BdAddr,
        GattConnParams { .. }: GattConnParams
    ) -> anyhow::Result<()> {
        let mut connected_state = self.connected_state.lock().unwrap();
        connected_state.connections.push(Connection {
            peer: addr,
            conn_id,
            mtu: None,
        });
        self.gap.set_conn_params_conf(addr, 10, 20, 0, 400)?;
        Ok(())
    }

    pub(crate) fn on_gap_event(&self, event: BleGapEvent) -> anyhow::Result<()> {
        match event {
            BleGapEvent::AdvertisingConfigured(status) => {
                self.check_bt_status(status)?;
                self.gap.start_advertising()?;
            }
            _ => {}
        }
        Ok(())
    }

    pub(crate) fn on_gatts_event(
        &self,
        gatt_if: GattInterface,
        event: GattsEvent
    ) -> anyhow::Result<()> {
        match event {
            GattsEvent::ServiceRegistered { status, app_id } => {
                self.check_gatt_status(status)?;
                if app_id == self.app_id {
                    self.on_service_registered(gatt_if)?;
                }
            }
            GattsEvent::ServiceCreated { status, service_handle, service_id } => {
                self.check_gatt_status(status)?;
                self.gatts.start_service(service_handle)?;
                self.on_service_created(service_handle, service_id)?;
            }
            GattsEvent::CharacteristicAdded { status, attr_handle, char_uuid, service_handle } => {
                self.check_gatt_status(status)?;
                self.on_characteristic_added(attr_handle, service_handle, char_uuid)?;
            }
            GattsEvent::Write {
                conn_id,
                trans_id,
                addr,
                handle,
                offset,
                need_rsp,
                is_prep,
                value,
            } => {
                log::warn!("{addr:?} write  conn_id:{:?} value:{:?}", conn_id, value);
                self.send_write_response(
                    gatt_if,
                    conn_id,
                    trans_id,
                    handle,
                    offset,
                    need_rsp,
                    is_prep,
                    value
                )?;
                self.on_write(handle, value)?;
            }
            GattsEvent::Read { conn_id, trans_id, addr, handle, offset, need_rsp, .. } => {
                log::info!("{addr:?} read  conn_id:{:?}", conn_id);
                // 返回响应
                if need_rsp {
                    let mut response = GattResponse::new();
                    response
                        .attr_handle(handle)
                        .auth_req(0)
                        .offset(offset)
                        .value(self.on_read(handle)?)?;

                    self.gatts.send_response(
                        gatt_if,
                        conn_id,
                        trans_id,
                        GattStatus::Ok,
                        Some(&response)
                    )?;
                }
            }
            GattsEvent::PeerConnected { conn_id, addr, conn_params, .. } => {
                self.on_peer_connected(conn_id, addr, conn_params)?;
            }
            GattsEvent::Confirm { status, .. } => {
                self.check_gatt_status(status)?;
                self.confirm_notify()?;
            }
            _ => {}
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn send_write_response(
        &self,
        gatt_if: GattInterface,
        conn_id: ConnectionId,
        trans_id: TransferId,
        handle: Handle,
        offset: u16,
        need_rsp: bool,
        is_prep: bool,
        value: &[u8]
    ) -> anyhow::Result<()> {
        if need_rsp {
            if is_prep {
                let mut response = GattResponse::new();
                response.attr_handle(handle).auth_req(0).offset(offset).value(value)?;

                self.gatts.send_response(
                    gatt_if,
                    conn_id,
                    trans_id,
                    GattStatus::Ok,
                    Some(&response)
                )?;
            } else {
                self.gatts.send_response(gatt_if, conn_id, trans_id, GattStatus::Ok, None)?;
            }
        }

        Ok(())
    }

    fn check_bt_status(&self, status: BtStatus) -> anyhow::Result<()> {
        if matches!(status, BtStatus::Success) {
            Ok(())
        } else {
            bail!("BtStatus error:{:?}", status)
        }
    }

    fn check_gatt_status(&self, status: GattStatus) -> anyhow::Result<()> {
        if matches!(status, GattStatus::Ok) {
            Ok(())
        } else {
            bail!("GattStatus error:{:?}", status)
        }
    }

    pub fn notify<F>(&self, char_uuid: &BtUuid, f: F) -> anyhow::Result<()>
        where F: Fn(&[Connection], T) -> anyhow::Result<Vec<(&Connection, &[u8])>>
    {
        let mut connected_state = self.connected_state.lock().unwrap();
        let gatts_if = connected_state.gatt_if.ok_or(anyhow!("gatt_if not found"))?;
        let attr_handle = connected_state.attr_handle_map
            .iter()
            .find_map(|(attr_handle, uuid)| (
                if uuid == char_uuid {
                    Some(*attr_handle)
                } else {
                    None
                }
            ))
            .ok_or(anyhow!("attr_handle not found"))?;
        let connections = connected_state.connections.clone();
        let connect_data = f(&connections, self.state.clone())?;

        for (conn, data) in connect_data {
            log::warn!("notify conn:{:?} data:{:?}", conn, data);
            if connected_state.notify_confirmed.is_none() {
                self.gatts.notify(gatts_if, conn.conn_id, attr_handle, data)?;
                connected_state.notify_confirmed = Some(conn.peer);
            } else {
                connected_state = self.condvar.wait(connected_state).unwrap();
            }
        }

        Ok(())
    }

    fn confirm_notify(&self) -> anyhow::Result<()> {
        let mut state = self.connected_state.lock().unwrap();
        if state.notify_confirmed.is_none() {
            // 不应该发生：意味着我们收到了一个我们未发送的指示的确认。
            unreachable!();
        }

        state.notify_confirmed = None; // 以便主循环可以发送下一个指示
        self.condvar.notify_all();

        Ok(())
    }
}
