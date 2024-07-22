//! 示例展示了使用ESP IDF Bluedroid BLE绑定的BLE GATT服务器。
//! 当前情况下，使用`--features experimental`进行构建。
//!
//! 可以通过任何"GATT浏览器"应用进行测试，例如：
//! 在Android上可用的"GATTBrowser"移动应用。
//!
//! 示例服务器发布了一个单一的服务，其中包含两个特性：
//! - 一个"接收"特性，客户端可以向其写入数据
//! - 一个"指示"特性，客户端可以订阅并从其接收指示
//!
//! 示例相对复杂，因为它不仅展示了如何从客户端接收数据，
//! 还演示了如何向订阅了特性的所有客户端广播数据，包括
//! 处理指示确认的过程。
//!
//! 注意Bluedroid堆栈消耗大量内存，因此`sdkconfig.defaults`应仔细配置
//! 以避免内存不足的问题。
//!
//! 下面是一个可行的配置示例，但你可能需要根据具体的应用场景做进一步调整：
//!
//! CONFIG_BT_ENABLED=y
//! CONFIG_BT_BLUEDROID_ENABLED=y
//! CONFIG_BT_CLASSIC_ENABLED=n
//! CONFIG_BTDM_CTRL_MODE_BLE_ONLY=y
//! CONFIG_BTDM_CTRL_MODE_BR_EDR_ONLY=n
//! CONFIG_BTDM_CTRL_MODE_BTDM=n
//! CONFIG_BT_BLE_42_FEATURES_SUPPORTED=y
//! CONFIG_BT_BLE_50_FEATURES_SUPPORTED=n
//! CONFIG_BT_BTC_TASK_STACK_SIZE=15000
//! CONFIG_BT_BLE_DYNAMIC_ENV_MEMORY=y

use std::sync::{ Arc, Condvar, Mutex };

use enumset::enum_set;
use esp_idf_svc::bt::ble::gap::{ AdvConfiguration, BleGapEvent, EspBleGap };
use esp_idf_svc::bt::ble::gatt::server::{ ConnectionId, EspGatts, GattsEvent, TransferId };
use esp_idf_svc::bt::ble::gatt::{
    AutoResponse,
    GattCharacteristic,
    GattDescriptor,
    GattId,
    GattInterface,
    GattResponse,
    GattServiceId,
    GattStatus,
    Handle,
    Permission,
    Property,
};
use esp_idf_svc::bt::{ BdAddr, Ble, BtDriver, BtStatus, BtUuid };
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sys::{ EspError, ESP_FAIL };

use log::{ info, warn };

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let bt = Arc::new(BtDriver::new(peripherals.modem, Some(nvs.clone()))?);

    let server = ExampleServer::new(
        Arc::new(EspBleGap::new(bt.clone())?),
        Arc::new(EspGatts::new(bt.clone())?)
    );

    info!("BLE Gap and Gatts initialized");

    let gap_server = server.clone();

    server.gap.subscribe(move |event| {
        gap_server.check_esp_status(gap_server.on_gap_event(event));
    })?;

    let gatts_server = server.clone();

    server.gatts.subscribe(move |(gatt_if, event)| {
        gatts_server.check_esp_status(gatts_server.on_gatts_event(gatt_if, event))
    })?;

    info!("BLE Gap and Gatts subscriptions initialized");

    server.gatts.register_app(APP_ID)?;

    info!("Gatts BTP app registered");

    let mut ind_data = 0_u16;

    loop {
        server.indicate(&ind_data.to_le_bytes())?;
        info!("Broadcasted indication: {ind_data}");

        ind_data = ind_data.wrapping_add(1);

        FreeRtos::delay_ms(10000);
    }
}

const APP_ID: u16 = 0;
const MAX_CONNECTIONS: usize = 2;

// 我们的服务UUID
pub const SERVICE_UUID: u128 = 0xad91b201734740479e173bed82d75f9d;

/// 我们的"接收"特性 —— 即客户端可以发送数据的地方。
pub const RECV_CHARACTERISTIC_UUID: u128 = 0xb6fccb5087be44f3ae22f85485ea42c4;
/// 我们的"指示"特性 —— 即客户端若订阅，则可接收数据。
pub const IND_CHARACTERISTIC_UUID: u128 = 0x503de214868246c4828fd59144da41be;

// 为了在下面的各个函数中获得更短的类型签名，将这些类型按照示例中的用法命名。
// 注意到，除了使用`Arc`之外，你也可以使用常规引用，但是那样你就得处理生命周期问题，
// 并且下面的签名将不会是`'static`。
type ExBtDriver = BtDriver<'static, Ble>;
type ExEspBleGap = Arc<EspBleGap<'static, Ble, Arc<ExBtDriver>>>;
type ExEspGatts = Arc<EspGatts<'static, Ble, Arc<ExBtDriver>>>;

#[derive(Debug, Clone)]
struct Connection {
    peer: BdAddr,
    conn_id: Handle,
    subscribed: bool,
    mtu: Option<u16>,
}

#[derive(Default)]
struct State {
    gatt_if: Option<GattInterface>,
    service_handle: Option<Handle>,
    recv_handle: Option<Handle>,
    ind_handle: Option<Handle>,
    ind_cccd_handle: Option<Handle>,
    connections: heapless::Vec<Connection, MAX_CONNECTIONS>,
    response: GattResponse,
    ind_confirmed: Option<BdAddr>,
}

#[derive(Clone)]
pub struct ExampleServer {
    gap: ExEspBleGap,
    gatts: ExEspGatts,
    state: Arc<Mutex<State>>,
    condvar: Arc<Condvar>,
}

impl ExampleServer {
    pub fn new(gap: ExEspBleGap, gatts: ExEspGatts) -> Self {
        Self {
            gap,
            gatts,
            state: Arc::new(Mutex::new(Default::default())),
            condvar: Arc::new(Condvar::new()),
        }
    }
}

impl ExampleServer {
    /// 向所有当前订阅我们指示特性的对等方发送（指示）数据。
    ///
    /// 使用Mutex + Condvar等待接收到指示确认。
    ///
    /// 这种复杂性仅在使用指示时是必要的。
    /// 通知实际上不发送确认，因此不需要这种同步。
    fn indicate(&self, data: &[u8]) -> Result<(), EspError> {
        for peer_index in 0..MAX_CONNECTIONS {
            // 将此数据传播给所有已连接并且订阅了我们指示特性的客户端

            let mut state = self.state.lock().unwrap();

            loop {
                if state.connections.len() <= peer_index {
                    // 我们已经向所有已连接的客户端发送了数据
                    break;
                }

                let Some(gatt_if) = state.gatt_if else {
                    // 我们在此期间丢失了gatt接口
                    break;
                };

                let Some(ind_handle) = state.ind_handle else {
                    // 我们在此期间丢失了指示句柄
                    break;
                };

                if state.ind_confirmed.is_none() {
                    let conn = &state.connections[peer_index];

                    self.gatts.indicate(gatt_if, conn.conn_id, ind_handle, data)?;

                    state.ind_confirmed = Some(conn.peer);
                    let conn = &state.connections[peer_index];

                    info!("向 {} 指示数据", conn.peer);
                    break;
                } else {
                    state = self.condvar.wait(state).unwrap();
                }
            }
        }

        Ok(())
    }

    /// 用户代码可以处理新订阅客户端的示例回调。
    ///
    /// 如果用户代码只是向所有订阅的客户端广播相同的指示，
    /// 此回调可能没有必要。
    fn on_subscribed(&self, addr: BdAddr) {
        // 在这里放置您的自定义代码或留空
        // `indicate()` 无论如何会向所有已连接的客户端发送
        warn!("客户端 {} 订阅 - 在这里放置您的自定义逻辑", addr);
    }
    /// 用户代码可以处理取消订阅客户端的示例回调。
    ///
    /// 如果用户代码只是向所有订阅的客户端广播相同的指示，
    /// 此回调可能没有必要。
    fn on_unsubscribed(&self, addr: BdAddr) {
        // 在这里放置您的自定义代码
        // `indicate()` 无论如何会向所有已连接的客户端发送
        warn!("客户端 {addr} 取消订阅 - 在这里放置您的自定义逻辑");
    }

    /// 用户代码可以处理接收到数据的示例回调
    /// 为了演示目的，数据只是被记录下来。
    fn on_recv(&self, addr: BdAddr, data: &[u8], offset: u16, mtu: Option<u16>) {
        // 在这里放置您的自定义代码
        warn!(
            "从 {addr} 接收到数据: {data:?}, 偏移量: {offset}, 最大传输单元: {mtu:?} - 在这里放置您的自定义逻辑"
        );
    }

    /// GAP事件的主要事件处理器
    fn on_gap_event(&self, event: BleGapEvent) -> Result<(), EspError> {
        info!("接收到事件: {event:?}");

        if let BleGapEvent::AdvertisingConfigured(status) = event {
            self.check_bt_status(status)?;
            self.gap.start_advertising()?;
        }

        Ok(())
    }

    /// GATTS事件的主要事件处理器
    fn on_gatts_event(&self, gatt_if: GattInterface, event: GattsEvent) -> Result<(), EspError> {
        info!("Got event: {event:?}");

        match event {
            GattsEvent::ServiceRegistered { status, app_id } => {
                self.check_gatt_status(status)?;
                if APP_ID == app_id {
                    self.create_service(gatt_if)?;
                }
            }
            GattsEvent::ServiceCreated { status, service_handle, .. } => {
                self.check_gatt_status(status)?;
                self.configure_and_start_service(service_handle)?;
            }
            GattsEvent::CharacteristicAdded { status, attr_handle, service_handle, char_uuid } => {
                self.check_gatt_status(status)?;

                self.register_characteristic(service_handle, attr_handle, char_uuid)?;
            }
            GattsEvent::DescriptorAdded { status, attr_handle, service_handle, descr_uuid } => {
                self.check_gatt_status(status)?;
                self.register_cccd_descriptor(service_handle, attr_handle, descr_uuid)?;
            }
            GattsEvent::ServiceDeleted { status, service_handle } => {
                self.check_gatt_status(status)?;
                self.delete_service(service_handle)?;
            }
            GattsEvent::ServiceUnregistered { status, service_handle, .. } => {
                self.check_gatt_status(status)?;
                self.unregister_service(service_handle)?;
            }
            GattsEvent::Mtu { conn_id, mtu } => {
                self.register_conn_mtu(conn_id, mtu)?;
            }
            GattsEvent::PeerConnected { conn_id, addr, .. } => {
                self.create_conn(conn_id, addr)?;
            }
            GattsEvent::PeerDisconnected { addr, .. } => {
                self.delete_conn(addr)?;
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
                let handled = self.recv(
                    gatt_if,
                    conn_id,
                    trans_id,
                    addr,
                    handle,
                    offset,
                    need_rsp,
                    is_prep,
                    value
                )?;

                if handled {
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
                }
            }
            GattsEvent::Confirm { status, .. } => {
                self.check_gatt_status(status)?;
                self.confirm_indication()?;
            }
            _ => (),
        }

        Ok(())
    }

    /// 创建服务并开始广播
    /// 当收到通知 GATTS 应用程序已注册时，在事件回调中调用
    fn create_service(&self, gatt_if: GattInterface) -> Result<(), EspError> {
        self.state.lock().unwrap().gatt_if = Some(gatt_if);

        self.gap.set_device_name("ESP32")?;
        self.gap.set_adv_conf(
            &(AdvConfiguration {
                include_name: true,
                include_txpower: true,
                flag: 2,
                service_uuid: Some(BtUuid::uuid128(SERVICE_UUID)),
                // service_data: todo!(), // 服务数据: 待办事项
                // manufacturer_data: todo!(), // 制造商数据: 待办事项
                ..Default::default()
            })
        )?;
        self.gatts.create_service(
            gatt_if,
            &(GattServiceId {
                id: GattId {
                    uuid: BtUuid::uuid128(SERVICE_UUID),
                    inst_id: 0,
                },
                is_primary: true,
            }),
            8
        )?;

        Ok(())
    }

    /// 删除服务
    /// 在收到 GATTS 应用已被删除的通知后，从事件回调内部调用
    fn delete_service(&self, service_handle: Handle) -> Result<(), EspError> {
        let mut state = self.state.lock().unwrap();

        if state.service_handle == Some(service_handle) {
            state.recv_handle = None;
            state.ind_handle = None;
            state.ind_cccd_handle = None;
        }

        Ok(())
    }

    /// 注销服务
    /// 在收到 GATTS 应用已被注销的通知后，从事件回调内部调用
    fn unregister_service(&self, service_handle: Handle) -> Result<(), EspError> {
        let mut state = self.state.lock().unwrap();

        if state.service_handle == Some(service_handle) {
            state.gatt_if = None;
            state.service_handle = None;
        }

        Ok(())
    }

    /// 配置并启动服务
    /// 在收到服务创建的通知后，从事件回调内部调用
    fn configure_and_start_service(&self, service_handle: Handle) -> Result<(), EspError> {
        self.state.lock().unwrap().service_handle = Some(service_handle);

        self.gatts.start_service(service_handle)?;
        self.add_characteristics(service_handle)?;

        Ok(())
    }

    /// 向服务添加我们的两个特性
    /// 在收到服务创建的通知后，从事件回调内部调用
    fn add_characteristics(&self, service_handle: Handle) -> Result<(), EspError> {
        self.gatts.add_characteristic(
            service_handle,
            &(GattCharacteristic {
                uuid: BtUuid::uuid128(RECV_CHARACTERISTIC_UUID),
                permissions: enum_set!(Permission::Write),
                properties: enum_set!(Property::Write),
                max_len: 200, // Max recv data
                auto_rsp: AutoResponse::ByApp,
            }),
            &[]
        )?;

        self.gatts.add_characteristic(
            service_handle,
            &(GattCharacteristic {
                uuid: BtUuid::uuid128(IND_CHARACTERISTIC_UUID),
                permissions: enum_set!(Permission::Write | Permission::Read),
                properties: enum_set!(Property::Indicate),
                max_len: 400, // Mac iondicate data
                auto_rsp: AutoResponse::ByApp,
            }),
            &[]
        )?;

        Ok(())
    }

    /// 添加 CCCD 描述符
    /// 在收到特征描述符被添加的通知后，从事件回调内部调用，
    /// 但是仅当添加的特征是“指示”特性时，此方法才会执行某些操作
    fn register_characteristic(
        &self,
        service_handle: Handle,
        attr_handle: Handle,
        char_uuid: BtUuid
    ) -> Result<(), EspError> {
        let indicate_char = {
            let mut state = self.state.lock().unwrap();

            if state.service_handle != Some(service_handle) {
                false
            } else if char_uuid == BtUuid::uuid128(RECV_CHARACTERISTIC_UUID) {
                state.recv_handle = Some(attr_handle);

                false
            } else if char_uuid == BtUuid::uuid128(IND_CHARACTERISTIC_UUID) {
                state.ind_handle = Some(attr_handle);

                true
            } else {
                false
            }
        };

        if indicate_char {
            self.gatts.add_descriptor(
                service_handle,
                &(GattDescriptor {
                    uuid: BtUuid::uuid16(0x2902), // CCCD
                    permissions: enum_set!(Permission::Read | Permission::Write),
                })
            )?;
        }

        Ok(())
    }

    /// 注册 CCCD 描述符
    /// 在收到描述符被添加的通知后，从事件回调内部调用
    fn register_cccd_descriptor(
        &self,
        service_handle: Handle,
        attr_handle: Handle,
        descr_uuid: BtUuid
    ) -> Result<(), EspError> {
        let mut state = self.state.lock().unwrap();

        if
            descr_uuid == BtUuid::uuid16(0x2902) && // CCCD UUID
            state.service_handle == Some(service_handle)
        {
            state.ind_cccd_handle = Some(attr_handle);
        }

        Ok(())
    }

    /// 接收客户端的数据
    /// 在接收到连接 MTU 的通知后，从事件回调内部调用
    fn register_conn_mtu(&self, conn_id: ConnectionId, mtu: u16) -> Result<(), EspError> {
        let mut state = self.state.lock().unwrap();

        if let Some(conn) = state.connections.iter_mut().find(|conn| conn.conn_id == conn_id) {
            conn.mtu = Some(mtu);
        }

        Ok(())
    }

    /// 建立新连接
    /// 在接收到新连接的通知后，从事件回调内部调用
    fn create_conn(&self, conn_id: ConnectionId, addr: BdAddr) -> Result<(), EspError> {
        let added = {
            let mut state = self.state.lock().unwrap();

            if state.connections.len() < MAX_CONNECTIONS {
                state.connections
                    .push(Connection {
                        peer: addr,
                        conn_id,
                        subscribed: false,
                        mtu: None,
                    })
                    .map_err(|_| ())
                    .unwrap();

                true
            } else {
                false
            }
        };

        if added {
            self.gap.set_conn_params_conf(addr, 10, 20, 0, 400)?;
        }

        Ok(())
    }

    /// 删除一个连接
    /// 当我们接收到断开连接的对等方的通知时，在事件回调内部调用
    fn delete_conn(&self, addr: BdAddr) -> Result<(), EspError> {
        let mut state = self.state.lock().unwrap();

        if
            let Some(index) = state.connections
                .iter()
                .position(|Connection { peer, .. }| *peer == addr)
        {
            state.connections.swap_remove(index);
        }

        Ok(())
    }

    /// 处理客户端向我们发送到 "recv" 特性的数据的辅助方法
    #[allow(clippy::too_many_arguments)]
    fn recv(
        &self,
        _gatt_if: GattInterface,
        conn_id: ConnectionId,
        _trans_id: TransferId,
        addr: BdAddr,
        handle: Handle,
        offset: u16,
        _need_rsp: bool,
        _is_prep: bool,
        value: &[u8]
    ) -> Result<bool, EspError> {
        let mut state = self.state.lock().unwrap();

        let recv_handle = state.recv_handle;
        let ind_cccd_handle = state.ind_cccd_handle;

        let Some(conn) = state.connections.iter_mut().find(|conn| conn.conn_id == conn_id) else {
            return Ok(false);
        };

        if Some(handle) == ind_cccd_handle {
            // 订阅或取消订阅我们的指示特性

            if offset == 0 && value.len() == 2 {
                let value = u16::from_le_bytes([value[0], value[1]]);
                if value == 0x02 {
                    if !conn.subscribed {
                        conn.subscribed = true;
                        self.on_subscribed(conn.peer);
                    }
                } else if conn.subscribed {
                    conn.subscribed = false;
                    self.on_unsubscribed(conn.peer);
                }
            }
        } else if Some(handle) == recv_handle {
            // 在 recv 特性上接收数据

            self.on_recv(addr, value, offset, conn.mtu);
        } else {
            return Ok(false);
        }

        Ok(true)
    }

    /// 一个辅助方法，用于向刚刚通过 "recv" 特性向我们发送数据的对等方发送响应。
    ///
    /// 这是必要的，因为我们支持写入确认
    /// （与未确认写入相比，这是一个更复杂的情况）。
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
    ) -> Result<(), EspError> {
        if !need_rsp {
            return Ok(());
        }

        if is_prep {
            let mut state = self.state.lock().unwrap();

            state.response
                .attr_handle(handle)
                .auth_req(0)
                .offset(offset)
                .value(value)
                .map_err(|_| EspError::from_infallible::<ESP_FAIL>())?;

            self.gatts.send_response(
                gatt_if,
                conn_id,
                trans_id,
                GattStatus::Ok,
                Some(&state.response)
            )?;
        } else {
            self.gatts.send_response(gatt_if, conn_id, trans_id, GattStatus::Ok, None)?;
        }

        Ok(())
    }

    /// 一个辅助方法来处理指示确认。
    /// 基本上，我们需要通知 "indicate" 方法发送的指示已被确认，
    /// 因此它可以自由地发送下一个指示。
    fn confirm_indication(&self) -> Result<(), EspError> {
        let mut state = self.state.lock().unwrap();
        if state.ind_confirmed.is_none() {
            // 不应该发生：意味着我们收到了一个我们未发送的指示的确认。
            unreachable!();
        }

        state.ind_confirmed = None; // 以便主循环可以发送下一个指示
        self.condvar.notify_all();

        Ok(())
    }

    fn check_esp_status(&self, status: Result<(), EspError>) {
        if let Err(e) = status {
            warn!("Got status: {:?}", e);
        }
    }

    fn check_bt_status(&self, status: BtStatus) -> Result<(), EspError> {
        if !matches!(status, BtStatus::Success) {
            warn!("Got status: {:?}", status);
            Err(EspError::from_infallible::<ESP_FAIL>())
        } else {
            Ok(())
        }
    }

    fn check_gatt_status(&self, status: GattStatus) -> Result<(), EspError> {
        if !matches!(status, GattStatus::Ok) {
            warn!("Got status: {:?}", status);
            Err(EspError::from_infallible::<ESP_FAIL>())
        } else {
            Ok(())
        }
    }
}
